import { test, expect } from './fixtures';
import { multiPlanDxf } from '../src/lib/cad/__tests__/dxf-fixture';

test.use({ viewport: { width: 1440, height: 900 } });

/*
 * A DXF with the plans of three floors drawn side by side, 30 m apart: the wizard finds them,
 * gives each a floor, lays each on the one below by its columns, and the building comes out
 * stacked over the same footprint (not spread 60 m along x).
 */
test('@smoke floor plans drawn side by side come out stacked', async ({ pro: page }) => {
  await page.evaluate(() => window.dispatchEvent(new Event('stabileo-import-dxf')));
  await page.getByTestId('cad-file').setInputFiles({ name: 'plantas.dxf', mimeType: 'application/dxf', buffer: Buffer.from(multiPlanDxf()) });

  await page.getByTestId('cad-plans').locator('summary').click();
  await page.getByTestId('cad-detect-plans').click();
  await expect(page.getByTestId('cad-plan-row')).toHaveCount(3);
  const align = page.getByTestId('cad-plan-align');
  await expect(align.nth(1)).toContainText('6');
  await expect(align.nth(2)).toContainText('4');
  // The second plan's reference point is its own lower-left column, 30 m along.
  await expect(page.getByTestId('cad-plan-ref-x').nth(1)).toHaveValue('30');
  await page.screenshot({ path: test.info().outputPath('cad-plans.png') });

  await page.getByTestId('cad-next').click();
  await page.getByTestId('cad-next').click();
  await expect(page.getByTestId('cad-plans-summary')).toContainText('3');
  await page.getByTestId('cad-generate').click();
  await page.getByTestId('cad-apply').click();

  const pts = await page.evaluate(() => window.__stabileo.nodeIds().map((id) => window.__stabileo.nodePosition(id)!));
  expect(pts.length).toBeGreaterThan(0);
  const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
  expect(Math.min(...xs)).toBeGreaterThan(-0.05);
  expect(Math.max(...xs)).toBeLessThan(10.05);
  expect(Math.max(...ys)).toBeLessThan(4.05);
  // Three floors over the ground: the top one, smaller, still on the same columns.
  const levels = [...new Set(pts.map((p) => Math.round(p.z * 100) / 100))].sort((a, b) => a - b);
  expect(levels).toEqual([0, 3, 6, 9]);
  const top = pts.filter((p) => Math.abs(p.z - 9) < 1e-6);
  expect(Math.max(...top.map((p) => p.x))).toBeLessThan(5.05);
});
