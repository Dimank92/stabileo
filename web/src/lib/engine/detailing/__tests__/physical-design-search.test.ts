import { afterEach, describe, expect, it, vi } from 'vitest';
import * as design from '../../design/candidate-search';
import { cirsoc201Adapter } from '../../design/adapters/cirsoc201-adapter';
import type { MemberContext } from '../../design/member-context';
import type { MemberDesignOutcome } from '../../design/outcome';
import type { RunDetailingResult } from '../run-detailing';
import { detailRegresses, runDesignFeedbackLoop } from '../design-feedback-loop';
import { searchPhysicalDesignAlternatives } from '../physical-design-search';
import { syntheticBeamContext } from '../../design/__tests__/helpers';
import { rebarHash } from '../../design/rebar-hash';

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

describe('atomic top continuity repair', () => {
  function paired() {
    const { input } = harness();
    for (const id of [1, 2]) input.contexts.set(id, syntheticBeamContext({ elementId: id }));
    input.outcomes.set(1, outcome(1, 12)); input.outcomes.set(2, outcome(2, 16));
    input.result = detail([1, 2]);
    input.result.assemblies[0].unsupported = [{ key: 'detailing.lapCheck.unarranged',
      scope: { elementIds: [1, 2] }, params: { joint: 'n2' }, message: 'overlap', refs: [] }];
    input.result.lapping = { laps: [{ jointId: 'n2', fromElementId: 1, toElementId: 2 }] as never, fused: 0, unmaterialised: [] };
    const verify = vi.spyOn(input.adapter, 'verify').mockReturnValue({
      overallStatus: 'ok', strengthCheckCount: 2, checks: [{ status: 'ok' }], worstUtilization: 0.8, checkedAxes: ['My'],
    } as never);
    return { input: { ...input, budget: { maxDetailingRuns: 2, maxMemberSearches: 4, operators: ['harmonizeTop'] as const } }, verify };
  }

  it('checks both proposed members against their new neighbours and publishes the pair only after full detail', () => {
    const { input, verify } = paired();
    const result = searchPhysicalDesignAlternatives(input);
    expect(verify).toHaveBeenCalledTimes(2);
    expect(result.trials[0]).toMatchObject({ elementIds: [1, 2], jointId: 'n2', disposition: 'accepted' });
    expect(input.detail).toHaveBeenCalledTimes(1);
    const proposed = input.detail.mock.calls[0][0];
    for (const id of [1, 2]) {
      const r = proposed.get(id)!;
      expect(r.certificate!.rebarHash).toBe(rebarHash(r.accepted));
      expect(r.accepted!.regions!.topStart!.diameter).toBe(12);
    }
    // Top seats also account for the unchanged bottom hooks (Ø16).
    expect(verify.mock.calls[0][0].beamCorners!.get(2)!.topDiasMm).toEqual([12, 16]);
    expect(input.outcomes.get(2)!.accepted!.regions!.topStart!.diameter).toBe(16);
    expect(result.stats).toMatchObject({ detailingRuns: 1, memberSearches: 2, verifierCalls: 2, structuralSolves: 0 });
  });

  it('rejects the whole pair if either nominal check fails', () => {
    const { input, verify } = paired();
    verify.mockReturnValue({ strengthCheckCount: 2, checks: [{ status: 'fail' }], worstUtilization: 1.2 } as never);
    const result = searchPhysicalDesignAlternatives(input);
    expect(input.detail).not.toHaveBeenCalled();
    expect(result.outcomes).toBe(input.outcomes);
    expect(result.trials.every(t => t.nominalFailures?.length === 2)).toBe(true);
  });

  it('does not certify a warning that requires a hook the proposal does not provide', () => {
    const { input, verify } = paired();
    verify.mockReturnValue({ overallStatus: 'fail', strengthCheckCount: 2,
      checks: [{ status: 'warn', anchorHook: { face: 'bottom', toward: 'start', group: 0 } }],
      worstUtilization: 0.8, checkedAxes: ['My'] } as never);
    expect(searchPhysicalDesignAlternatives(input).changed).toEqual([]);
    expect(input.detail).not.toHaveBeenCalled();
  });

  it.each(['locked', 'provisional', 'budget'] as const)('does not partly replace a pair when %s', mode => {
    const { input, verify } = paired();
    if (mode === 'provisional') input.outcomes.set(2, { ...input.outcomes.get(2)!, outcome: 'PROVISIONAL_BIAXIAL' });
    if (mode === 'budget') input.budget.maxMemberSearches = 1;
    const result = searchPhysicalDesignAlternatives({ ...input, lockedMembers: mode === 'locked' ? new Set([2]) : undefined });
    expect(verify).not.toHaveBeenCalled();
    expect(result.outcomes).toBe(input.outcomes);
    expect(result.changed).toEqual([]);
    if (mode === 'budget') expect(result.termination).toBe('budget');
  });

  it('rejects a nominally valid pair if the final detail regresses', () => {
    const { input } = paired();
    input.detail.mockReturnValue(detail([], [2]));
    const result = searchPhysicalDesignAlternatives(input);
    expect(result.changed).toEqual([]);
    expect(result.result).toBe(input.result);
    expect(result.trials.map(t => t.disposition)).toEqual(['regression', 'regression']);
  });

  it('requires improvement of the target splice, not just fewer other clashes', () => {
    const { input } = paired();
    const fewer = detail([]);
    fewer.assemblies[0].unsupported = input.result.assemblies[0].unsupported;
    input.detail.mockReturnValue(fewer);
    expect(searchPhysicalDesignAlternatives(input).trials.map(t => t.disposition)).toEqual(['noProgress', 'noProgress']);
  });

  it('shares operation budgets across paired and single-member operators', () => {
    const { input } = paired();
    input.detail.mockReturnValueOnce(detail([1])).mockReturnValue(detail([]));
    const result = searchPhysicalDesignAlternatives({ ...input,
      budget: { maxDetailingRuns: 2, maxMemberSearches: 3, operators: ['harmonizeTop', 'diameterFamily'] } });
    expect(result.trials.map(t => t.operator)).toEqual(['harmonizeTop', 'diameterFamily']);
    expect(result.trials.map(t => t.disposition)).toEqual(['accepted', 'accepted']);
    expect(result.stats).toMatchObject({ detailingRuns: 2, memberSearches: 3, verifierCalls: 3 });
    expect(result.changed).toEqual([1, 2]);
  });
});

