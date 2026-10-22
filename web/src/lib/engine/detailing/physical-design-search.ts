/** Bounded exploration of physical repair alternatives. No store writes or structural solves. */
import type { DesignCodeAdapter } from '../design/code-adapter';
import type { MemberContext } from '../design/member-context';
import { designMember, outcomeFromProvided } from '../design/candidate-search';
import { beamCornerRecord, type BeamCornerRecord } from '../design/end-support-anchorage';
import { STANDARD_LONG_DIAS } from '../design/objective';
import { rebarHash } from '../design/rebar-hash';
import { detailableReinforcement, type MemberDesignOutcome } from '../design/outcome';
import type { RunDetailingResult } from './run-detailing';
import { beamLayerAlternatives, type BeamLayerAlternative } from './beam-layer-alternatives';
import { beamContinuityAlternatives } from './beam-continuity-alternatives';

export type PhysicalRepairOperator = 'diameterFamily' | 'redistributeLayers' | 'harmonizeTop';

export interface PhysicalSearchBudget {
  /** Full geometry builds, including rejected trials. */
  maxDetailingRuns: number;
  /** Nominal member searches, each additionally bounded by designMember's budget. */
  maxMemberSearches: number;
  /** Ordered repair operators. Omitted preserves the original diameter search. */
  operators?: readonly PhysicalRepairOperator[];
}

export const DEFAULT_PHYSICAL_SEARCH_BUDGET: PhysicalSearchBudget = {
  maxDetailingRuns: 8, maxMemberSearches: 64,
};

