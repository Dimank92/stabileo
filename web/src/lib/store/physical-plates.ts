/**
 * Physical plates in the model (`model/physical/plate.ts`): made from an outline, meshed on demand
 * one at a time or many together, meshed again without leaving anything behind, and made ready
 * for an analysis.
 *
 * ── Meshing is planned, then applied ──────────────────────────────
 *
 * `planPlateMeshes` runs the mesher for each plate and changes nothing: the reader sees the count,
 * the quality and what cannot be carried over before anything happens, and a long plan can be
 * stopped between plates. `applyPlateMeshes` puts the planned meshes in as one undo step: each
 * plate's old mesh comes out first (`clearPlateMesh`), so meshing twice never stacks two meshes.
 *
 * ── What a mesh leaves when it goes ───────────────────────────────
 *
 * Its shells, the nodes it created that nothing else holds, and the cuts it made in the members
 * (merged back where the cut node held nothing else, `merge-collinear.ts`). A node of the mesh
 * that something else took meanwhile (a support, a nodal load, a member drawn to it) stays, and
 * the next mesh meets it. Area loads defined on the plate (`floorLoad` definitions targeting it)
 * follow every mesh; a load put by hand on one shell of the mesh cannot, and is reported.
 */
import { modelStore } from './model.svelte';
import { uiStore } from './ui.svelte';
import { physicalDataOf, type PhysicalData } from '../model/physical';
import {
  DEFAULT_PLATE_MESH, externalFeatures, gridOfSpec, meshQuality, plateGeometry, plateKindOf, plateLoops, plateMeshInput, plateProblem,
  plateSignature, plateSurfaceOf, type MeshQuality, type PlateMeshSpec, type PlateProblem, type PlateSurface,
} from '../model/physical/plate';
import { generateMesh, MAX_MESH_CELLS, type MeshOutput } from '../model/edit/mesher';
import { frameAlong, gridMesh } from '../model/edit/grid-mesh';
import { applyMesh } from '../model/edit/mesh-apply';
import { mergeCollinear } from '../model/edit/merge-collinear';
import { addFloorLoadDef } from './defined-loads';
import { constraintNodes } from '../engine/steel/unbraced-length';
import { meshBeforeAnalysis } from './mesh-policy.svelte';
export { meshBeforeAnalysis };
import type { FloorLoadDef } from '../model/loads/floor-definitions';
import type { ModelGroup } from './model.svelte';

export type PlateStatus = 'unmeshed' | 'current' | 'stale' | 'invalid';

export interface PlateEntry { id: number; name: string; group: ModelGroup; physical: PhysicalData; surface: PlateSurface }

/** Every physical plate of the model. */
export function physicalPlates(): PlateEntry[] {
  const out: PlateEntry[] = [];
  for (const g of modelStore.model.groups.values()) {
    const physical = physicalDataOf(g);
    const surface = plateSurfaceOf(g);
    if (physical && surface) out.push({ id: g.id, name: g.name, group: g, physical, surface });
  }
  return out;
}

export function plateEntry(id: number): PlateEntry | null {
  const g = modelStore.model.groups.get(id);
  const physical = g ? physicalDataOf(g) : null;
  const surface = g ? plateSurfaceOf(g) : null;
  return g && physical && surface ? { id, name: g.name, group: g, physical, surface } : null;
}

/** The plate a shell belongs to ("q12" or "p4"), or null. */
export function plateOfShell(key: string): number | null {
  const kind = key[0] === 'q' ? 'quads' : key[0] === 'p' ? 'plates' : null;
  const id = Number(key.slice(1));
  if (!kind || !Number.isFinite(id)) return null;
  for (const p of physicalPlates()) if (p.group.members[kind]?.includes(id)) return p.id;
  return null;
}

/** The selection keys of a plate's shells. */
export function plateShellKeys(id: number): string[] {
  const g = modelStore.model.groups.get(id);
  return [...(g?.members.quads ?? []).map((q) => `q${q}`), ...(g?.members.plates ?? []).map((p) => `p${p}`)];
}
uiStore.setPlateShellResolver(plateShellKeys);

const plateModel = () => ({ nodes: modelStore.nodes, elements: modelStore.elements });

/**
 * What a plate's mesh has to meet. Members are read whole through the nodes every plate's mesh
 * created, so meshing one plate never puts its neighbours out of date; the nodes of the other
 * shells on its outline are points the mesh passes through, so neighbouring meshes meet node to
 * node.
 */
