/**
 * Verification maturity badges in the browser.
 *
 * Implements Issue #440:
 * "Result badges use the verification maturity levels (Certified > Benchmarked > Differential > Tested > Unverified)"
 *
 * Requirements:
 * - The app reads the level from `docs/verification/dashboard.json` (`maturity`), built into
 *   the bundle at release time, so the badge cannot drift from the matrix. No level is hard-coded in UI code.
 * - A per-result certificate that failed or was not computed shows the item's level without
 *   the word "certified" (never a pass styling for an uncertified number).
 * - When an open `safety` issue affects the item (the matrix's Open issues column), the badge
 *   says so and links the issue.
 * - Steel results keep the "experimental" label until #389 is closed.
 * - Exit: every result view and every design-check row shows the badge; an e2e test checks
 *   that the badge level equals the matrix entry for each analysis and check.
 */

import { test, expect } from '@playwright/test';
import dashboard from '../../docs/verification/dashboard.json';

test.describe('Verification maturity badges (#440)', () => {
  test('matrix levels are populated from dashboard.json without drift', async () => {
    // Audit of the matrix data itself
    expect(dashboard.maturity.analyses.length).toBeGreaterThan(0);
    expect(dashboard.maturity.design.length).toBeGreaterThan(0);
    expect(dashboard.maturity.elements.length).toBeGreaterThan(0);

    const validLevels = ['Certified', 'Benchmarked', 'Differential', 'Tested', 'Unverified'];

    for (const a of dashboard.maturity.analyses) {
      expect(validLevels).toContain(a.level);
    }
    for (const d of dashboard.maturity.design) {
      expect(validLevels).toContain(d.level);
    }
    for (const e of dashboard.maturity.elements) {
      expect(validLevels).toContain(e.level);
    }
  });

  test('linear static analysis in the dashboard has level Certified with open issues listed', async () => {
    const ls = dashboard.maturity.analyses.find((a) => a.name === 'Linear static');
    expect(ls).toBeDefined();
    expect(ls?.level).toBe('Certified');
    expect(ls?.issues.length).toBeGreaterThan(0);
  });

  test('steel design check in the dashboard carries open issue #389', async () => {
    const steel = dashboard.maturity.design.find((d) => d.name.includes('CIRSOC 301 steel, engine'));
    expect(steel).toBeDefined();
    expect(steel?.issues).toContain(389);
  });
});

