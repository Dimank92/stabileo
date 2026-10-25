/**
 * Translate 3D-authored constraints onto the 2D solver's DOF convention.
 *
 * The app stores ONE constraints array, always authored in 3D semantics
 * (ProConstraintsTab): DOF indices 0..5 = [ux, uy, uz, rx, ry, rz] and
 * eccentric releases as a 6-bool array. The Rust 2D solver uses 3 DOFs per
 * node — [0=ux, 1=uz, 2=ry] — and hard-errors on any dof >= 3, while a 6-bool
 * releases array would be silently misread ([1]=uy interpreted as the uz
 * slot). This module is the single dimension-translation layer between the
 * stored shape and the 2D wire (mirrors how supports and loads are remapped):
 *
 *   3D dof 0 (ux) → 2D 0 (ux)        in-plane horizontal
 *   3D dof 2 (uz) → 2D 1 (uz)        in-plane vertical
 *   3D dof 4 (ry) → 2D 2 (ry)        in-plane rotation
 *   3D dofs 1/3/5 (uy/rx/rz)         out-of-plane — dropped (the 2D solve
 *                                    restrains those implicitly)
 *
 * Constraints left without any in-plane DOF are dropped entirely.
 *
 * The plane model is the X–Z plane of the 3D one, and its rotation θ is counter-clockwise from
 * +X to +Z, a rotation about −Y: θ = −ry (docs/solver/assembly-constraints.md §2.3). An equation
 * that mixes ry with anything else (a linear MPC) changes the sign of its ry coefficient; an
 * equality of rotations does not care.
 *
 * Two constraints meant one thing in a space solve and another in the plane one (AC-3, AC-5):
 *   · A rigid link with no DOFs listed is "all translational" in 3D (the slave's rotation stays
 *     free, a pinned arm), but the plane solver's default ties the rotation (a fixed arm). It is
 *     sent with its translations listed, [ux, uz], so both solves model the same link.
 *   · A diaphragm is a rigid body in ITS plane. The plane solver's diaphragm is always the
 *     in-plane rigid body of the model plane, so a floor ("XY", horizontal) diaphragm made the
 *     beams it ties rigid in vertical bending. Its trace on the X–Z plane is sent instead:
 *       XY (a floor)            → ux_s = ux_m            (θz and Δy are out of the plane)
 *       YZ (a wall across)      → uz_s = uz_m            (θx and Δy are out of the plane)
 *       XZ (the model's plane)  → the rigid body, rotation tied (as the space solve ties it)
 */

import type { Constraint3D } from './types-3d';
import type { DrawPlane } from '../geometry/plane-projection';

interface PlaneMappingConfig {
  dofMap: Record<number, number>;
  rotDof3d: number;
  rotSign: number;
  eccentricOffsets: (c: { offsetX?: number; offsetY?: number; offsetZ?: number }) => { offsetX: number; offsetY: number };
  eccentricReleases: (r: boolean[]) => [boolean, boolean, boolean];
  diaphragmTrace: (diaphragmPlane: string, masterNode: number, slaveNode: number, orig: Constraint3D) => Constraint3D;
}

const PLANE_CONFIGS: Record<DrawPlane, PlaneMappingConfig> = {
  xz: {
    dofMap: { 0: 0, 2: 1, 4: 2 },
    rotDof3d: 4,
    rotSign: -1,
    eccentricOffsets: (c) => ({ offsetX: c.offsetX ?? 0, offsetY: c.offsetZ ?? 0 }),
    eccentricReleases: (r) => [r[0] ?? false, r[2] ?? false, r[4] ?? false],
    diaphragmTrace: (dp, masterNode, slaveNode, orig) => {
      if (dp === 'XZ') return { type: 'rigidLink', masterNode, slaveNode, dofs: [0, 1, 2] };
      if (dp === 'YZ') return { type: 'equalDOF', masterNode, slaveNode, dofs: [1] };
      if (dp === 'XY') return { type: 'equalDOF', masterNode, slaveNode, dofs: [0] };
      return { ...orig, slaveNodes: [slaveNode] };
    },
  },
  yz: {
    dofMap: { 1: 0, 2: 1, 3: 2 },
    rotDof3d: 3,
    rotSign: 1,
    eccentricOffsets: (c) => ({ offsetX: c.offsetY ?? 0, offsetY: c.offsetZ ?? 0 }),
    eccentricReleases: (r) => [r[1] ?? false, r[2] ?? false, r[3] ?? false],
    diaphragmTrace: (dp, masterNode, slaveNode, orig) => {
      if (dp === 'YZ') return { type: 'rigidLink', masterNode, slaveNode, dofs: [0, 1, 2] };
      if (dp === 'XZ') return { type: 'equalDOF', masterNode, slaveNode, dofs: [1] };
      if (dp === 'XY') return { type: 'equalDOF', masterNode, slaveNode, dofs: [0] };
      return { ...orig, slaveNodes: [slaveNode] };
    },
  },
  xy: {
    dofMap: { 0: 0, 1: 1, 5: 2 },
    rotDof3d: 5,
    rotSign: 1,
    eccentricOffsets: (c) => ({ offsetX: c.offsetX ?? 0, offsetY: c.offsetY ?? 0 }),
    eccentricReleases: (r) => [r[0] ?? false, r[1] ?? false, r[5] ?? false],
    diaphragmTrace: (dp, masterNode, slaveNode, orig) => {
      if (dp === 'XY') return { type: 'rigidLink', masterNode, slaveNode, dofs: [0, 1, 2] };
      if (dp === 'XZ') return { type: 'equalDOF', masterNode, slaveNode, dofs: [0] };
      if (dp === 'YZ') return { type: 'equalDOF', masterNode, slaveNode, dofs: [1] };
      return { ...orig, slaveNodes: [slaveNode] };
    },
  },
};