function featuresOf(e: PlateEntry) {
  const geo = plateGeometry(plateModel(), e.group);
  if (!geo) return null;
  const loops = plateLoops(e.group);
  const mine = new Set(loops.owned);
  const owned = new Set<number>();
  for (const g of modelStore.model.groups.values()) if (plateSurfaceOf(g)) for (const n of plateLoops(g).owned) owned.add(n);
  const myShells = new Set([...(e.group.members.quads ?? []).map((q) => `q${q}`), ...(e.group.members.plates ?? []).map((p) => `p${p}`)]);
  const neighbours = new Set<number>();
  for (const q of modelStore.quads.values()) if (!myShells.has(`q${q.id}`)) q.nodes.forEach((n) => neighbours.add(n));
  for (const p of modelStore.plates.values()) if (!myShells.has(`p${p.id}`)) p.nodes.forEach((n) => neighbours.add(n));
  const loopNodes = new Set([...loops.outer, ...loops.holes.flat()]);
  return { geo, features: externalFeatures(plateModel(), geo, owned, loopNodes, { mine, neighbours }) };
}

let statusMemo: { version: number; groups: unknown; map: Map<number, PlateStatus> } | null = null;

/** Whether a plate has a mesh, and whether it is the one its definition asks for now. */
export function plateStatus(id: number): PlateStatus {
  const version = modelStore.modelVersion;
  if (!statusMemo || statusMemo.version !== version || statusMemo.groups !== modelStore.model.groups) statusMemo = { version, groups: modelStore.model.groups, map: new Map() };
  const hit = statusMemo.map.get(id);
  if (hit) return hit;
  const e = plateEntry(id);
  let s: PlateStatus;
  if (!e) s = 'invalid';
  else {
    const f = featuresOf(e);
    const shells = (e.group.members.quads?.length ?? 0) + (e.group.members.plates?.length ?? 0);
    if (!f) s = 'invalid';
    else if (!e.surface.meshed || shells === 0) s = 'unmeshed';
    else s = plateSignature(f.geo, e.surface.mesh, f.features) === e.surface.meshed.signature ? 'current' : 'stale';
  }
  statusMemo.map.set(id, s);
  return s;
}

/** The plates whose mesh is missing or out of date. */
export function platesNeedingMesh(): number[] {
  return physicalPlates().filter((p) => { const s = plateStatus(p.id); return s === 'unmeshed' || s === 'stale'; }).map((p) => p.id);
}

// ─── Making a plate ──────────────────────────────────────────────

export interface NewPlate { outline: number[]; holes?: number[][]; materialId: number; thickness: number; name?: string; mesh?: PlateMeshSpec }

/** A plate from an outline of nodes; one undo step. The problem when the nodes make none. */
export function createPlate(p: NewPlate): { id: number } | { problem: PlateProblem } {
  const holes = p.holes ?? [];
  const problem = plateProblem(plateModel(), p.outline, holes);
  if (problem) return { problem };
  const surface: PlateSurface = { loops: [p.outline.length, ...holes.map((h) => h.length)], materialId: p.materialId, thickness: p.thickness, mesh: { ...(p.mesh ?? DEFAULT_PLATE_MESH) } };
  const temp = { id: 0, members: { nodes: [...p.outline, ...holes.flat()] }, data: { surface } };
  const geo = plateGeometry(plateModel(), temp)!;
  const physical: PhysicalData & { surface: PlateSurface } = { kind: plateKindOf(geo), provenance: 'declared', surface };
  const n = physicalPlates().length + 1;
  const id = modelStore.addGroup(p.name ?? `P${n}`, 'physicalMember', temp.members, { data: physical as unknown as Record<string, unknown> });
  return { id };
}

function writeSurface(id: number, surface: PlateSurface | ((s: PlateSurface) => PlateSurface)): void {
  const g = modelStore.model.groups.get(id);
  const d = g ? physicalDataOf(g) : null;
  const s = g ? plateSurfaceOf(g) : null;
  if (!g || !d || !s) return;
  const next = typeof surface === 'function' ? surface(s) : surface;
  modelStore.setGroupData(id, { ...d, surface: next } as unknown as Record<string, unknown>);
}

