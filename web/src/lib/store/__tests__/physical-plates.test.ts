/**
 * A plate as a physical object, meshed on demand:
 *  - made from an outline and a hole, it holds no shell until it is meshed;
 *  - meshed, the hole has no shell and the plate holds its whole mesh;
 *  - meshed again (other size, same size), nothing accumulates and the area load defined on the
 *    plate gives the same resultant force and moment;
 *  - a beam lying across it is met and cut, and whole again when the mesh goes; a column through
 *    it is met where it crosses; a beam of another level over it in plan is left alone;
 *  - moving its outline or adding a beam in its plane puts the mesh out of date;
 *  - undo and redo take the creation and the mesh back and forth without orphans;
 *  - saved and opened, the plate keeps its outline, its mesh and its load.
 */
import { describe, it, expect, beforeEach, beforeAll } from 'vitest';
import * as wasmSolver from '../../engine/wasm-solver';
import { historyStore, modelStore, uiStore } from '..';
import {
  addPlateLoad, applyPlateMeshes, clearPlateMesh, createPlate, deletePlate, meshBeforeAnalysis, physicalPlates, planPlateMesh,
  plateOfShell, plateShellKeys, plateStatus, preparePlatesForAnalysis,
} from '../physical-plates';
import { syncDefinedLoadsForAnalysis } from '../defined-loads';
import { serializeProject, deserializeProject } from '../file';
import { plateLoops } from '../../model/physical/plate';
import { copyTransformed } from '../../model/edit/transformed-copy';
import { compose, rotation, translation } from '../../model/edit/affine';
import { plateLoadDefs } from '../physical-plates';
import type { AnalysisResults3D } from '../../engine/types-3d';

/** A 6 × 4 m plate at z = 0 with a 1 × 1 m hole, pinned at its corners. */
function build() {
  modelStore.clear(); historyStore.clear(); uiStore.analysisMode = 'pro';
  const c = [[0, 0], [6, 0], [6, 4], [0, 4]].map(([x, y]) => modelStore.addNode(x!, y!, 0));
  const h = [[2, 1.5], [3, 1.5], [3, 2.5], [2, 2.5]].map(([x, y]) => modelStore.addNode(x!, y!, 0));
  for (const n of c) modelStore.addSupport(n, 'pinned3d');
  const r = createPlate({ outline: c, holes: [h], materialId: 1, thickness: 0.15 });
  expect('id' in r, JSON.stringify(r)).toBe(true);
  return { id: (r as { id: number }).id, corners: c, hole: h };
}

const mesh = (id: number, size: number, followMembers = true) => {
  const plan = planPlateMesh(id, { size, followMembers, element: 'quad' })!;
  expect(plan.mesh, plan.refused).not.toBeNull();
  return applyPlateMeshes([plan]);
};

const solve = () => {
  syncDefinedLoadsForAnalysis();
  const r = modelStore.solve3D(false, false, true);
  expect(typeof r, String(r)).not.toBe('string');
  return r as AnalysisResults3D;
};

/** Resultant vertical reaction and its moments about the origin. */
const resultant = (r: AnalysisResults3D) => r.reactions.reduce((s, x) => {
  const n = modelStore.nodes.get(x.nodeId)!;
  return { fz: s.fz + x.fz, mx: s.mx + x.fz * n.y, my: s.my - x.fz * n.x };
}, { fz: 0, mx: 0, my: 0 });

const centroid = (nodes: number[]) => {
  const p = nodes.map((id) => modelStore.nodes.get(id)!);
  return { x: p.reduce((s, n) => s + n.x, 0) / p.length, y: p.reduce((s, n) => s + n.y, 0) / p.length };
};

beforeAll(async () => {
  await new Promise((r) => setTimeout(r, 0));
  expect(wasmSolver.isSolverReady(), 'real WASM solver required').toBe(true);
});