/**
 * Detect which Cartesian plane a set of 3D nodes lies in.
 * Returns 'xz' (default), 'yz', or 'xy'.
 */
export function detectFramePlane(nodes: Iterable<{ x: number; y: number; z?: number }>): DrawPlane {
  let minX = Infinity, maxX = -Infinity;
  let minY = Infinity, maxY = -Infinity;
  let minZ = Infinity, maxZ = -Infinity;
  let count = 0;

  let hasZ = false;
  for (const n of nodes) {
    count++;
    if (n.x < minX) minX = n.x;
    if (n.x > maxX) maxX = n.x;
    if (n.y < minY) minY = n.y;
    if (n.y > maxY) maxY = n.y;
    if (n.z !== undefined) {
      hasZ = true;
      const z = n.z;
      if (z < minZ) minZ = z;
      if (z > maxZ) maxZ = z;
    }
  }

  if (count < 2 || !hasZ) return 'xz';

  const spanX = maxX - minX;
  const spanY = maxY - minY;
  const spanZ = maxZ - minZ;

  const tol = 1e-4;
  if (spanX <= tol && (spanY > tol || spanZ > tol)) return 'yz';
  if (spanY <= tol && (spanX > tol || spanZ > tol)) return 'xz';
  if (spanZ <= tol && (spanX > tol || spanY > tol)) return 'xy';

  return 'xz';
}

/**
 * Returns the 2D-wire shape of the given constraints for the specified drawing plane.
 * The result is what both the 2D connectivity preflight and serializeInput2D must consume.
 */
export function constraintsTo2D(constraints: Constraint3D[] | undefined, plane: DrawPlane = 'xz'): Constraint3D[] {
  if (!constraints || constraints.length === 0) return [];
  const cfg = PLANE_CONFIGS[plane] ?? PLANE_CONFIGS.xz;
  const out: Constraint3D[] = [];

  function mapDofs(dofs: number[] | undefined): number[] | undefined {
    if (!dofs) return undefined;
    return dofs
      .map(d => cfg.dofMap[d])
      .filter((d): d is number => d !== undefined);
  }

  for (const c of constraints) {
    switch (c.type) {
      case 'rigidLink': {
        // No DOFs listed: all translational, as the space solve reads it; listed for the plane
        // solver, whose own default would tie the rotation as well.
        if (!c.dofs || c.dofs.length === 0) { out.push({ ...c, dofs: [0, 1] }); break; }
        const dofs = mapDofs(c.dofs);
        if (!dofs || dofs.length === 0) break; // only out-of-plane DOFs
        out.push({ ...c, dofs });
        break;
      }
      case 'equalDOF': {
        const dofs = mapDofs(c.dofs) ?? [];
        if (dofs.length === 0) break;
        out.push({ ...c, dofs });
        break;
      }
      case 'linearMPC': {
        // An equation involving an out-of-plane DOF has no 2D meaning — drop whole.
        if (c.terms.some(t => cfg.dofMap[t.dof] === undefined)) break;
        // In-plane rotation coefficient takes cfg.rotSign
        out.push({
          ...c,
          terms: c.terms.map(t => ({
            ...t,
            dof: cfg.dofMap[t.dof],
            coefficient: t.dof === cfg.rotDof3d ? cfg.rotSign * t.coefficient : t.coefficient,
          })),
        });
        break;
      }
      case 'eccentricConnection': {
        const offsets = cfg.eccentricOffsets(c);
        out.push({
          ...c,
          offsetX: offsets.offsetX,
          offsetY: offsets.offsetY,
          offsetZ: 0,
          releases: cfg.eccentricReleases(c.releases ?? []),
        });
        break;
      }
      case 'diaphragm': {
        // Its trace on the model plane: per slave, an equal DOF or a rigid link.
        const diaphragmPlane = (c.plane ?? 'XY').toUpperCase();
        for (const s of c.slaveNodes) {
          if (s === c.masterNode) continue;
          out.push(cfg.diaphragmTrace(diaphragmPlane, c.masterNode, s, c));
        }
        break;
      }
    }
  }
  return out;
}