/** Change a plate's material or thickness, on its definition and its shells; one undo step. */
export function setPlateProperties(id: number, props: { materialId?: number; thickness?: number }): void {
  const e = plateEntry(id);
  if (!e) return;
  if (props.thickness !== undefined && !(props.thickness > 0)) return;
  if (props.materialId !== undefined && !modelStore.materials.has(props.materialId)) return;
  modelStore.batch(() => {
    writeSurface(id, (s) => ({ ...s, ...props }));
    for (const q of e.group.members.quads ?? []) modelStore.updateQuad(q, props);
    for (const q of e.group.members.plates ?? []) modelStore.updatePlate(q, props);
  });
}

/**
 * A shell's material or thickness, as every table and panel edits it. A shell of a physical
 * plate's mesh is the plate's: the edit goes to the plate (its definition and its whole mesh),
 * so a remesh does not undo it and the plate never says one thing while its elements say another.
 * Returns the plates edited that way, for the caller to say so.
 */
export function setShellProperties(keys: readonly string[], patch: { materialId?: number; thickness?: number }): number[] {
  const plates = new Set<number>();
  modelStore.batch(() => {
    for (const k of keys) {
      const plate = plateOfShell(k);
      if (plate !== null) { plates.add(plate); continue; }
      const id = Number(k.slice(1));
      if (k[0] === 'p') modelStore.updatePlate(id, patch); else modelStore.updateQuad(id, patch);
    }
    for (const id of plates) setPlateProperties(id, patch);
  });
  return [...plates];
}

/** Change how a plate is to be meshed. Its mesh, if any, is then out of date. */
export function setPlateMeshSpec(ids: readonly number[], spec: Partial<PlateMeshSpec>): void {
  modelStore.batch(() => { for (const id of ids) writeSurface(id, (s) => ({ ...s, mesh: { ...s.mesh, ...spec } })); });
}

/** Replace a plate's holes (node loops); one undo step. The problem when they do not fit. */
export function setPlateHoles(id: number, holes: number[][]): PlateProblem | null {
  const e = plateEntry(id);
  if (!e) return 'needNodes';
  const loops = plateLoops(e.group);
  const problem = plateProblem(plateModel(), loops.outer, holes);
  if (problem) return problem;
  modelStore.batch(() => {
    modelStore.setGroupMembers(id, { ...e.group.members, nodes: [...loops.outer, ...holes.flat(), ...loops.owned] });
    writeSurface(id, (s) => ({ ...s, loops: [loops.outer.length, ...holes.map((h) => h.length)] }));
  });
  return null;
}

/** An area load on the whole plate, kept as a definition so that it follows every mesh. */
export function addPlateLoad(id: number, caseId: number, q: number, name?: string): number | null {
  const e = plateEntry(id);
  if (!e || !Number.isFinite(q)) return null;
  const def: FloorLoadDef = { caseId, q, target: { by: 'group', groupId: id }, distribution: 'slab' };
  return addFloorLoadDef(name ?? `${e.name} · q`, def);
}

/** The area-load definitions that target a plate. */
export function plateLoadDefs(id: number): ModelGroup[] {
  return [...modelStore.model.groups.values()].filter((g) => {
    if (g.kind !== 'floorLoad') return false;
    const t = (g.data as unknown as FloorLoadDef | undefined)?.target;
    return t?.by === 'group' && t.groupId === id;
  });
}

// ─── Meshing ─────────────────────────────────────────────────────

export interface PlateMeshPlan {
  id: number;
  name: string;
  spec: PlateMeshSpec;
  /** Null when refused, with the reason. */
  mesh: MeshOutput | null;
  refused?: 'invalid' | 'tooManyCells' | 'cannotMesh';
  cells: number;
  quality: MeshQuality | null;
  signature: string;
  input: ReturnType<typeof plateMeshInput> | null;
  /** Loads put by hand on shells of the current mesh, which a new mesh cannot carry. */
  localLoads: number;
  /** Nodes of the current mesh that something else holds, which stay and are met by the new one. */
  keptNodes: number;
  /** The element asked for and what the mesher gives: quadrilaterals from triangles, outside the structured paths. */
  mixed: boolean;
  /** A grid's mesh: its rectangles, the strip along the boundary, and the rectangles' sides as they came out. */
  grid?: { gridCells: number; bandCells: number; cellA: number; cellB: number };
}

/** Every node something holds; `except` a plate whose own outline and mesh do not count. */
const held = (except?: number): Set<number> => {
  const s = new Set<number>();
  for (const e of modelStore.elements.values()) { s.add(e.nodeI); s.add(e.nodeJ); }
  for (const q of modelStore.quads.values()) q.nodes.forEach((n) => s.add(n));
  for (const p of modelStore.plates.values()) p.nodes.forEach((n) => s.add(n));
  for (const sp of modelStore.supports.values()) s.add(sp.nodeId);
  for (const l of modelStore.loads) if ('nodeId' in l.data) s.add((l.data as { nodeId: number }).nodeId);
  for (const n of constraintNodes(modelStore.model.constraints)) s.add(n);
  for (const g of modelStore.model.groups.values()) if (g.id !== except) g.members.nodes?.forEach((n) => s.add(n));
  return s;
};

