/**
 * Slabs meshed when the model is solved, not when they are placed.
 *
 * A slab meshed at placement cuts its beams at every edge node of the mesh and adds tens of
 * shells per panel: the model the person edits gets heavy long before it is analysed. A slab
 * placed this way is one shell per panel, its beams whole, and it carries the mesh size it
 * will be analysed with (`PhysicalData.meshAtSolve`). The first solve meshes it in place
 * (`meshQuad`, which keeps its loads and groups and cuts the beams so slab and beams share their
 * nodes), so the analysis, the results on screen and the slab design read the mesh they always
 * read. When the editing goes on, `simplifySlabs` puts the panel back: one shell, its loads, the
 * beams merged where the mesh had cut them and the mesh's own nodes gone.
 *
 * Every call is one undo step.
 */
import { modelStore } from '../../store/model.svelte';
import { physicalGroups } from '../../store/physical';
import { meshQuad } from '../edit/mesh-region';
import { mergeCollinear } from '../edit/merge-collinear';
import type { PhysicalData } from '../physical/types';
import type { SurfaceLoad3D, ThermalLoadQuad3D } from '../../store/model.svelte';

type Deferred = ReturnType<typeof physicalGroups>[number] & { physical: PhysicalData & { meshAtSolve: NonNullable<PhysicalData['meshAtSolve']> } };
const deferred = (): Deferred[] => physicalGroups().filter((g): g is Deferred => g.physical.kind === 'slab' && !!g.physical.meshAtSolve);

/** The slabs waiting for their mesh: placed simple and not yet solved. */
export const slabsToMesh = (): number => deferred().filter((g) => !g.physical.meshAtSolve.meshed).length;
/** The slabs meshed by a solve, which can go back to one shell per panel. */
export const slabsToSimplify = (): number => deferred().filter((g) => g.physical.meshAtSolve.meshed).length;

/** Mesh every slab that is waiting for it, at its own size. Returns how many slabs and shells. */
export function meshDeferredSlabs(): { slabs: number; shells: number } {
  const out = { slabs: 0, shells: 0 };
  const waiting = deferred().filter((g) => !g.physical.meshAtSolve.meshed);
  if (waiting.length === 0) return out;
  modelStore.batch(() => {
    for (const g of waiting) {
      const quads = g.members.quads ?? [];
      // One shell per panel is what was placed; anything else was edited by hand and is left as it is.
      if (quads.length !== 1) continue;
      const r = meshQuad(quads[0]!, { density: { mode: 'targetSize', size: g.physical.meshAtSolve.size }, splitBeams: true });
      if (!r || 'refused' in r) continue;
      out.slabs++; out.shells += r.quadCount;
      modelStore.setGroupData(g.id, { ...g.physical, meshAtSolve: { ...g.physical.meshAtSolve, meshed: true } } as unknown as Record<string, unknown>);
    }
  });
  return out;
}

/**
 * Put meshed slabs back to one shell per panel: the panel's corners, the material, thickness and
 * offset of its shells, the surface and thermal loads they carried (the same per square metre on
 * every shell of the mesh, so the first one's), the beams merged where the mesh had cut them, and
 * the mesh's own nodes removed. `ids` limits it to some slabs; absent, every one.
 */
export function simplifySlabs(ids?: readonly number[]): { slabs: number; removedNodes: number } {
  const out = { slabs: 0, removedNodes: 0 };
  const meshed = deferred().filter((g) => g.physical.meshAtSolve.meshed && (!ids || ids.includes(g.id)));
  if (meshed.length === 0) return out;
  modelStore.batch(() => {
    for (const g of meshed) {
      const quads = (g.members.quads ?? []).filter((id) => modelStore.quads.has(id));
      const panel = g.physical.meshAtSolve.panel;
      if (quads.length === 0 || panel.length !== 4 || panel.some((n) => !modelStore.nodes.has(n))) continue;
      const first = modelStore.quads.get(quads[0]!)!;
      const onFirst = (l: { data: { quadId?: number; on?: string } }) => l.data.quadId === first.id && !l.data.on;
      const surface = modelStore.loads.filter((l) => l.type === 'surface3d' && onFirst(l as never)).map((l) => l.data as SurfaceLoad3D);
      const thermal = modelStore.loads.filter((l) => l.type === 'thermalQuad3d' && onFirst(l as never)).map((l) => l.data as ThermalLoadQuad3D);
      const meshNodes = new Set(quads.flatMap((id) => modelStore.quads.get(id)!.nodes));
      for (const n of panel) meshNodes.delete(n);
      for (const id of quads) modelStore.removeQuad(id);
      const q = modelStore.addQuad(panel as [number, number, number, number], first.materialId, first.thickness);
      if (first.offset) modelStore.setShellOffset('quad', q, first.offset);
      for (const s of surface) {
        const { id: _id, quadId: _q, q: qv, caseId, qNodes: _n, ...extra } = s;
        modelStore.addSurfaceLoad3D(q, qv, caseId, extra);
      }
      for (const t of thermal) modelStore.addThermalLoadQuad3D(q, t.dtUniform, t.dtGradient, t.caseId);
      const group = modelStore.model.groups.get(g.id);
      if (group) modelStore.setGroupMembers(g.id, { ...group.members, quads: [q] });
      modelStore.setGroupData(g.id, { ...g.physical, meshAtSolve: { ...g.physical.meshAtSolve, meshed: false } } as unknown as Record<string, unknown>);
      // The beams the mesh cut along the panel's edges, whole again where nothing else holds the cut.
      const onEdge = [...modelStore.elements.values()].filter((e) => meshNodes.has(e.nodeI) || meshNodes.has(e.nodeJ)).map((e) => e.id);
      out.removedNodes += mergeCollinear(onEdge).removedNodes;
      // The mesh's own nodes inside the panel, now holding nothing.
      const held = new Set<number>();
      for (const e of modelStore.elements.values()) { held.add(e.nodeI); held.add(e.nodeJ); }
      for (const s of modelStore.quads.values()) s.nodes.forEach((n) => held.add(n));
      for (const p of modelStore.plates.values()) p.nodes.forEach((n) => held.add(n));
      for (const s of modelStore.supports.values()) held.add(s.nodeId);
      for (const l of modelStore.loads) if ('nodeId' in l.data) held.add((l.data as { nodeId: number }).nodeId);
      for (const n of meshNodes) if (modelStore.nodes.has(n) && !held.has(n)) { modelStore.removeNode(n); out.removedNodes++; }
      out.slabs++;
    }
  });
  return out;
}
