/**
 * Differential test oracle for rigid floor diaphragms via stiff penalty beams.
 *
 * This was the earlier TypeScript approximation before the Rust engine's multi-point
 * constraint (`Constraint::Diaphragm` in `solver/constraints.rs`) took over diaphragm
 * constraints natively.
 *
 * Kept only as a test oracle to verify that the engine's exact constraint formulation
 * agrees with the classical penalty-beam approach on 3D frame models.
 */
import type { SolverInput3D, SolverNode3D } from '../../../types-3d';

export interface DiaphragmConfig {
  levels: number[];
  tolerance?: number;
  stiffnessMultiplier?: number;
}

export function findCentroid(nodes: Map<number, SolverNode3D>, z: number, tol: number): { x: number; y: number } {
  let sx = 0, sy = 0, n = 0;
  for (const node of nodes.values()) {
    if (Math.abs((node.z ?? 0) - z) < tol) {
      sx += node.x;
      sy += node.y;
      n++;
    }
  }
  return n > 0 ? { x: sx / n, y: sy / n } : { x: 0, y: 0 };
}

/**
 * Apply rigid diaphragm constraints to a 3D model by adding very stiff
 * horizontal beams at each floor level.
 *
 * Test oracle only.
 */
export function applyRigidDiaphragm(
  input: SolverInput3D,
  config: DiaphragmConfig,
): SolverInput3D {
  const tol = config.tolerance ?? 0.05;
  const mult = config.stiffnessMultiplier ?? 1e6;

  const nodes = new Map(input.nodes);
  const elements = new Map(input.elements);
  const materials = new Map(input.materials);
  const sections = new Map(input.sections);
  const supports = new Map(input.supports);
  const loads = [...input.loads];

  let maxNodeId = 0, maxElemId = 0, maxMatId = 0, maxSecId = 0;
  for (const id of nodes.keys()) maxNodeId = Math.max(maxNodeId, id);
  for (const id of elements.keys()) maxElemId = Math.max(maxElemId, id);
  for (const id of materials.keys()) maxMatId = Math.max(maxMatId, id);
  for (const id of sections.keys()) maxSecId = Math.max(maxSecId, id);

  const diagMatId = maxMatId + 1;
  materials.set(diagMatId, {
    id: diagMatId,
    e: 200000 * mult,
    nu: 0.3,
  });

  const diagSecId = maxSecId + 1;
  sections.set(diagSecId, {
    id: diagSecId,
    a: 0.01 * mult,
    iy: 1e-4 * mult,
    iz: 1e-4 * mult,
    j: 1e-4 * mult,
  });

  let nextElemId = maxElemId + 1;

  for (const zLevel of config.levels) {
    const levelNodeIds: number[] = [];
    for (const [id, n] of nodes) {
      if (Math.abs((n.z ?? 0) - zLevel) < tol) {
        levelNodeIds.push(id);
      }
    }

    if (levelNodeIds.length < 2) continue;

    const centroid = findCentroid(nodes, zLevel, tol);

    let masterNodeId = levelNodeIds[0];
    let minDist = Infinity;
    for (const nid of levelNodeIds) {
      const n = nodes.get(nid)!;
      const dist = Math.sqrt((n.x - centroid.x) ** 2 + (n.y - centroid.y) ** 2);
      if (dist < minDist) {
        minDist = dist;
        masterNodeId = nid;
      }
    }

    for (const nid of levelNodeIds) {
      if (nid === masterNodeId) continue;

      const ni = nodes.get(masterNodeId)!;
      const nj = nodes.get(nid)!;
      const dx = nj.x - ni.x;
      const dy = nj.y - ni.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 0.001) continue;

      elements.set(nextElemId, {
        id: nextElemId,
        type: 'frame',
        nodeI: masterNodeId,
        nodeJ: nid,
        materialId: diagMatId,
        sectionId: diagSecId,
        releaseMyStart: false,
        releaseMyEnd: false,
        releaseMzStart: false,
        releaseMzEnd: false,
        releaseTStart: false,
        releaseTEnd: false,
      });
      nextElemId++;
    }
  }

  return { nodes, elements, materials, sections, supports, loads };
}