function localLoadsOn(e: PlateEntry): number {
  const quads = new Set(e.group.members.quads ?? []), plates = new Set(e.group.members.plates ?? []);
  return modelStore.loads.filter((l) => {
    if (l.type !== 'surface3d' && l.type !== 'thermalQuad3d') return false;
    const d = l.data as { quadId: number; on?: string; fromDef?: number };
    if (d.fromDef !== undefined) return false;
    return d.on === 'plate' ? plates.has(d.quadId) : quads.has(d.quadId);
  }).length;
}

/** Nodes of a plate's mesh that something other than its own shells holds. */
function keptMeshNodes(e: PlateEntry): number {
  const owned = plateLoops(e.group).owned;
  if (!owned.length) return 0;
  const mine = new Set([...(e.group.members.quads ?? []).map((q) => `q${q}`), ...(e.group.members.plates ?? []).map((p) => `p${p}`)]);
  const busy = new Set<number>();
  for (const sp of modelStore.supports.values()) busy.add(sp.nodeId);
  for (const l of modelStore.loads) if ('nodeId' in l.data) busy.add((l.data as { nodeId: number }).nodeId);
  for (const q of modelStore.quads.values()) if (!mine.has(`q${q.id}`)) q.nodes.forEach((n) => busy.add(n));
  for (const p of modelStore.plates.values()) if (!mine.has(`p${p.id}`)) p.nodes.forEach((n) => busy.add(n));
  return owned.filter((n) => busy.has(n)).length;
}

/**
 * The mesh each plate would get, without changing the model. `spec` overrides each plate's own
 * settings (the batch's common settings). Yields between plates and stops when `signal` aborts.
 */
export async function planPlateMeshes(ids: readonly number[], spec?: Partial<PlateMeshSpec>, opts: { signal?: AbortSignal; onProgress?: (done: number, total: number) => void } = {}): Promise<PlateMeshPlan[] | null> {
  const out: PlateMeshPlan[] = [];
  for (let i = 0; i < ids.length; i++) {
    if (opts.signal?.aborted) return null;
    const plan = planPlateMesh(ids[i]!, spec);
    if (plan) out.push(plan);
    opts.onProgress?.(i + 1, ids.length);
    // Give the page a frame between plates, so a long batch can be watched and stopped.
    if (ids.length > 1) await new Promise((r) => setTimeout(r, 0));
  }
  return out;
}

/** One plate's plan, synchronously. */
export function planPlateMesh(id: number, specOverride?: Partial<PlateMeshSpec>): PlateMeshPlan | null {
  const e = plateEntry(id);
  if (!e) return null;
  const spec = { ...e.surface.mesh, ...(specOverride ?? {}) };
  const base = { id, name: e.name, spec, localLoads: localLoadsOn(e), keptNodes: keptMeshNodes(e) };
  const f = featuresOf(e);
  if (!f) return { ...base, mesh: null, refused: 'invalid', cells: 0, quality: null, signature: '', input: null, mixed: false };
  const input = plateMeshInput(f.geo, spec, f.features);
  const signature = plateSignature(f.geo, spec, f.features);
  // A plate far past the cap would be a long wait for a refusal: estimate from the area first.
  const estimate = plateAreaEstimate(f.geo, spec);
  if (estimate > MAX_MESH_CELLS * 1.5) return { ...base, mesh: null, refused: 'tooManyCells', cells: Math.round(estimate), quality: null, signature, input, mixed: false };
  const grid = gridOfSpec(spec);
  if (grid) {
    const frame = grid.orient === 'globalX' ? frameAlong(f.geo.frame, [1, 0, 0]) : f.geo.frame;
    const g = gridMesh({
      outer: f.geo.outer, holes: f.geo.holes, frame, a: grid.a, b: grid.b, element: 'quad',
      ...(input.sides ? { sides: input.sides } : {}), fixedPoints: input.fixedPoints ?? [], fixedSegments: input.fixedSegments ?? [],
    });
    if (!g) return { ...base, mesh: null, refused: 'cannotMesh', cells: 0, quality: null, signature, input, mixed: false };
    if (g.cells.length > MAX_MESH_CELLS) return { ...base, mesh: null, refused: 'tooManyCells', cells: g.cells.length, quality: null, signature, input, mixed: false };
    const { gridCells, bandCells, cellA, cellB, ...mesh } = g;
    return { ...base, mesh, cells: mesh.cells.length, quality: meshQuality(mesh), signature, input, mixed: bandCells > 0, grid: { gridCells, bandCells, cellA, cellB } };
  }
  const mesh = generateMesh(input);
  if (!mesh) return { ...base, mesh: null, refused: 'cannotMesh', cells: 0, quality: null, signature, input, mixed: false };
  if (mesh.cells.length > MAX_MESH_CELLS) return { ...base, mesh: null, refused: 'tooManyCells', cells: mesh.cells.length, quality: null, signature, input, mixed: false };
  return { ...base, mesh, cells: mesh.cells.length, quality: meshQuality(mesh), signature, input, mixed: spec.element === 'quad' && !mesh.structured };
}

