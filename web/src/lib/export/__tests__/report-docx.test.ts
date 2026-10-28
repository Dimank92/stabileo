import { describe, it, expect } from 'vitest';
import katex from 'katex';
import * as XLSX from 'xlsx';
import { strFromU8, unzipSync } from 'fflate';
import { htmlToDocxBlocks, texRuns, printColours } from '../report-docx';
import { docxBytes, type DocBlock } from '../docx';
import { htmlToBlocks, reportFile } from '../report-formats';

const PNG = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3]);
const stub = async (_src: string, size: { width?: number; height?: number }) => ({ png: PNG, width: size.width ?? 600, height: size.height ?? 400 });
const km = (tex: string) => katex.renderToString(tex, { throwOnError: false, output: 'htmlAndMathml' });

function wellFormed(xml: string): void {
  const stack: string[] = [];
  for (const m of xml.matchAll(/<(\/?)([A-Za-z][\w:.-]*)((?:\s+[\w:.-]+="[^"]*")*)\s*(\/?)>/g)) {
    if (m[1]) expect(stack.pop(), `closing ${m[2]}`).toBe(m[2]);
    else if (!m[4]) stack.push(m[2]!);
  }
  expect(stack).toEqual([]);
  expect(xml.replace(/<\?xml[^>]*\?>/, '').replace(/<(\/?)([A-Za-z][\w:.-]*)((?:\s+[\w:.-]+="[^"]*")*)\s*(\/?)>/g, '')).not.toMatch(/[<>]/);
}

const REPORT = `<!DOCTYPE html><html><head><title>x</title><style>.a{}</style></head><body>
<button class="print-btn no-print" onclick="window.print()">Imprimir</button>
<div class="cover-page"><img src="data:image/png;base64,AAAA" alt="Logo" style="max-height:80px;max-width:250px" />
<h1>Obra Sur</h1><div class="subtitle">Memoria de cálculo</div>
<div style="border:2px solid #b8860b">Borrador de CAD sin revisar</div><div class="date">2026-10-28</div></div>
<div class="page-break"></div>
<h2>1. Resultados</h2>
<p class="note">Valores de la combinación C1 (envolvente).</p>
<table><thead><tr><th>Nodo</th><th>${km('F_x')} (kN)</th><th>${km('\\sigma_{xx}')} (MPa)</th></tr></thead>
<tbody><tr><td>N1</td><td class="num">-12,5</td><td class="num">3,0</td></tr><tr><td>N2</td><td class="num">4,0</td><td class="num"></td></tr></tbody></table>
<figure><img class="screenshot" src="data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%2F%3E" alt="Momentos My" /><figcaption>Figura 1. Momentos My</figcaption></figure>
<p class="diag-warn">El nodo 7 está suelto.</p>
<section class="provenance"><h2>Procedencia</h2><p class="provenance-stale"><strong>Los resultados no corresponden al modelo actual</strong></p>
<table class="provenance-table"><tr><th>Estado</th><td><code>desactualizado</code></td></tr></table></section>
<script type="application/json" id="stabileo-provenance">{}</script>
</body></html>`;

describe('symbols written as TeX, read for Word', () => {
  it('keeps sub- and superscripts and writes greek and operators as characters', () => {
    expect(texRuns('F_x')).toEqual([{ text: 'F' }, { text: 'x', vert: 'sub' }]);
    expect(texRuns('\\sigma_{xx}')).toEqual([{ text: 'σ' }, { text: 'xx', vert: 'sub' }]);
    expect(texRuns("f'_c / f_y")).toEqual([{ text: 'f′' }, { text: 'c', vert: 'sub' }, { text: ' / f' }, { text: 'y', vert: 'sub' }]);
    expect(texRuns('A_{s,y}')).toEqual([{ text: 'A' }, { text: 's,y', vert: 'sub' }]);
    expect(texRuns('m^4')).toEqual([{ text: 'm' }, { text: '4', vert: 'sup' }]);
    expect(texRuns('\\frac{a+b}{2} \\leq \\gamma')).toEqual([{ text: '(a+b)/2 ≤ γ' }]);
  });
});

describe('a report read for Word', () => {
  it('keeps the cover with its logo, the page break, styled notes and warnings, figures with their caption, and number columns to the right', async () => {
    const { blocks, cover } = await htmlToDocxBlocks(REPORT, stub);
    expect(cover).toBe(true);
    const kinds = blocks.map((b) => b.kind);
    expect(kinds[0]).toBe('image');
    expect(blocks[0]).toMatchObject({ kind: 'image', alt: 'Logo', align: 'center', width: 250, height: 80 });
    expect(blocks[1]).toMatchObject({ kind: 'paragraph', text: 'Obra Sur', align: 'center' });
    expect(blocks.find((b) => b.kind === 'paragraph' && b.text.startsWith('Borrador'))).toMatchObject({ tone: 'warn', align: 'center' });
    const brk = kinds.indexOf('pageBreak');
    expect(brk).toBeGreaterThan(1);
    expect(blocks[brk + 1]).toMatchObject({ kind: 'heading', text: '1. Resultados', level: 2 });
    expect(blocks.find((b) => b.kind === 'paragraph' && b.text.startsWith('Valores'))).toMatchObject({ tone: 'note' });
    const t = blocks.find((b) => b.kind === 'table') as Extract<DocBlock, { kind: 'table' }>;
    expect(t.head).toEqual(['Nodo', 'Fx (kN)', 'σxx (MPa)']);
    expect(t.headRuns![1]).toEqual([{ text: 'F' }, { text: 'x', vert: 'sub' }, { text: ' (kN)' }]);
    expect(t.align).toEqual(['left', 'right', 'right']);
    expect(t.rows).toEqual([['N1', '-12,5', '3,0'], ['N2', '4,0', '']]);
    const fig = blocks.findIndex((b) => b.kind === 'image' && b.alt === 'Momentos My');
    expect(blocks[fig + 1]).toMatchObject({ kind: 'paragraph', caption: true, text: 'Figura 1. Momentos My' });
    expect(blocks.find((b) => b.kind === 'paragraph' && b.text.startsWith('El nodo 7'))).toMatchObject({ tone: 'warn' });
    expect(blocks.find((b) => b.kind === 'paragraph' && b.text.startsWith('Los resultados no'))).toMatchObject({ tone: 'error' });
    expect(blocks.some((b) => b.kind === 'paragraph' && b.text.includes('Imprimir'))).toBe(false);
  });

  it('writes a figure it cannot draw as its caption, never silently drops it', async () => {
    const { blocks } = await htmlToDocxBlocks(REPORT, undefined, '[Figura no incluida]');
    expect(blocks.filter((b) => b.kind === 'image')).toHaveLength(0);
    expect(blocks.find((b) => b.kind === 'paragraph' && b.text === '[Figura no incluida] Momentos My')).toMatchObject({ tone: 'note' });
  });

  it('recolours the dark interaction diagrams as the print style does', () => {
    expect(printColours('<rect fill="#1a2a40"/><line stroke="#4ecdc4"/>')).toBe('<rect fill="#f0f0f0"/><line stroke="#1a5090"/>');
  });

  it('the plain reading (workbook and CSV) does not read KaTeX twice', () => {
    const t = htmlToBlocks(REPORT).find((b) => b.kind === 'table') as Extract<DocBlock, { kind: 'table' }>;
    expect(t.head).toEqual(['Nodo', 'Fx (kN)', 'σxx (MPa)']);
  });
});

describe('the Word package', () => {
  it('holds every part it names: images, header and footer with page fields, the styles its paragraphs use', async () => {
    const f = (await reportFile('docx', { title: 'Obra Sur', html: REPORT, stem: 'informe', provenance: { title: 'Procedencia', lines: [['Estado', 'desactualizado']] }, docx: { header: 'Stabileo 0.3.0 · Obra Sur', footer: { left: 'Obra Sur · 2026-10-28', page: 'Página {page} de {pages}' }, rasterize: stub }, loadXlsx: async () => XLSX }))!;
    const z = unzipSync(f.bytes as Uint8Array);
    const doc = strFromU8(z['word/document.xml']!), rels = strFromU8(z['word/_rels/document.xml.rels']!), types = strFromU8(z['[Content_Types].xml']!), styles = strFromU8(z['word/styles.xml']!);
    for (const p of ['word/document.xml', 'word/styles.xml', 'word/header1.xml', 'word/footer1.xml', 'word/_rels/document.xml.rels', '[Content_Types].xml']) wellFormed(strFromU8(z[p]!));
    expect(Object.keys(z).filter((k) => k.startsWith('word/media/'))).toHaveLength(2);
    for (const id of doc.matchAll(/r:(?:embed|id)="([^"]+)"/g)) {
      const target = new RegExp(`Id="${id[1]}"[^>]*Target="([^"]+)"`).exec(rels)?.[1];
      expect(target, id[1]).toBeTruthy();
      expect(z[`word/${target}`], target).toBeTruthy();
    }
    expect(types).toContain('Extension="png"');
    expect(types).toContain('/word/header1.xml');
    for (const s of doc.matchAll(/<w:pStyle w:val="([^"]+)"\/>/g)) expect(styles, s[1]).toContain(`w:styleId="${s[1]}"`);
    const footer = strFromU8(z['word/footer1.xml']!);
    expect(footer).toContain('w:instr=" PAGE "');
    expect(footer).toContain('w:instr=" NUMPAGES "');
    expect(footer).toContain('Página ');
    expect(strFromU8(z['word/header1.xml']!)).toContain('Stabileo 0.3.0 · Obra Sur');
    expect(doc).toContain('<w:titlePg/>');
    expect(doc).not.toContain('w:val="Title"');
    expect(doc).toContain('<w:vertAlign w:val="subscript"/>');
    expect((doc.match(/Procedencia/g) ?? []).length).toBe(1);
    expect((doc.match(/<w:tbl>/g) ?? []).length).toBe(2);
    expect(doc).toContain('<w:jc w:val="right"/>');
  });

  it('keeps the plain blocks other writers pass: a title, headings, paragraphs and tables, and no header without one', () => {
    const z = unzipSync(docxBytes({ title: 'Listado', blocks: [{ kind: 'heading', text: 'Columnas', level: 1 }, { kind: 'paragraph', text: 'Nota', muted: true }, { kind: 'table', head: ['Columna', 'N'], rows: [['C1', '300']], align: ['left', 'right'] }] }));
    const doc = strFromU8(z['word/document.xml']!);
    wellFormed(doc);
    expect(doc).toContain('w:val="Title"');
    expect(doc).toContain('Listado');
    expect(z['word/header1.xml']).toBeUndefined();
    expect(doc).not.toContain('headerReference');
  });
});
