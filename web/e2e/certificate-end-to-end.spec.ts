import { test, expect } from './fixtures';

// The end-to-end certificate (docs/verification/CERTIFY.md, "End-to-end certificates") must
// reach the results panel through the real certify path (worker or main-thread fallback),
// not only through direct engine calls in the Rust tests: a solved frame shows the
// end-to-end line under the certificate.
test.describe('@smoke end-to-end certificate', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/app/basic?e2e=1');
    await page.waitForFunction(() => !!window.__stabileoActions, null, { timeout: 60_000 });
  });

  // 2D and 3D examples: both show the certificate status line and the end-to-end check
  // in the results panel once solved.
  for (const example of ['portal-frame', 'simply-supported', '3d-portal-frame', '3d-cantilever-load']) {
    test(`a solved ${example} shows the end-to-end check`, async ({ page }) => {
      const errors: string[] = [];
      page.on('pageerror', (e) => errors.push(e.message));
      await page.evaluate((name) => window.__stabileoActions.loadExample(name), example);
      await page.evaluate(() => window.__stabileoActions.solve());
      // The results panel (where the certificate is shown) opens with a diagram.
      await page.getByTestId('rb-cmd-deformed').first().click();
      await expect(page.getByTestId('certificate-status')).toContainText('Certified', { timeout: 60_000 });
      const line = page.getByTestId('certificate-e2e');
      await expect(line).toBeVisible({ timeout: 60_000 });
      await expect(line).toContainText('Checked end to end');
      await expect(line).toContainText('independent re-assembly');
      expect(errors).toEqual([]);
    });
  }
});