function plateAreaEstimate(geo: NonNullable<ReturnType<typeof plateGeometry>>, spec: PlateMeshSpec): number {
  const proj = (l: typeof geo.outer) => { let s = 0; const f = geo.frame; for (let i = 0; i < l.length; i++) { const a = l[i]!, b = l[(i + 1) % l.length]!; const ua = (a[0] - f.o[0]) * f.u[0] + (a[1] - f.o[1]) * f.u[1] + (a[2] - f.o[2]) * f.u[2], va = (a[0] - f.o[0]) * f.v[0] + (a[1] - f.o[1]) * f.v[1] + (a[2] - f.o[2]) * f.v[2]; const ub = (b[0] - f.o[0]) * f.u[0] + (b[1] - f.o[1]) * f.u[1] + (b[2] - f.o[2]) * f.u[2], vb = (b[0] - f.o[0]) * f.v[0] + (b[1] - f.o[1]) * f.v[1] + (b[2] - f.o[2]) * f.v[2]; s += ua * vb - ub * va; } return Math.abs(s / 2); };
  const area = proj(geo.outer) - geo.holes.reduce((s, h) => s + proj(h), 0);
  const grid = gridOfSpec(spec);
  return area / (grid ? grid.a * grid.b : spec.size * spec.size);
}

/**
 * Take a plate's mesh out: its shells, the cuts it made in the members where nothing else holds
 * the cut, and the nodes it created that nothing holds. The plate keeps its definition and its
 * area loads. One undo step with the caller's.
 */
export function clearPlateMesh(id: number): { removedNodes: number; keptNodes: number } {
  const e = plateEntry(id);
  if (!e) return { removedNodes: 0, keptNodes: 0 };
  const loops = plateLoops(e.group);
  const owned = new Set(loops.owned);
  let removedNodes = 0, keptNodes = 0;
  modelStore.batch(() => {
    for (const q of e.group.members.quads ?? []) if (modelStore.quads.has(q)) modelStore.removeQuad(q);
    for (const p of e.group.members.plates ?? []) if (modelStore.plates.has(p)) modelStore.removePlate(p);
    // The members the mesh cut, whole again where the cut node is the mesh's and holds nothing else.
    const cut = [...modelStore.elements.values()].filter((el) => owned.has(el.nodeI) || owned.has(el.nodeJ)).map((el) => el.id);
    if (cut.length) removedNodes += mergeCollinear(cut).removedNodes;
    const h = held(id);
    for (const n of owned) {
      if (!modelStore.nodes.has(n)) continue;
      if (h.has(n)) { keptNodes++; continue; }
      modelStore.removeNode(n); removedNodes++;
    }
    const g = modelStore.model.groups.get(id)!;
    modelStore.setGroupMembers(id, { ...g.members, nodes: [...loops.outer, ...loops.holes.flat()], quads: [], plates: [] });
    writeSurface(id, (s) => { const { meshed: _m, ...rest } = s; return rest; });
  });
  return { removedNodes, keptNodes };
}

export interface PlateMeshReport { plates: number; cells: number; refused: Array<{ id: number; reason: string }>; droppedLocalLoads: number; keptNodes: number }

