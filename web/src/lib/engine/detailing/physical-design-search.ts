/** Bounded exploration of physical repair alternatives. No store writes or structural solves. */
import type { DesignCodeAdapter } from '../design/code-adapter';
import type { MemberContext } from '../design/member-context';
import { designMember } from '../design/candidate-search';
import { beamCornerRecord, type BeamCornerRecord } from '../design/end-support-anchorage';
import { STANDARD_LONG_DIAS } from '../design/objective';
import { rebarHash } from '../design/rebar-hash';
import { detailableReinforcement, type MemberDesignOutcome } from '../design/outcome';
import type { RunDetailingResult } from './run-detailing';

export interface PhysicalSearchBudget {
  /** Full geometry builds, including rejected trials. */
  maxDetailingRuns: number;
  /** Nominal member searches, each additionally bounded by designMember's budget. */
  maxMemberSearches: number;
}

export const DEFAULT_PHYSICAL_SEARCH_BUDGET: PhysicalSearchBudget = {
  maxDetailingRuns: 8, maxMemberSearches: 64,
};

export interface PhysicalSearchTrial {
  elementId: number;
  diameterMm: number;
  disposition: 'nominalFailure' | 'unchanged' | 'regression' | 'noProgress' | 'accepted';
  /** Members acquiring a physical condition they did not previously have. */
  newlyBlocked?: number[];
  blockingConflicts?: number;
}

export interface PhysicalSearchResult {
  outcomes: ReadonlyMap<number, MemberDesignOutcome>;
  result: RunDetailingResult;
  trials: PhysicalSearchTrial[];
  changed: number[];
  /** A completed sweep can stop at the first improvement per member; not an exhaustion proof. */
  termination: 'budget' | 'sweepComplete';
  stats: { detailingRuns: number; memberSearches: number; candidatesConsidered: number; verifierCalls: number; structuralSolves: 0 };
}

/** Conservative scope: an unscoped condition blocks its whole assembly. */
export function physicallyBlockedMembers(result: RunDetailingResult): Set<number> {
  return new Set(result.assemblies.flatMap(a => [
    ...a.conflicts.filter(c => c.severity !== 'marginal').flatMap(c => c.elementIds),
    ...a.unsupported.flatMap(u => u.scope?.elementIds ?? a.elementIds),
  ]));
}

const conflicts = (r: RunDetailingResult) => r.assemblies.reduce((n, a) =>
  n + a.conflicts.filter(c => c.severity !== 'marginal').length, 0);

/** Refresh the neighboring beam seats after every accepted assignment. */
function contextsFor(
  contexts: ReadonlyMap<number, MemberContext>, outcomes: ReadonlyMap<number, MemberDesignOutcome>,
): ReadonlyMap<number, MemberContext> {
  const corners = new Map<number, BeamCornerRecord>();
  for (const [id, ctx] of contexts) {
    if (ctx.elementType !== 'beam') continue;
    const record = beamCornerRecord(detailableReinforcement(outcomes.get(id))?.rebar, ctx.material.stirrupDia);
    if (record) corners.set(id, record);
  }
  return new Map([...contexts].map(([id, ctx]) => [id,
    ctx.elementType === 'beam' ? { ...ctx, beamCorners: corners } : ctx]));
}