describe('a physical plate', () => {
  beforeEach(() => { modelStore.clear(); historyStore.clear(); meshBeforeAnalysis.value = true; });

  it('is made without a mesh and refuses outlines that make no plate', () => {
    const { id, corners, hole } = build();
    expect(modelStore.quads.size + modelStore.plates.size).toBe(0);
    expect(plateStatus(id)).toBe('unmeshed');
    expect(physicalPlates()[0]!.physical.kind).toBe('slab');
    // Out of order: the outline crosses itself.
    expect(createPlate({ outline: [corners[0]!, corners[2]!, corners[1]!, corners[3]!], materialId: 1, thickness: 0.1 })).toEqual({ problem: 'selfIntersecting' });
    const far = [[10, 10], [11, 10], [11, 11]].map(([x, y]) => modelStore.addNode(x!, y!, 0));
    expect(createPlate({ outline: corners, holes: [far], materialId: 1, thickness: 0.1 })).toEqual({ problem: 'holeOutside' });
    const up = modelStore.addNode(0, 0, 1);
    expect(createPlate({ outline: [corners[0]!, corners[1]!, corners[2]!, up], materialId: 1, thickness: 0.1 })).toEqual({ problem: 'notPlanar' });
    expect(hole.length).toBe(4);
  });

  it('meshes around its hole and holds its whole mesh', () => {
    const { id } = build();
    const r = mesh(id, 0.5);
    expect(r.plates).toBe(1);
    expect(plateStatus(id)).toBe('current');
    const g = modelStore.model.groups.get(id)!;
    expect(g.members.quads!.length + (g.members.plates?.length ?? 0)).toBe(modelStore.quads.size + modelStore.plates.size);
    // No shell inside the hole.
    for (const q of modelStore.quads.values()) {
      const c = centroid(q.nodes);
      expect(c.x > 2 && c.x < 3 && c.y > 1.5 && c.y < 2.5).toBe(false);
    }
    for (const key of plateShellKeys(id)) expect(plateOfShell(key)).toBe(id);
  });

  it('meshes again without accumulating, and its area load keeps its resultant', () => {
    const { id } = build();
    addPlateLoad(id, 1, 5);
    mesh(id, 0.5);
    const a = resultant(solve());
    const net = 6 * 4 - 1;
    // q is downward: the supports push up with q times the plate's area less its hole.
    expect(a.fz).toBeCloseTo(5 * net, 6);
    const census = () => ({ nodes: modelStore.nodes.size, quads: modelStore.quads.size, plates: modelStore.plates.size });
    mesh(id, 0.5);
    const same = census();
    mesh(id, 0.5);
    expect(census()).toEqual(same);
    mesh(id, 0.3);
    const b = resultant(solve());
    expect(b.fz).toBeCloseTo(a.fz, 6);
    expect(b.mx).toBeCloseTo(a.mx, 5);
    expect(b.my).toBeCloseTo(a.my, 5);
    // Only one load per shell: the definition's, written once.
    const onShell = new Map<number, number>();
    for (const l of modelStore.loads) if (l.type === 'surface3d') onShell.set(l.data.quadId, (onShell.get(l.data.quadId) ?? 0) + 1);
    expect(Math.max(...onShell.values())).toBe(1);
    // Taken out, the model is the plate's outline, its hole and nothing else.
    clearPlateMesh(id);
    expect(census()).toEqual({ nodes: 8, quads: 0, plates: 0 });
    expect(plateStatus(id)).toBe('unmeshed');
  });

  it('meets a beam across it and a column through it, and leaves a beam of another level alone', () => {
    const { id } = build();
    const b0 = modelStore.addNode(0, 1, 0), b1 = modelStore.addNode(6, 1, 0);
    const beam = modelStore.addElement(b0, b1);
    const c0 = modelStore.addNode(4.5, 3, -3), c1 = modelStore.addNode(4.5, 3, 3);
    modelStore.addElement(c0, c1);
    const u0 = modelStore.addNode(0, 3.3, 3), u1 = modelStore.addNode(6, 3.3, 3);
    const upper = modelStore.addElement(u0, u1);
    mesh(id, 0.5);
    // The beam is cut at the mesh nodes along it; the column at the slab; the upper beam is whole.
    expect(modelStore.elements.has(upper)).toBe(true);
    const onBeam = [...modelStore.elements.values()].filter((e) => [e.nodeI, e.nodeJ].every((n) => Math.abs(modelStore.nodes.get(n)!.y - 1) < 1e-9 && Math.abs(modelStore.nodes.get(n)!.z ?? 0) < 1e-9));
    expect(onBeam.length).toBeGreaterThan(5);
    const atSlab = [...modelStore.nodes.values()].find((n) => Math.abs(n.x - 4.5) < 1e-9 && Math.abs(n.y - 3) < 1e-9 && Math.abs(n.z ?? 0) < 1e-9);
    expect(atSlab, 'the column is met where it crosses the slab').toBeDefined();
    expect([...modelStore.elements.values()].filter((e) => e.nodeI === atSlab!.id || e.nodeJ === atSlab!.id).length).toBe(2);
    // Meshed again, then taken out: the beam and the column are whole members again.
    mesh(id, 0.4);
    expect(plateStatus(id)).toBe('current');
    clearPlateMesh(id);
    expect(modelStore.elements.size).toBe(3);
    expect(modelStore.elements.has(beam) || [...modelStore.elements.values()].some((e) => [e.nodeI, e.nodeJ].sort().join() === [b0, b1].sort().join())).toBe(true);
  });

  it('goes out of date when its outline moves or a member is added in its plane', () => {
    const { id, corners } = build();
    mesh(id, 0.5);
    expect(plateStatus(id)).toBe('current');
    const n0 = modelStore.addNode(1, 0, 0), n1 = modelStore.addNode(1, 4, 0);
    modelStore.addElement(n0, n1);
    expect(plateStatus(id)).toBe('stale');
    mesh(id, 0.5);
    expect(plateStatus(id)).toBe('current');
    modelStore.updateNode(corners[2]!, 6.5, 4, 0);
    expect(plateStatus(id)).toBe('stale');
  });

  it('meshes before an analysis only when asked to, and once', () => {
    const { id } = build();
    meshBeforeAnalysis.value = false;
    expect(preparePlatesForAnalysis()).toEqual({ ok: false, reason: 'needsMesh', plates: ['P1'] });
    meshBeforeAnalysis.value = true;
    const r = preparePlatesForAnalysis();
    expect(r.ok && r.meshed).toBe(1);
    expect(preparePlatesForAnalysis()).toEqual({ ok: true, meshed: 0, cells: 0 });
    expect(plateStatus(id)).toBe('current');
  });

  it('undoes and redoes its creation and its mesh without orphans', () => {
    modelStore.clear(); historyStore.clear(); uiStore.analysisMode = 'pro';
    const c = [[0, 0], [4, 0], [4, 4], [0, 4]].map(([x, y]) => modelStore.addNode(x!, y!, 0));
    const before = modelStore.nodes.size;
    const { id } = createPlate({ outline: c, materialId: 1, thickness: 0.2 }) as { id: number };
    mesh(id, 1);
    const meshed = { nodes: modelStore.nodes.size, quads: modelStore.quads.size };
    expect(meshed.quads).toBe(16);
    historyStore.undo();
    expect(modelStore.quads.size).toBe(0);
    expect(modelStore.nodes.size).toBe(before);
    expect(plateStatus(id)).toBe('unmeshed');
    historyStore.undo();
    expect(physicalPlates()).toHaveLength(0);
    historyStore.redo(); historyStore.redo();
    expect({ nodes: modelStore.nodes.size, quads: modelStore.quads.size }).toEqual(meshed);
    expect(plateStatus(id)).toBe('current');
    deletePlate(id);
    expect(modelStore.quads.size + modelStore.nodes.size).toBe(0);
  });

  it('copies whole: mesh still current, its load a definition on the copy', () => {
    const { id } = build();
    addPlateLoad(id, 1, 4);
    mesh(id, 0.5);
    const g = modelStore.model.groups.get(id)!;
    // Turned 90° about z and moved clear of the original, with its supports.
    const T = compose(translation([20, 0, 0]), rotation([0, 0, 0], [0, 0, 1], 90));
    copyTransformed({ nodes: g.members.nodes!, elements: [], quads: g.members.quads!, plates: g.members.plates ?? [] }, [T], { withSupports: true, withLoads: true });
    const all = physicalPlates();
    expect(all).toHaveLength(2);
    const copy = all.find((p) => p.id !== id)!;
    expect(plateStatus(copy.id)).toBe('current');
    expect(plateLoadDefs(copy.id)).toHaveLength(1);
    expect(plateLoadDefs(id)).toHaveLength(1);
    // Written by its own definition onto every shell of the copy, once.
    syncDefinedLoadsForAnalysis();
    const def = plateLoadDefs(copy.id)[0]!.id;
    const loaded = modelStore.loads.filter((l) => l.type === 'surface3d' && (l.data as { fromDef?: number }).fromDef === def).map((l) => l.data.quadId);
    expect(new Set(loaded).size).toBe(loaded.length);
    expect(loaded.length).toBe(copy.group.members.quads!.length);
  });

  it('opens again with its outline, mesh and load', () => {
    const { id } = build();
    addPlateLoad(id, 1, 3);
    mesh(id, 0.5);
    const loops = plateLoops(modelStore.model.groups.get(id)!);
    const text = serializeProject();
    modelStore.clear();
    expect(deserializeProject(text)).toBe(true);
    const g = modelStore.model.groups.get(id)!;
    expect(plateLoops(g)).toEqual(loops);
    expect(plateStatus(id)).toBe('current');
    expect(resultant(solve()).fz).toBeCloseTo(3 * 23, 6);
  });
});
