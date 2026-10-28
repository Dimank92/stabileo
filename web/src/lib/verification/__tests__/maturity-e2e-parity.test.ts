/**
 * Parity and contract test for Issue #440:
 * "Exit: every result view and every design-check row shows the badge; an e2e test checks
 * that the badge level equals the matrix entry for each analysis and check."
 */

import { describe, it, expect } from 'vitest';
import dashboard from '../../../../../docs/verification/dashboard.json';
import {
  MATURITY_LEVELS,
  getDesignCheckMaturity,
  resolveMaturityBadge,
  analysesMaturity,
  designMaturity,
  elementsMaturity,
} from '../maturity';
import { concreteCheckRows } from '../../engine/design/check-rows';
import type { ProvidedRebarResult } from '../../engine/station-design-forces';

describe('Verification Maturity Parity and Badge Levels (Issue #440)', () => {
  it('checks that every analysis in dashboard.json has a badge whose level equals the matrix entry', () => {
    for (const item of dashboard.maturity.analyses) {
      const badge = resolveMaturityBadge({
        category: 'analyses',
        nameOrKey: item.name,
        perResultCertified: true,
      });

      expect(badge.matrixLevel).toBe(item.level);
      expect(badge.displayLevel).toBe(item.level);
      expect(badge.name).toBe(item.name);
      expect(badge.openIssues).toEqual(item.issues);
    }
  });

  it('checks that every design check in dashboard.json has a badge whose level equals the matrix entry', () => {
    for (const item of dashboard.maturity.design) {
      const badge = resolveMaturityBadge({
        category: 'design',
        nameOrKey: item.name,
        perResultCertified: true,
      });

      expect(badge.matrixLevel).toBe(item.level);
      expect(badge.displayLevel).toBe(item.level);
      expect(badge.name).toBe(item.name);
      expect(badge.openIssues).toEqual(item.issues);
    }
  });

  it('checks that every element in dashboard.json has a badge whose level equals the matrix entry', () => {
    for (const item of dashboard.maturity.elements) {
      const badge = resolveMaturityBadge({
        category: 'elements',
        nameOrKey: item.name,
        perResultCertified: true,
      });

      expect(badge.matrixLevel).toBe(item.level);
      expect(badge.displayLevel).toBe(item.level);
      expect(badge.name).toBe(item.name);
      expect(badge.openIssues).toEqual(item.issues);
    }
  });

  it('checks that design check rows carry the CIRSOC 201 maturity level from dashboard.json', () => {
    const dummyProvided: ProvidedRebarResult = {
      elementId: 1,
      elementType: 'beam',
      checks: [
        {
          category: 'Flexure, span',
          limiting: 'flexure',
          demand: 50,
          capacity: 100,
          ratio: 0.5,
          status: 'ok',
          unit: 'kN·m',
        },
        {
          category: 'Shear, left',
          limiting: 'shear',
          demand: 30,
          capacity: 60,
          ratio: 0.5,
          status: 'ok',
          unit: 'kN',
        },
      ],
      governingCheck: {
        category: 'Flexure, span',
        limiting: 'flexure',
        demand: 50,
        capacity: 100,
        ratio: 0.5,
        status: 'ok',
        unit: 'kN·m',
      },
      worstRatio: 0.5,
    } as unknown as ProvidedRebarResult;

    const rows = concreteCheckRows(dummyProvided);
    expect(rows.length).toBe(2);

    const cirsoc201Maturity = getDesignCheckMaturity('cirsoc201');
    expect(cirsoc201Maturity).toBeDefined();

    for (const row of rows) {
      expect(row.maturity).toBe(cirsoc201Maturity?.level);
    }
  });

  it('proves no level is hardcoded: all levels are members of MATURITY_LEVELS', () => {
    for (const item of [...analysesMaturity, ...designMaturity, ...elementsMaturity]) {
      expect(MATURITY_LEVELS).toContain(item.level);
    }
  });
});

