// Render the gallery thumbnail of the spring-network experiment from our own run: the page at
// eta = 0.5 (auxetic foam), the shared Stabileo viewer's canvas, 640 x 400, WebP.
//   node scripts/research-thumbs-grossman.mjs <site base URL> <out.webp>
// Run against a built site (vite preview); Playwright's Chromium does the rendering.
import { chromium } from 'playwright';

const [base, out] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 1 });
await page.goto(`${base}/en/research/metamaterials/grossman-2026/?eta=0.5&thumb`, { waitUntil: 'networkidle' });
await page.waitForFunction(() => Number(document.querySelector('.sp-viewer')?.getAttribute('data-nodes')) > 500);
await page.waitForTimeout(1500);
// Only the model: hide the viewer's overlays (legend, buttons, axis triad, status bar).
await page.evaluate(() => {
  const root = document.querySelector('.sp-viewer');
  const main = root.querySelector('canvas');
  for (const el of root.querySelectorAll('*')) {
    if (el !== main && !el.contains(main)) el.style.visibility = 'hidden';
  }
});
await page.waitForTimeout(300);
const canvas = page.locator('.sp-viewer canvas').first();
const box = await canvas.boundingBox();
if (!box) throw new Error('viewer canvas not found');
const w = Math.min(box.width, (box.height * 640) / 400);
const h = (w * 400) / 640;
const png = await page.screenshot({ clip: { x: box.x + (box.width - w) / 2, y: box.y + (box.height - h) / 2, width: w, height: h } });
// Encode to WebP at 640 x 400 in the browser itself (no image dependency).
const webp = await page.evaluate(async (b64) => {
  const img = new Image();
  img.src = `data:image/png;base64,${b64}`;
  await img.decode();
  const c = document.createElement('canvas');
  c.width = 640;
  c.height = 400;
  c.getContext('2d').drawImage(img, 0, 0, 640, 400);
  return c.toDataURL('image/webp', 0.86).split(',')[1];
}, png.toString('base64'));
const { writeFileSync } = await import('node:fs');
writeFileSync(out, Buffer.from(webp, 'base64'));
console.log(`thumbnail: ${out}`);
await browser.close();
