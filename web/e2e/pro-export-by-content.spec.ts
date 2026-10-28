import { test, expect, loadModel } from './fixtures';

test.describe('@smoke exporting by what comes out', () => {
  test('four buttons by content; the format is chosen inside each dialog', async ({ pro: page }) => {
    await loadModel(page, 'mat-foundation');
    await page.getByTestId('pr-project').click();
    for (const id of ['pp-export-results', 'pp-export-report', 'pp-export-drawings', 'pp-export-ifc']) await expect(page.getByTestId(id)).toBeEnabled();
    for (const gone of ['project-workbook', 'pp-export-csv', 'pp-export-dxf', 'pp-export-svg', 'pp-export-png', 'pp-export-jpg']) await expect(page.getByTestId(gone)).toHaveCount(0);

    await page.getByTestId('pp-export-results').click();
    const data = page.getByTestId('pro-data-export-dialog');
    await expect(data.getByTestId('pro-data-format-xlsx')).toBeVisible();
    await expect(data.getByTestId('pro-data-format-csv')).toBeVisible();
    await page.keyboard.press('Escape');

    await page.getByTestId('pp-export-report').click();
    const report = page.getByTestId('pro-report-dialog');
    for (const f of ['pdf', 'docx', 'html']) await expect(report.getByTestId(`rpt-format-${f}`)).toBeVisible();
    await expect(report.getByTestId('rpt-format-xlsx')).toHaveCount(0);
    await report.getByTestId('rpt-format-docx').click();
    await expect(report.getByTestId('rpt-pdf')).toHaveText(/Word/);
    await page.keyboard.press('Escape');

    // Drawings: one row of formats, DXF and the pictures in turn, each keeping its own options.
    await page.getByTestId('pp-export-drawings').click();
    const dxf = page.getByTestId('pro-dxf-export-dialog');
    await expect(dxf.getByTestId('drawing-format-dxf')).toHaveClass(/on/);
    await dxf.locator('summary', { hasText: /Dibujo|Drawing/ }).click();
    await dxf.getByTestId('pro-dxf-units-mm').click();
    await dxf.getByTestId('drawing-format-png').click();
    await expect(dxf).toHaveCount(0);
    const view = page.getByTestId('pro-view-export-dialog');
    await expect(view.getByTestId('drawing-format-png')).toHaveClass(/on/);
    await view.getByTestId('drawing-format-dxf').click();
    await expect(view).toHaveCount(0);
    await expect(page.getByTestId('pro-dxf-export-dialog').getByTestId('pro-dxf-units-mm')).toHaveClass(/on/);
  });
});