describe('guided physical alternatives', () => {
  it.each([false, true])('reverifies a layer redistribution and retains it only if the full detail improves (nominal failure: %s)', nominalFailure => {
    const { input, generate } = harness();
    input.contexts.set(1, syntheticBeamContext());
    const current = outcome(1, 16);
    current.accepted!.regions!.bottomSpan = { count: 6, diameter: 16 };
    current.accepted!.regions!.bottomSpanLayers = [
      { count: 4, row: 0, diameter: 16 }, { count: 2, row: 1, diameter: 16 },
    ];
    input.outcomes.set(1, current);
    const verify = vi.spyOn(input.adapter, 'verify').mockReturnValue({
      strengthCheckCount: 2, checks: [{ status: nominalFailure ? 'fail' : 'ok' }],
      worstUtilization: nominalFailure ? 1.1 : 0.9, checkedAxes: ['My'],
    } as never);
    const r = searchPhysicalDesignAlternatives({ ...input,
      budget: { maxDetailingRuns: 1, maxMemberSearches: 1, operators: ['redistributeLayers'] },
    });
    expect(generate).not.toHaveBeenCalled();
    expect(verify).toHaveBeenCalledTimes(1);
    expect(r.stats.candidatesConsidered).toBe(1);
    expect(r.stats.verifierCalls).toBe(1);
    expect(r.trials[0].operator).toBe('redistributeLayers');
    if (nominalFailure) {
      expect(input.detail).not.toHaveBeenCalled();
      expect(r.outcomes).toBe(input.outcomes);
    } else {
      expect(r.changed).toEqual([1]);
      const accepted = r.outcomes.get(1)!;
      expect(accepted.accepted!.regions!.bottomSpanLayers!.map(l => l.count)).toEqual([3, 3]);
      expect(accepted.certificate!.rebarHash).toBe(rebarHash(accepted.accepted));
      expect(accepted.certificate!.rebarHash).not.toBe(rebarHash(current.accepted));
    }
  });
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
    const r = runDesignFeedbackLoop({ ...input, budget: undefined, detail: () => input.result,
      physicalSearch: { maxDetailingRuns: 0, maxMemberSearches: 0 },
    });
    expect(r.outcome).toBe('PHYSICAL_REPAIR_PENDING');
    expect(r.stats.truncated).toBe(true);
    expect(r.physicalSearch!.termination).toBe('budget');
    expect(r.outcomes).toBe(input.outcomes);
    expect(r.result).toBe(input.result);
  });
});
