/**
 * Several floor plans in one drawing (`floor-regions.ts`): found in reading order, and laid on
 * each other by their columns, wherever on the sheet each was drawn.
 */
import { describe, it, expect } from 'vitest';
import { parseCadDxf } from '../parse';
import { extractArchPlan, suggestLayerMappings } from '../classify';
import { cropDoc } from '../infer';
import { alignPlans, detectPlanWindows, shiftDoc } from '../floor-regions';
import { multiPlanDxf } from './dxf-fixture';

const doc = parseCadDxf(multiPlanDxf(), 'plantas.dxf');
const mappings = suggestLayerMappings(doc, 'm');

describe('floor plans drawn side by side', () => {
  it('finds the three plans left to right, and leaves the detail and the title block out', () => {
    const wins = detectPlanWindows(doc, 'm', mappings);
    expect(wins).toHaveLength(3);
    expect(wins[0]!.x0).toBeLessThan(0); expect(wins[0]!.x1).toBeGreaterThan(10);
    expect(wins[1]!.x0).toBeGreaterThan(20); expect(wins[1]!.x1).toBeLessThan(45);
    expect(wins[2]!.x0).toBeGreaterThan(50); expect(wins[2]!.x1).toBeLessThan(70);
    // Each window keeps its whole plan.
    const plans = wins.map((w) => extractArchPlan(cropDoc(doc, w), mappings, 'm'));
    expect(plans.map((p) => p.columns.length)).toEqual([6, 6, 4]);
  });

  it('lays each plan on the one below by its columns', () => {
    const plans = detectPlanWindows(doc, 'm', mappings).map((w) => extractArchPlan(cropDoc(doc, w), mappings, 'm'));
    const al = alignPlans(plans);
    expect(al.map((a) => a.method)).toEqual(['base', 'columns', 'columns']);
    expect(al[0]!.ref.x).toBeCloseTo(0, 6); expect(al[0]!.ref.y).toBeCloseTo(0, 6);
    expect(al[1]!.ref.x).toBeCloseTo(30, 6); expect(al[1]!.ref.y).toBeCloseTo(0, 6);
    expect(al[1]!.matched).toBe(6);
    expect(al[2]!.ref.x).toBeCloseTo(60, 6); expect(al[2]!.ref.y).toBeCloseTo(1, 6);
    expect(al[2]!.matched).toBe(4);
  });

  it('keeps a point chosen by hand and lays the plans above on it', () => {
    const plans = detectPlanWindows(doc, 'm', mappings).map((w) => extractArchPlan(cropDoc(doc, w), mappings, 'm'));
    const al = alignPlans(plans, [null, { x: 35, y: 0 }]);
    expect(al[1]).toEqual({ ref: { x: 35, y: 0 }, method: 'manual', matched: 0 });
    // The top floor goes onto the second as that one was laid: 5 m to the right of it.
    expect(al[2]!.ref.x).toBeCloseTo(65, 6);
  });

  it('lays a plan with no matching columns by its lower-left corner', () => {
    const wins = detectPlanWindows(doc, 'm', mappings);
    const lower = extractArchPlan(cropDoc(doc, wins[0]!), mappings, 'm');
    const beamsOnly = { ...extractArchPlan(cropDoc(doc, wins[2]!), mappings, 'm'), columns: [] };
    const al = alignPlans([lower, beamsOnly]);
    expect(al[1]!.method).toBe('columns'); // its beams' ends still match the lower floor's
    const far = { ...beamsOnly, beams: beamsOnly.beams.map((b) => ({ ...b, a: { x: b.a.x * 1.37, y: b.a.y * 1.37 }, b: { x: b.b.x * 1.37, y: b.b.y * 1.37 } })) };
    const al2 = alignPlans([lower, far]);
    expect(al2[1]!.method).toBe('corner');
    expect(al2[1]!.ref.x).toBeCloseTo(60 * 1.37, 6); expect(al2[1]!.ref.y).toBeCloseTo(1 * 1.37, 6);
  });

  it('moves a drawing by a translation', () => {
    const moved = shiftDoc(doc, -30, 0);
    expect(moved.bbox!.minX).toBeCloseTo(doc.bbox!.minX - 30, 9);
    const p = extractArchPlan(cropDoc(moved, { x0: -1, x1: 11, y0: -1, y1: 5 }), mappings, 'm');
    expect(p.columns).toHaveLength(6);
  });
});
