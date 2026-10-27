/**
 * Rebuild the PRO report's slab-steel section from the slab design results (#373).
 *
 * Acceptance criteria:
 * - Slab-steel section restored, sourced only from the slab design's own results (load, span, f'c, As).
 * - Test asserts every value in the section traces to the design result object.
 * - Section omitted (with a note) when no slab design exists.
 */
import { describe, it, expect } from 'vitest';
import { generateReportHtml, type ReportData } from '../pro-report';
import { designSlabPanel, type SlabPanelInput, type SlabDesignResult } from '../detailing/slab-design';
import en from '../../i18n/locales/en';

function mockReportData(extra: Partial<ReportData> = {}): ReportData {
  return {
    projectName: 'Test Project',
    date: '2026-10-28',
    nodes: [],
    elements: [],
    materials: [],
    sections: [],
    supports: [],
    loadCount: 0,
    results: { displacements: [], reactions: [], elementForces: [] },
    t: (k: string) => (en as Record<string, string>)[k] ?? k,
    ...extra,
  } as ReportData;
}

const samplePanelInput: SlabPanelInput = {
  panelId: 'Panel-North-101',
  lx: 4.80,
  ly: 6.20,
  thickness: 0.15,
  cover: 0.025,
  supportedSides: 4,
  fc: 28,
  fy: 420,
  maxAggregateSizeMm: 20,
  edition: '2025',
  moments: { mx: 18.5, my: 12.3, mxy: 4.2 },
  qu: 14.75, // Factored area load (kPa)
};

describe('PRO report slab steel section (#373)', () => {
  it('restores slab-steel section sourced directly from slab design results (load, span, fc, As)', () => {
    const designResult: SlabDesignResult = designSlabPanel(samplePanelInput);

    const reportData = mockReportData({
      slabDesigns: [designResult],
    });

    const html = generateReportHtml(reportData);

    // Section title
    expect(html).toContain('Slab Reinforcement');

    // Sourced values tracing strictly to designResult / samplePanelInput
    expect(html).toContain(designResult.panelId); // 'Panel-North-101'
    expect(html).toContain('4.80'); // span lx
    expect(html).toContain('6.20'); // span ly
    expect(html).toContain('28'); // fc = 28 MPa
    expect(html).toContain('14.75'); // qu load = 14.75 kPa
    expect(html).toContain('15'); // thickness 15 cm

    // Reinforcement layer checks from designResult.layers
    for (const layer of designResult.layers) {
      expect(html).toContain(`Ø${layer.diameterMm}`);
      expect(html).toContain((layer.asProvided * 1e4).toFixed(2));
      expect(html).toContain((layer.asRequired * 1e4).toFixed(2));
    }

    // Shear check from designResult.shear
    expect(html).toContain(designResult.shear.vu.toFixed(1));
    expect(html).toContain(designResult.shear.phiVc.toFixed(1));
  });

  it('omits the slab-steel detail section with a note when no slab design exists', () => {
    const reportData = mockReportData({
      quads: [{ id: 1, nodes: [1, 2, 3, 4], sectionId: 1, materialId: 1 } as never],
      slabDesigns: [],
    });

    const html = generateReportHtml(reportData);

    expect(html).toContain('Slab Reinforcement');
    expect(html).toMatch(/No slab design|not designed in this report|Run slab design/i);
    expect(html).not.toContain('Panel-North-101');
  });
});
