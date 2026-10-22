import { afterEach, describe, expect, it, vi } from 'vitest';
import * as design from '../../design/candidate-search';
import { cirsoc201Adapter } from '../../design/adapters/cirsoc201-adapter';
import type { MemberContext } from '../../design/member-context';
import type { MemberDesignOutcome } from '../../design/outcome';
import type { RunDetailingResult } from '../run-detailing';
import { detailRegresses, runDesignFeedbackLoop } from '../design-feedback-loop';
import { searchPhysicalDesignAlternatives } from '../physical-design-search';

function outcome(id: number, diameter: number): MemberDesignOutcome {
  return { elementId: id, outcome: 'VERIFIED', accepted: {
    regions: { bottomSpan: { count: 2, diameter }, topStart: { count: 2, diameter }, topEnd: { count: 2, diameter } },
  }, searchStats: { candidatesTried: 1, verifierCalls: 1 } } as MemberDesignOutcome;
}

function detail(blocked: number[], fail: number[] = []): RunDetailingResult {
  return {
    assemblies: [{ id: 'floor', elementIds: [1, 2], unsupported: [], conflicts: blocked.map(id => ({
      elementIds: [id], severity: 'overlap',
    })) }],
    reverification: [1, 2].map(elementId => ({ elementId, status: fail.includes(elementId) ? 'fail' : 'ok' })),
    skipped: [], provisionalMembers: [],
  } as unknown as RunDetailingResult;
}

function harness() {
  const contexts = new Map([1, 2].map(elementId => [elementId, {
    elementId, elementType: 'beam', material: { stirrupDia: 8, longDiametersMm: [12, 16] },
  } as unknown as MemberContext]));
  const outcomes = new Map([1, 2].map(id => [id, outcome(id, 10)]));
  const generate = vi.spyOn(design, 'designMember').mockImplementation((_adapter, ctx) =>
    outcome(ctx.elementId, ctx.material.longDiametersMm![0]));
  return {
    generate, input: { contexts, outcomes, result: detail([1, 1]), adapter: cirsoc201Adapter,
      regresses: detailRegresses, budget: { maxDetailingRuns: 3, maxMemberSearches: 6 },
      detail: vi.fn((_outcomes: ReadonlyMap<number, MemberDesignOutcome>, _contexts: ReadonlyMap<number, MemberContext>) => detail([])),
    },
  };
}

afterEach(() => vi.restoreAllMocks());

describe('guided physical alternatives', () => {
  it('rejects damage to a neighbor, then tries another diameter and retains only the accepted assignment', () => {
    const { input } = harness();
    input.detail.mockReturnValueOnce(detail([], [2])).mockReturnValueOnce(detail([]));
    const r = searchPhysicalDesignAlternatives(input);
    expect(r.trials.map(t => t.disposition)).toEqual(['regression', 'accepted']);
    expect(r.outcomes.get(1)!.accepted!.regions!.bottomSpan!.diameter).toBe(12);
    expect(input.outcomes.get(1)!.accepted!.regions!.bottomSpan!.diameter).toBe(10);
    expect(r.outcomes.get(2)).toBe(input.outcomes.get(2));
    expect(r.result).toBe(input.detail.mock.results[1].value);
    expect(r.stats.detailingRuns).toBe(2);
    expect(r.stats.structuralSolves).toBe(0);
  });

  it('preserves physical approvals by identity even when total clashes decrease', () => {
    const { input } = harness();
    input.detail.mockReturnValue(detail([2]));
    const r = searchPhysicalDesignAlternatives(input);
    expect(r.changed).toEqual([]);
    expect(r.outcomes).toBe(input.outcomes);
    expect(r.result).toBe(input.result);
    expect(r.trials.every(t => t.newlyBlocked?.includes(2))).toBe(true);
  });

  it('does not accept fewer clashes achieved by dropping verification coverage', () => {
    const { input } = harness();
    const missing = detail([]);
    missing.reverification.pop();
    input.detail.mockReturnValue(missing);
    expect(searchPhysicalDesignAlternatives(input).changed).toEqual([]);
  });

  it('never drops a member after a nominal failure and bounds nominal search work', () => {
    const { input, generate } = harness();
    generate.mockReturnValue({ ...outcome(1, 12), outcome: 'SEARCH_EXHAUSTED', accepted: undefined, certificate: undefined });
    input.budget.maxMemberSearches = 1;
    const r = searchPhysicalDesignAlternatives(input);
    expect(r.termination).toBe('budget');
    expect(r.stats.memberSearches).toBe(1);
    expect(r.outcomes).toBe(input.outcomes);
    expect(input.detail).not.toHaveBeenCalled();
  });

  it.each(['locked', 'span', 'provisional'] as const)('does not redesign a %s member', mode => {
    const { input, generate } = harness();
    if (mode === 'span') input.contexts.get(1)!.span = { representative: 1, elements: [1, 2] } as never;
    if (mode === 'provisional') input.outcomes.set(1, { ...outcome(1, 10), outcome: 'PROVISIONAL_BIAXIAL' } as MemberDesignOutcome);
    const r = searchPhysicalDesignAlternatives({ ...input, lockedMembers: mode === 'locked' ? new Set([1]) : undefined });
    expect(generate).not.toHaveBeenCalled();
    expect(r.changed).toEqual([]);
  });

  it('passes updated neighboring seats into each full-detail verification without mutating contexts', () => {
    const { input } = harness();
    searchPhysicalDesignAlternatives(input);
    const contexts = input.detail.mock.calls[0][1];
    expect(contexts.get(2)!.beamCorners!.get(1)!.bottomDiasMm).toEqual([16]);
    expect(input.contexts.get(2)!.beamCorners).toBeUndefined();
  });

  it('uses the last accepted neighbor assignment for subsequent member searches', () => {
    const { input, generate } = harness();
    input.result = detail([1, 1, 2]);
    input.detail.mockReturnValueOnce(detail([2])).mockReturnValueOnce(detail([]));
    const r = searchPhysicalDesignAlternatives(input);
    expect(r.changed).toEqual([1, 2]);
    expect(generate.mock.calls[1][1].beamCorners!.get(1)!.bottomDiasMm).toEqual([16]);
  });

  it('respects the project catalog and a zero geometry budget', () => {
    const { input, generate } = harness();
    input.contexts.get(1)!.material.longDiametersMm = [12];
    searchPhysicalDesignAlternatives(input);
    expect(generate.mock.calls.map(c => c[1].material.longDiametersMm)).toEqual([[12]]);
    generate.mockClear();
    input.budget.maxDetailingRuns = 0;
    expect(searchPhysicalDesignAlternatives(input).termination).toBe('budget');
    expect(generate).not.toHaveBeenCalled();
  });

  it.each([NaN, Infinity, -1, 1.5])('rejects invalid operation budgets (%s)', value => {
    const { input } = harness();
    input.budget.maxMemberSearches = value;
    expect(() => searchPhysicalDesignAlternatives(input)).toThrow('non-negative integers');
  });

  it('reports physical work pending when a budget prevents repair despite passing section checks', () => {
    const { input } = harness();
    const r = runDesignFeedbackLoop({ ...input, detail: () => input.result,
      physicalSearch: { maxDetailingRuns: 0, maxMemberSearches: 0 },
    });
    expect(r.outcome).toBe('PHYSICAL_REPAIR_PENDING');
    expect(r.stats.truncated).toBe(true);
    expect(r.physicalSearch!.termination).toBe('budget');
    expect(r.outcomes).toBe(input.outcomes);
    expect(r.result).toBe(input.result);
  });
});
