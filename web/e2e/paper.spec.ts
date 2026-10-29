import { test, expect } from '@playwright/test';

/**
 * /paper: the JAIE 2026 paper in full, set as a paper (KaTeX), between Blog and Research in the
 * site header, with its PDF.
 *
 *   npx playwright test --grep "@smoke paper"
 */
test.describe('@smoke paper', () => {
  test('the header links Blog, Paper, Research in that order and Paper opens the paper', async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto('/es/blog/');
    const nav = page.locator('nav.site-header [data-nav]');
    const ids = await nav.evaluateAll((els) => els.map((e) => e.getAttribute('data-nav')));
    expect(ids.slice(ids.indexOf('blog'), ids.indexOf('blog') + 3)).toEqual(['blog', 'paper', 'research']);
    const link = page.locator('nav.site-header [data-nav="paper"]');
    await expect(link).toHaveText('Paper');
    await link.click();
    await expect(page).toHaveURL(/\/es\/paper\/$/);
    await expect(page.locator('[data-testid="paper"] h1')).toHaveText('Desarrollo de un agente estructural de cálculo sobre un solver verificado');
  });

  test('renders the whole paper: sections, equations, tables, figures and references', async ({ page }) => {
    await page.goto('/es/paper/');
    const paper = page.locator('[data-testid="paper"]');
    await expect(paper.locator('h2.pp-h1:not(.pp-unnumbered)')).toHaveCount(9);
    await expect(paper.locator('.pp-eq .katex-display')).toHaveCount(4);
    await expect(paper.locator('.pp-eq .katex-error, .katex-error')).toHaveCount(0);
    await expect(paper.locator('figure.pp-table table')).toHaveCount(4);
    await expect(paper.locator('ol.pp-refs > li')).toHaveCount(18);
    await expect(paper.locator('a[href="https://code-aster.org"]')).toHaveCount(1);
    const figures = paper.locator('figure.pp-figure img');
    await expect(figures).toHaveCount(4);
    for (const img of await figures.all()) {
      await img.scrollIntoViewIfNeeded();
      await expect.poll(() => img.evaluate((i: HTMLImageElement) => i.complete && i.naturalWidth > 0)).toBe(true);
    }
    const cite = paper.locator('sup.pp-cite a[href="#ref-6"]').first();
    await cite.click();
    await expect(page).toHaveURL(/#ref-6$/);
    const href = await page.getByTestId('paper-download').getAttribute('href');
    const pdf = await page.request.get(href!);
    expect(pdf.status()).toBe(200);
    expect((await pdf.body()).subarray(0, 5).toString()).toBe('%PDF-');
  });

  test('in English it keeps the Spanish text and says so', async ({ page }) => {
    await page.goto('/en/paper/');
    await expect(page.locator('[data-testid="paper"]')).toHaveAttribute('lang', 'es');
    await expect(page.getByText('The paper is written in Spanish; its abstract is in English.')).toBeVisible();
    await expect(page.getByTestId('paper-download')).toHaveText('Download PDF');
    await expect(page.locator('nav.site-header [data-nav="paper"][aria-current="page"]')).toHaveCount(1);
  });

  test('fits a phone without scrolling sideways', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 800 });
    await page.goto('/es/paper/');
    await expect(page.locator('[data-testid="paper"] .pp-eq').first()).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  });
});