export interface PhysicalSearchTrial {
  elementId: number;
  /** Atomic replacement; elementId remains the representative for single-member readers. */
  elementIds?: number[];
  jointId?: string;
  operator: PhysicalRepairOperator;
  diameterMm?: number;
  region?: BeamLayerAlternative['region'];
  counts?: number[];
  disposition: 'nominalFailure' | 'unchanged' | 'regression' | 'noProgress' | 'accepted';
  /** Members acquiring a physical condition they did not previously have. */
  newlyBlocked?: number[];
  blockingConflicts?: number;
  nominalFailures?: number[];
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
  const operators = [...new Set<PhysicalRepairOperator>(input.budget.operators ?? ['diameterFamily'])];
  if (operators.some(o => o !== 'diameterFamily' && o !== 'redistributeLayers' && o !== 'harmonizeTop')) {
    throw new Error('Unknown physical repair operator');
  }
  if (operators.includes('harmonizeTop')) {
    if (operators.length === 1) return searchBeamContinuity(input);
    // Atomic pair repair is a separate sweep. All sweeps share the caller's budgets.
    let retained: PhysicalSearchResult = { outcomes: input.outcomes, result: input.result, trials: [],
      changed: [], termination: 'sweepComplete', stats: { detailingRuns: 0, memberSearches: 0,
        candidatesConsidered: 0, verifierCalls: 0, structuralSolves: 0 } };
    for (const operator of operators) {
      const next = searchPhysicalDesignAlternatives({ ...input, outcomes: retained.outcomes, result: retained.result,
        budget: { operators: [operator], maxDetailingRuns: input.budget.maxDetailingRuns - retained.stats.detailingRuns,
          maxMemberSearches: input.budget.maxMemberSearches - retained.stats.memberSearches } });
      const stats = { ...retained.stats };
      for (const key of ['detailingRuns', 'memberSearches', 'candidatesConsidered', 'verifierCalls'] as const) stats[key] += next.stats[key];
      retained = { ...next, stats, trials: [...retained.trials, ...next.trials],
        changed: [...new Set([...retained.changed, ...next.changed])].sort((a, b) => a - b) };
      if (next.termination === 'budget') break;
    }
    return retained;
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
    function* proposals(): Generator<{ trial: PhysicalSearchTrial; evaluate: () => MemberDesignOutcome | null }> {
      for (const operator of operators) {
        if (operator === 'diameterFamily') {
          for (const diameterMm of diameters) yield {
            trial: { elementId: id, operator, diameterMm, disposition: 'nominalFailure' },
            evaluate: () => designMember(input.adapter, {
              ...ctx!, material: { ...ctx!.material, longDiametersMm: [diameterMm] },
            }),
          };
        } else {
          for (const alternative of beamLayerAlternatives(ctx!, current!.accepted!)) yield {
            trial: { elementId: id, operator, region: alternative.region, counts: alternative.counts, disposition: 'nominalFailure' },
            evaluate: () => {
              const verdict = input.adapter.verify(ctx!, alternative.reinforcement);
              const checked = outcomeFromProvided(input.adapter, ctx!, alternative.reinforcement, verdict);
              if (checked) checked.searchStats = { ...checked.searchStats, candidatesTried: 1, verifierCalls: 1 };
              return checked;
            },
          };
        }
      }
    }
    for (const proposal of proposals()) {
      if (stats.detailingRuns >= input.budget.maxDetailingRuns
        || stats.memberSearches >= input.budget.maxMemberSearches) {
        termination = 'budget'; break members;
      }
      const candidate = proposal.evaluate();
      stats.memberSearches++;
      const direct = proposal.trial.operator === 'redistributeLayers';
      stats.candidatesConsidered += direct ? 1 : candidate!.searchStats.candidatesTried;
      stats.verifierCalls += direct ? 1 : candidate!.searchStats.verifierCalls;
      const trial = proposal.trial;
      trials.push(trial);
      if (!candidate || candidate.outcome !== 'VERIFIED' || !candidate.accepted) continue;
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

/** Uniform top patterns are tried on a whole pair, never published one member at a time. */
function searchBeamContinuity(input: Parameters<typeof searchPhysicalDesignAlternatives>[0]): PhysicalSearchResult {
  let result = input.result, outcomes = input.outcomes;
  const trials: PhysicalSearchTrial[] = [], changed = new Set<number>();
  const stats: PhysicalSearchResult['stats'] = { detailingRuns: 0, memberSearches: 0,
    candidatesConsidered: 0, verifierCalls: 0, structuralSolves: 0 };
  let termination: PhysicalSearchResult['termination'] = 'sweepComplete';
  // Only real transitions with an open splice condition. No inference from member numbering.
  const conditions = (r: RunDetailingResult, jointId: string) => r.assemblies.flatMap(a => a.unsupported)
    .filter(u => (u.key === 'detailing.lapCheck.unarranged' || u.key === 'detailing.lapCheck.integrity')
      && u.params?.joint === jointId).length;
  const pairs = new Map<string, { ids: [number, number]; jointId: string }>();
  for (const lap of result.lapping?.laps ?? []) {
    if (lap.fromElementId === undefined || lap.toElementId === undefined || lap.fromElementId === lap.toElementId) continue;
    const ids = [lap.fromElementId, lap.toElementId].sort((a, b) => a - b) as [number, number];
    if (conditions(result, lap.jointId)) pairs.set(`${lap.jointId}:${ids.join(',')}`, { ids, jointId: lap.jointId });
  }
  // Start with smaller blocked neighbourhoods: many repeated stirrup clashes should not
  // consume every trial before a simple continuity defect is considered.
  const burden = (ids: number[]) => result.assemblies.reduce((n, a) => n + a.conflicts
    .filter(c => c.severity !== 'marginal' && c.elementIds.some(id => ids.includes(id))).length, 0);
  const ranked = [...pairs.values()].map(p => ({ ...p, burden: burden(p.ids) }))
    .sort((a, b) => a.burden - b.burden || a.ids[0] - b.ids[0] || a.ids[1] - b.ids[1]);
  const seen = new Set<string>();
  pairLoop: for (const { ids, jointId } of ranked) {
    if (!conditions(result, jointId)) continue;
    const contexts = contextsFor(input.contexts, outcomes);
    if (ids.some(id => !contexts.has(id) || contexts.get(id)!.elementType !== 'beam'
      || contexts.get(id)!.span || input.lockedMembers?.has(id)
      || outcomes.get(id)?.outcome !== 'VERIFIED' || !outcomes.get(id)?.accepted)) continue;
    const ctx = ids.map(id => contexts.get(id)!) as [MemberContext, MemberContext];
    const current = ids.map(id => outcomes.get(id)!.accepted!) as [NonNullable<MemberDesignOutcome['accepted']>, NonNullable<MemberDesignOutcome['accepted']>];
    for (const proposal of beamContinuityAlternatives(ctx, current)) {
      if (stats.detailingRuns >= input.budget.maxDetailingRuns || stats.memberSearches + 2 > input.budget.maxMemberSearches) {
        termination = 'budget'; break pairLoop;
      }
      if (proposal.reinforcement.every((r, i) => rebarHash(r) === rebarHash(current[i]))) continue;
      // Neighbours are part of the key: a refusal from an earlier assignment is not final.
      const hash = [...outcomes].map(([id, o]) => `${id}:${o.accepted ? rebarHash(o.accepted) : o.outcome}`).join('|')
        + `>${ids.join(',')}:${proposal.reinforcement.map(rebarHash).join(',')}`;
      if (seen.has(hash)) continue;
      seen.add(hash);
      const proposed = new Map(outcomes);
      ids.forEach((id, i) => proposed.set(id, { ...outcomes.get(id)!, accepted: proposal.reinforcement[i] }));
      const trialContexts = contextsFor(input.contexts, proposed);
      const trial: PhysicalSearchTrial = { elementId: ids[0], elementIds: [...ids], jointId,
        operator: 'harmonizeTop', diameterMm: proposal.layers[0].diameter,
        counts: proposal.layers.map(l => l.count), disposition: 'nominalFailure', nominalFailures: [] };
      trials.push(trial);
      for (let i = 0; i < ids.length; i++) {
        const c = trialContexts.get(ids[i])!, reinforcement = proposal.reinforcement[i];
        const verdict = input.adapter.verify(c, reinforcement);
        const checked = outcomeFromProvided(input.adapter, c, reinforcement, verdict);
        stats.memberSearches++; stats.candidatesConsidered++; stats.verifierCalls++;
        // outcomeFromProvided also admits a warning asking the caller to add a hook.
        // This operator preserves end treatments; it cannot certify an unbuilt hook.
        if ((verdict.overallStatus !== 'ok' && verdict.overallStatus !== 'warn')
          || verdict.checks.some(c => c.anchorHook) || !checked?.accepted || checked.outcome !== 'VERIFIED') trial.nominalFailures!.push(ids[i]);
        else {
          checked.searchStats = { ...checked.searchStats, candidatesTried: 1, verifierCalls: 1 };
          proposed.set(ids[i], checked);
        }
      }
      if (trial.nominalFailures!.length) continue;
      const detailed = input.detail(proposed, trialContexts);
      stats.detailingRuns++;
      const before = physicallyBlockedMembers(result), after = physicallyBlockedMembers(detailed);
      trial.newlyBlocked = [...after].filter(id => !before.has(id)).sort((a, b) => a - b);
      trial.blockingConflicts = conflicts(detailed);
      if (trial.newlyBlocked.length || ids.some(id => !detailed.reverification.some(r => r.elementId === id
        && (r.status === 'ok' || r.status === 'warn'))) || input.regresses(result, detailed)) {
        trial.disposition = 'regression'; continue;
      }
      // Do not accept a change that merely displaces this joint's unresolved splice.
      if (conditions(detailed, jointId) >= conditions(result, jointId)) { trial.disposition = 'noProgress'; continue; }
      trial.disposition = 'accepted';
      ids.forEach((id, i) => { if (rebarHash(proposal.reinforcement[i]) !== rebarHash(current[i])) changed.add(id); });
      outcomes = proposed; result = detailed;
      break;
    }
  }
  return { outcomes, result, trials, changed: [...changed].sort((a, b) => a - b), termination, stats };
}
