import { test, expect } from '@playwright/test';

/**
 * Stabileo Research, spring networks (/research/metamaterials/grossman-2026): the network solves
 * in the browser and shows in the shared viewer; "Edit in Stabileo" loads the real editor only
 * when opened, and the production engine's bar forces equal the research solver's for the same
 * strip.
 *
 *   npx playwright test --grep "@smoke research springs"
 */
test.describe('@smoke research springs', () => {
  test('the network solves in the browser and the editor loads only on demand', async ({ page }) => {
    const requests: string[] = [];
    page.on('request', (r) => requests.push(r.url()));
    await page.goto('/en/research/metamaterials/grossman-2026/');
    await expect(page.getByRole('heading', { name: 'Auxetic spring networks' })).toBeVisible();
    await expect(page.getByText('Research, not a design tool')).toBeVisible();
    await expect.poll(async () => Number(await page.locator('.sp-viewer').getAttribute('data-nodes')), { timeout: 30_000 }).toBeGreaterThan(500);
    await expect(page.locator('.sp-viewer canvas').first()).toBeVisible();
    expect(requests.some((u) => u.endsWith('/research-wasm/grossman-2026/springs_bg.wasm'))).toBe(true);
    expect(requests.filter((u) => u.includes('/app/')), 'the editor is not loaded before its tab opens').toEqual([]);
  });

  test('the embedded editor solves the same strip with the production engine', async ({ page }) => {
    await page.goto('/en/research/metamaterials/grossman-2026/?e2e=1');
    await expect.poll(async () => Number(await page.locator('.sp-viewer').getAttribute('data-nodes')), { timeout: 30_000 }).toBeGreaterThan(500);
    await page.getByRole('tab', { name: 'Experiment' }).click(); // stops the sweep: the strip stays put
    await page.getByRole('tab', { name: 'Edit in Stabileo' }).click();
    await expect(page.locator('iframe[title="Stabileo editor (production engine)"]')).toBeVisible();
    const editor = () => page.frames().find((f) => f.url().includes('/app/basic'));
    await expect.poll(() => !!editor(), { timeout: 30_000 }).toBe(true);
    type Hooks = { memberAxialForces(): { id: number; n: number }[] | null };
    await expect
      .poll(async () => editor()!.evaluate(() => (window as unknown as { __stabileo?: Hooks }).__stabileo?.memberAxialForces()?.length ?? 0), { timeout: 60_000 })
      .toBeGreaterThan(500);
    const app = await editor()!.evaluate(() => (window as unknown as { __stabileo: Hooks }).__stabileo.memberAxialForces()!);
    const ours = await page.evaluate(() => (window as unknown as { __springsEditorStrip: { force: number[] } }).__springsEditorStrip.force);
    expect(app.length).toBe(ours.length);
    const scale = Math.max(...ours.map(Math.abs));
    // Bar forces in the editor are 1000 x ours (E·A/L = 1000 k kN/m).
    const worst = Math.max(...app.map((f) => Math.abs(f.n / 1000 - ours[f.id - 1])));
    expect(worst / scale).toBeLessThan(1e-6);
  });
});
