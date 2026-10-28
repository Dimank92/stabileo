/**
 * Test for Issue #389: Keep steel results labelled experimental until independent review of CIRSOC 301.
 * Acceptance criteria:
 * - Steel results show 'Proven bound' and no pass styling.
 * - Test asserts steel results never render with pass styling while the flag is set.
 * - Label removed only after a recorded independent review of the CIRSOC 301 checks.
 */
import { describe, it, expect } from 'vitest';
import {
  STEEL_RESULTS_EXPERIMENTAL,
  CIRSOC301_INDEPENDENT_REVIEW,
  steelAllowsPassStyling,
  steelCountsAsVerified,
  steelDisplayTone,
  type SteelMemberStatus,
} from '../steel-status';
import { steelMemberStatus } from '../steel-utilization';
import type { SteelVerification } from '../../codes/argentina/cirsoc301';

describe('Issue #389: Keep steel results labelled experimental until independent review of CIRSOC 301', () => {
  it('has STEEL_RESULTS_EXPERIMENTAL set to true by default', () => {
    expect(STEEL_RESULTS_EXPERIMENTAL).toBe(true);
  });

  it('has no independent review recorded by default', () => {
    expect(CIRSOC301_INDEPENDENT_REVIEW).toBeNull();
  });

  it('forbids pass styling while experimental flag is set or review is missing', () => {
    expect(steelAllowsPassStyling()).toBe(false);
  });

  it('steelMemberStatus never returns ok while experimental flag is set', () => {
    const mockVerif = {
      overallStatus: 'ok',
      engine: { available: true },
    } as unknown as SteelVerification;

    const status = steelMemberStatus(mockVerif);
    expect(status).not.toBe('ok');
    expect(status).toBe('warn');
  });

  it('steelDisplayTone never returns an ok or pass tone', () => {
    for (const status of ['EXPERIMENTAL', 'DEMAND_UNAVAILABLE', 'NOT_DESIGNED', 'NOT_APPLICABLE'] as SteelMemberStatus[]) {
      const tone = steelDisplayTone(status);
      expect(tone).not.toBe('ok');
      expect(['warn', 'neutral', 'info']).toContain(tone);
    }
  });

  it('steelCountsAsVerified returns literal false', () => {
    expect(steelCountsAsVerified('EXPERIMENTAL')).toBe(false);
  });
});
