import { describe, it, expect } from 'vitest';
import dashboard from '../../../../../docs/verification/dashboard.json';
import {
  MATURITY_LEVELS,
  getAnalysisMaturity,
  getDesignCheckMaturity,
  getElementMaturity,
  resolveMaturityBadge,
  allMaturityEntries,
} from '../maturity';

describe('Verification Maturity Matrix (docs/verification/dashboard.json)', () => {
  it('defines the five canonical maturity levels in precedence order', () => {
    expect(MATURITY_LEVELS).toEqual([
      'Certified',
      'Benchmarked',
      'Differential',
      'Tested',
      'Unverified',
    ]);
  });

  it('reads analysis maturity levels dynamically from dashboard.json without hardcoding', () => {
    const rawAnalyses = dashboard.maturity.analyses;
    expect(rawAnalyses.length).toBeGreaterThan(0);

    for (const item of rawAnalyses) {
      const resolved = getAnalysisMaturity(item.name);
      expect(resolved).toBeDefined();
      expect(resolved?.name).toBe(item.name);
      expect(resolved?.level).toBe(item.level);
      expect(resolved?.issues).toEqual(item.issues);
    }
  });

  it('reads design check maturity levels dynamically from dashboard.json', () => {
    const rawDesign = dashboard.maturity.design;
    expect(rawDesign.length).toBeGreaterThan(0);

    for (const item of rawDesign) {
      const resolved = getDesignCheckMaturity(item.name);
      expect(resolved).toBeDefined();
      expect(resolved?.name).toBe(item.name);
      expect(resolved?.level).toBe(item.level);
      expect(resolved?.issues).toEqual(item.issues);
    }
  });

  it('reads element maturity levels dynamically from dashboard.json', () => {
    const rawElements = dashboard.maturity.elements;
    expect(rawElements.length).toBeGreaterThan(0);

    for (const item of rawElements) {
      const resolved = getElementMaturity(item.name);
      expect(resolved).toBeDefined();
      expect(resolved?.name).toBe(item.name);
      expect(resolved?.level).toBe(item.level);
      expect(resolved?.issues).toEqual(item.issues);
    }
  });

  describe('resolveMaturityBadge rules', () => {
    it('shows Certified with pass styling ONLY when perResultCertified is true', () => {
      const badge = resolveMaturityBadge({
        category: 'analyses',
        nameOrKey: 'Linear static',
        perResultCertified: true,
      });

      expect(badge.matrixLevel).toBe('Certified');
      expect(badge.displayLevel).toBe('Certified');
      expect(badge.tone).toBe('ok');
      expect(badge.passStyling).toBe(true);
    });

    it('removes Certified and NEVER gives pass styling when per-result certificate is not computed or failed', () => {
      const uncertifiedBadge = resolveMaturityBadge({
        category: 'analyses',
        nameOrKey: 'Linear static',
        perResultCertified: false,
      });

      expect(uncertifiedBadge.matrixLevel).toBe('Certified');
      expect(uncertifiedBadge.displayLevel).not.toBe('Certified');
      expect(uncertifiedBadge.displayLevel.toLowerCase()).not.toContain('certified');
      expect(uncertifiedBadge.tone).not.toBe('ok');
      expect(uncertifiedBadge.passStyling).toBe(false);
      // For Linear static, next evidence is Benchmarked
      expect(uncertifiedBadge.displayLevel).toBe('Benchmarked');
    });

    it('reports open safety issues with links to GitHub', () => {
      const badge = resolveMaturityBadge({
        category: 'analyses',
        nameOrKey: 'Linear static',
        perResultCertified: true,
      });

      expect(badge.openIssues.length).toBeGreaterThan(0);
      expect(badge.hasOpenIssues).toBe(true);
      expect(badge.issueLinks[0]).toMatch(/^https:\/\/github\.com\/lambdaclass\/stabileo\/issues\/\d+$/);
    });

    it('marks steel results as experimental until issue #389 is closed', () => {
      const steelBadge = resolveMaturityBadge({
        category: 'design',
        nameOrKey: 'CIRSOC 301 steel, engine (`cirsoc301_check`)',
        perResultCertified: false,
      });

      expect(steelBadge.isExperimental).toBe(true);
      expect(steelBadge.openIssues).toContain(389);
    });
  });

  it('guarantees 100% of matrix entries in dashboard.json are covered without drift', () => {
    const all = allMaturityEntries();
    const totalExpected =
      dashboard.maturity.analyses.length +
      dashboard.maturity.elements.length +
      dashboard.maturity.design.length;

    expect(all.length).toBe(totalExpected);

    for (const entry of all) {
      expect(MATURITY_LEVELS).toContain(entry.level);
    }
  });
});

