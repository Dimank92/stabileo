/**
 * The verification report's bar marks count every stirrup leg, and its table speaks the reader's
 * language. Both were wrong: the legs were read and dropped (a 4-leg stirrup scheduled as a 2-leg
 * hoop, its steel weight short), and the table's labels named keys that do not exist
 * (`report.bm*`), so it printed the keys.
 */
import { describe, it, expect } from 'vitest';
import { computeBarMarks } from '../bar-marks';
import { barMarksTableHtml } from '../pro-report';
import es from '../../i18n/locales/es';
import type { ElementVerification } from '../codes/argentina/cirsoc201';

function beam(id: number, legs: number): ElementVerification {
  return {
    elementId: id, elementType: 'beam', b: 0.3, h: 0.6, cover: 0.03,
    flexure: { bars: '4Ø16', barDia: 16, barCount: 4 },
    shear: { stirrupDia: 8, spacing: 0.15, stirrupLegs: legs },
  } as unknown as ElementVerification;
}

describe('bar marks: stirrup legs', () => {
  const lengths = new Map([[1, 6], [2, 6]]);

  it('a 2-leg stirrup is one closed hoop and nothing more', () => {
    const marks = computeBarMarks([beam(1, 2)], lengths);
    expect(marks.filter((m) => m.shape === 'crosstie')).toEqual([]);
  });

  it('a 4-leg stirrup adds two crossties per hoop, across the depth', () => {
    const marks = computeBarMarks([beam(1, 4)], lengths);
    const hoops = marks.find((m) => m.shape === 'stirrup')!;
    const ties = marks.find((m) => m.shape === 'crosstie')!;
    expect(ties.count).toBe(hoops.count * 2);
    const hook = 6 * 8 / 1000;
    expect(ties.cuttingLength).toBeCloseTo(0.6 - 2 * 0.03 + 2 * hook, 2);
    expect(ties.weight).toBeGreaterThan(0);
  });

  it('beams with 2-leg and 4-leg stirrups of the same bar are two groups, not one', () => {
    const marks = computeBarMarks([beam(1, 2), beam(2, 4)], lengths);
    expect(marks.filter((m) => m.shape === 'stirrup')).toHaveLength(2);
  });
});

describe('the bar-marks table is in the reader’s language', () => {
  it('names no missing key: Spanish, crossties included', () => {
    const t = (k: string) => (es as Record<string, string>)[k] ?? k;
    const html = barMarksTableHtml(computeBarMarks([beam(1, 4)], new Map([[1, 6]])), t);
    expect(html).not.toMatch(/report\.|pro\.bm/);
    expect(html).toContain('Traba');
    expect(html).toContain('Estribo');
  });
});