export function searchPhysicalDesignAlternatives(input: {
  adapter: DesignCodeAdapter;
  contexts: ReadonlyMap<number, MemberContext>;
  outcomes: ReadonlyMap<number, MemberDesignOutcome>;
  result: RunDetailingResult;
  detail: (outcomes: ReadonlyMap<number, MemberDesignOutcome>, contexts: ReadonlyMap<number, MemberContext>) => RunDetailingResult;
  /** Shared full-detail acceptance gate, supplied by the feedback loop. */
  regresses: (before: RunDetailingResult, after: RunDetailingResult) => boolean;
  lockedMembers?: ReadonlySet<number>;
  budget: PhysicalSearchBudget;
}): PhysicalSearchResult {
  for (const value of [input.budget.maxDetailingRuns, input.budget.maxMemberSearches]) {
    if (!Number.isSafeInteger(value) || value < 0) throw new Error('Physical search budgets must be non-negative integers');
  }
  let result = input.result, outcomes = input.outcomes;
  let contexts = contextsFor(input.contexts, outcomes);
  const trials: PhysicalSearchTrial[] = [], changed = new Set<number>();
  const stats: PhysicalSearchResult['stats'] = { detailingRuns: 0, memberSearches: 0, candidatesConsidered: 0, verifierCalls: 0, structuralSolves: 0 };
  let termination: PhysicalSearchResult['termination'] = 'sweepComplete';
  const ranked = new Map<number, number>();
  for (const a of result.assemblies) {
    for (const c of a.conflicts) {
      if (c.severity === 'marginal') continue;
      for (const id of new Set(c.elementIds)) ranked.set(id, (ranked.get(id) ?? 0) + 1);
    }
  }
  // One sweep, most physical conflicts first. Do not repeatedly spend the budget
  // on the same member; a rejected trial does not stop exploration of its alternatives.
  members: for (const [id] of [...ranked].sort((a, b) => b[1] - a[1] || a[0] - b[0])) {
    const ctx = contexts.get(id), current = outcomes.get(id);
    // Multi-element spans require an atomic group replacement, which this first operator
    // does not implement. Never modify just one analysis piece or a provisional design.
    if (ctx?.elementType !== 'beam' || ctx.span || current?.outcome !== 'VERIFIED'
      || !current.accepted || input.lockedMembers?.has(id) || !physicallyBlockedMembers(result).has(id)) continue;
    const allowed = ctx.material.longDiametersMm ?? STANDARD_LONG_DIAS;
    // A priority, not a restricted project catalog. Remaining sizes remain alternatives.
    // The operator asks whether one diameter family can simplify this member's cage;
    // it does not assume that larger bars improve anchorage or physical clearance.
    const diameters = [...new Set([16, 12, 20, 25, ...allowed])].filter(d => allowed.includes(d));
    const seen = new Set([rebarHash(current.accepted)]);
    for (const diameterMm of diameters) {
      if (stats.detailingRuns >= input.budget.maxDetailingRuns
        || stats.memberSearches >= input.budget.maxMemberSearches) {
        termination = 'budget'; break members;
      }
      const candidate = designMember(input.adapter, {
        ...ctx, material: { ...ctx.material, longDiametersMm: [diameterMm] },
      });
      stats.memberSearches++;
      stats.candidatesConsidered += candidate.searchStats.candidatesTried;
      stats.verifierCalls += candidate.searchStats.verifierCalls;
      const trial: PhysicalSearchTrial = { elementId: id, diameterMm, disposition: 'nominalFailure' };
      trials.push(trial);
      if (candidate.outcome !== 'VERIFIED' || !candidate.accepted) continue;
      const hash = rebarHash(candidate.accepted);
      if (seen.has(hash)) { trial.disposition = 'unchanged'; continue; }
      seen.add(hash);
      const proposed = new Map(outcomes).set(id, candidate);
      // Updating the seats is required both for the next search and for this trial's
      // authoritative re-verification. Demands and sections are unchanged.
      const trialContexts = contextsFor(input.contexts, proposed);
      const detailed = input.detail(proposed, trialContexts);
      stats.detailingRuns++;
      const before = physicallyBlockedMembers(result), after = physicallyBlockedMembers(detailed);
      trial.newlyBlocked = [...after].filter(member => !before.has(member)).sort((a, b) => a - b);
      trial.blockingConflicts = conflicts(detailed);
      if (trial.newlyBlocked.length || input.regresses(result, detailed)) {
        trial.disposition = 'regression'; continue;
      }
      if (after.size >= before.size && trial.blockingConflicts >= conflicts(result)) {
        trial.disposition = 'noProgress'; continue;
      }
      trial.disposition = 'accepted';
      outcomes = proposed; result = detailed; contexts = trialContexts; changed.add(id);
      break;
    }
  }
  return { outcomes, result, trials, changed: [...changed].sort((a, b) => a - b), termination, stats };
}
