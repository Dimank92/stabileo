/**
 * A column standing inside a slab with no beam round it (a flat slab, a waffle slab on its drop
 * panels) is a vertex of the slab's mesh, so the slab rests on it: meshed, the region welds to the
 * column's node and the two share it. Before, the mesher placed fixed points on the boundary only,
 * and a column inside a region was left beside its slab, unconnected.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { modelStore } from '../../../store/model.svelte';
import { historyStore } from '../../../store/history.svelte';
import '../../../store/index';
import { generateMesh } from '../mesher';
import { applyMesh } from '../mesh-apply';

describe('fixed points inside a region', () => {
  beforeEach(() => { modelStore.clear(); historyStore.clear(); });

  it('the mesher makes an interior fixed point a vertex, the lattice giving way round it', () => {
    const outer = { kind: 'polygon' as const, points: [[0, 0, 3], [8, 0, 3], [8, 6, 3], [0, 6, 3]] as Array<[number, number, number]> };
    const plain = generateMesh({ outer, holes: [], size: 1, element: 'quad' })!;
    expect(plain.structured).toBe(true);
    const col: [number, number, number] = [3.3, 2.7, 3];
    const m = generateMesh({ outer, holes: [], size: 1, element: 'quad', fixedPoints: [col] })!;
    expect(m.structured).toBe(false);
    const hit = m.points.filter((p) => Math.hypot(p[0] - col[0], p[1] - col[1], p[2] - col[2]) < 1e-9);
    expect(hit).toHaveLength(1);
    // The point is used by cells, and no other point crowds it.
    const k = m.points.indexOf(hit[0]!);
    expect(m.cells.some((c) => c.includes(k))).toBe(true);
    const nearest = Math.min(...m.points.filter((_, i) => i !== k).map((p) => Math.hypot(p[0] - col[0], p[1] - col[1])));
    expect(nearest).toBeGreaterThan(0.2);
  });

  it("meshed onto the model, the slab shares the column's node", () => {
    const base = modelStore.addNode(3.3, 2.7, 0), top = modelStore.addNode(3.3, 2.7, 3);
    modelStore.addElement(base, top);
    const r = applyMesh({ outer: { kind: 'polygon', points: [[0, 0, 3], [8, 0, 3], [8, 6, 3], [0, 6, 3]] }, holes: [], size: 1, element: 'quad' }, { materialId: 1, thickness: 0.2, splitBeams: true })!;
    expect(r.nodeIds).toContain(top);
    expect([...modelStore.quads.values()].some((q) => q.nodes.includes(top))).toBe(true);
  });
});