/** Put the planned meshes in, replacing each plate's old one, as one undo step. */
export function applyPlateMeshes(plans: readonly PlateMeshPlan[]): PlateMeshReport {
  const report: PlateMeshReport = { plates: 0, cells: 0, refused: [], droppedLocalLoads: 0, keptNodes: 0 };
  const ok = plans.filter((p) => p.mesh && p.input);
  for (const p of plans) if (!p.mesh) report.refused.push({ id: p.id, reason: p.refused ?? 'cannotMesh' });
  if (!ok.length) return report;
  modelStore.batch(() => {
    for (const [k, planned] of ok.entries()) {
      // After the first, each plate is planned again on the model as it now is, so it meets the
      // meshes just put in beside it.
      const p = k === 0 ? planned : (planPlateMesh(planned.id, planned.spec) ?? planned);
      if (!p.mesh || !p.input) { report.refused.push({ id: p.id, reason: p.refused ?? 'cannotMesh' }); continue; }
      const e = plateEntry(p.id);
      if (!e) continue;
      report.droppedLocalLoads += p.localLoads;
      report.keptNodes += clearPlateMesh(p.id).keptNodes;
      const before = new Set(modelStore.nodes.keys());
      const r = applyMesh(p.input!, { materialId: e.surface.materialId, thickness: e.surface.thickness, splitBeams: p.spec.followMembers }, p.mesh!);
      if (!r || r.occupied) { report.refused.push({ id: p.id, reason: 'occupied' }); continue; }
      const created = [...new Set(r.nodeIds)].filter((n) => !before.has(n));
      const g = modelStore.model.groups.get(p.id)!;
      const loops = plateLoops(g);
      modelStore.setGroupMembers(p.id, { ...g.members, nodes: [...loops.outer, ...loops.holes.flat(), ...created], quads: r.quads, plates: r.plates });
      writeSurface(p.id, (s) => ({ ...s, mesh: p.spec, meshed: { signature: p.signature, cells: r.quads.length + r.plates.length, quality: p.quality! } }));
      report.plates++;
      report.cells += r.quads.length + r.plates.length;
    }
  });
  return report;
}

/** Remove a plate with its mesh and its area loads; one undo step. */
export function deletePlate(id: number): void {
  if (!plateEntry(id)) return;
  modelStore.batch(() => {
    clearPlateMesh(id);
    for (const d of plateLoadDefs(id)) modelStore.removeGroup(d.id);
    const g = modelStore.model.groups.get(id);
    const corner = new Set(g?.members.nodes ?? []);
    modelStore.removeGroup(id);
    // Outline nodes that held only the plate go with it.
    const h = held();
    for (const n of corner) if (modelStore.nodes.has(n) && !h.has(n)) modelStore.removeNode(n);
  });
}

// ─── Before an analysis ──────────────────────────────────────────

export type PlatePreparation =
  | { ok: true; meshed: number; cells: number }
  | { ok: false; reason: 'needsMesh'; plates: string[] }
  | { ok: false; reason: 'refused'; plates: string[] };

/**
 * Mesh what an analysis needs and nothing else: the plates with no mesh or an out-of-date one,
 * each at its own settings, as one undo step. With meshing before the analysis turned off, says
 * which plates stop it instead. Runs once per solve, before its inputs are taken.
 */
export function preparePlatesForAnalysis(): PlatePreparation {
  const need = platesNeedingMesh();
  if (!need.length) return { ok: true, meshed: 0, cells: 0 };
  const names = (ids: number[]) => ids.map((id) => plateEntry(id)?.name ?? `#${id}`);
  if (!meshBeforeAnalysis.value) return { ok: false, reason: 'needsMesh', plates: names(need) };
  const plans = need.map((id) => planPlateMesh(id)).filter((p): p is PlateMeshPlan => !!p);
  const r = applyPlateMeshes(plans);
  if (r.refused.length) return { ok: false, reason: 'refused', plates: names(r.refused.map((x) => x.id)) };
  return { ok: true, meshed: r.plates, cells: r.cells };
}

/**
 * The plates a marquee takes: by their shells when meshed (all of them in a window, any of them
 * across), by their outline's nodes when not.
 */
export function platesInMarquee(shells: ReadonlySet<string>, nodeIn: (id: number) => boolean, isWindow: boolean): number[] {
  const out: number[] = [];
  for (const p of physicalPlates()) {
    const keys = plateShellKeys(p.id);
    const hit = keys.length
      ? (isWindow ? keys.every((k) => shells.has(k)) : keys.some((k) => shells.has(k)))
      : (() => { const n = plateLoops(p.group).outer; return isWindow ? n.every(nodeIn) : n.some(nodeIn); })();
    if (hit) out.push(p.id);
  }
  return out;
}
