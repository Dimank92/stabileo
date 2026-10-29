import { test, expect } from '@playwright/test';

test.describe('@smoke DaDeppo arch first version', () => {
  for (const width of [360, 390, 810]) {
    test(`replay and live production result at ${width}px`, async ({ page }, testInfo) => {
      test.setTimeout(180_000);
      await page.setViewportSize({ width, height: 900 });
      await page.emulateMedia({ reducedMotion: 'reduce' });
      await page.goto('/en/research/benchmarks/dadeppo-arch-1975/');
      await expect(page.getByText('First version · 0 reviews').first()).toBeVisible();
      await expect(page.locator('.arch-app')).toHaveAttribute('data-source', 'replay');
      await expect(page.getByTestId('arch-peak')).toHaveText('9.01129');
      await expect(page.locator('.sfv canvas').first()).toBeVisible();
      if (width === 810) await page.locator('.sfv').screenshot({ path: testInfo.outputPath('arch-viewer.png') });
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
      await page.getByRole('button', { name: 'Recompute live', exact: true }).click();
      await expect(page.locator('.arch-app')).toHaveAttribute('data-busy', 'true');
      await expect(page.locator('.arch-app')).toHaveAttribute('data-source', 'cached', { timeout: 150_000 });
      await expect(page.getByTestId('arch-peak')).toHaveText('9.01129');
      await expect(page.getByTestId('arch-verdict')).toContainText('descending branch reached');
      await expect(page.getByRole('alert')).toHaveCount(0);
      await page.getByRole('button', { name: 'Restart', exact: true }).click();
      await expect(page.locator('.arch-app')).toHaveAttribute('data-source', 'replay');
      await page.getByRole('button', { name: 'Stop', exact: true }).click();
    });
  }
});
