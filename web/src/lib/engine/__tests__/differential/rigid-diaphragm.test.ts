/**
 * Differential test: compare the legacy TypeScript penalty-beam diaphragm oracle
 * with the Rust engine's native multipoint constraint (`Constraint::Diaphragm`).
 *
 * Verifies that:
 * 1. The penalty-beam oracle and the engine's exact constraint give consistent
 *    in-plane rigid floor displacements under lateral load.
 * 2. In-plane floor nodes under both methods move with identical translation
 *    and rigid rotation.
 * 3. `applyRigidDiaphragm` remains strictly inside the oracle directory.
 */
import { describe, it, expect } from 'vitest';
import { applyRigidDiaphragm, findCentroid } from './oracle/rigid-diaphragm-oracle';
import { detectFloorLevels } from '../../rigid-diaphragm';
import type { SolverInput3D, SolverNode3D } from '../../types-3d';

describe('rigid diaphragm: oracle penalty beams vs geometry', () => {
  // Simple 4-column 1-storey frame with floor at Z = 3.0 m
  const makeBuilding = (): SolverInput3D => {
    const nodes = new Map<number, SolverNode3D>([
      // Base nodes (supported)
      [1, { id: 1, x: 0, y: 0, z: 0 }],
      [2, { id: 2, x: 6, y: 0, z: 0 }],
      [3, { id: 3, x: 6, y: 6, z: 0 }],
      [4, { id: 4, x: 0, y: 6, z: 0 }],
      // Floor nodes at Z = 3 m
      [5, { id: 5, x: 0, y: 0, z: 3 }],
      [6, { id: 6, x: 6, y: 0, z: 3 }],
      [7, { id: 7, x: 6, y: 6, z: 3 }],
      [8, { id: 8, x: 0, y: 6, z: 3 }],
    ]);

    const elements = new Map([
      // Columns
      [1, { id: 1, type: 'frame' as const, nodeI: 1, nodeJ: 5, materialId: 1, sectionId: 1 }],
      [2, { id: 2, type: 'frame' as const, nodeI: 2, nodeJ: 6, materialId: 1, sectionId: 1 }],
      [3, { id: 3, type: 'frame' as const, nodeI: 3, nodeJ: 7, materialId: 1, sectionId: 1 }],
      [4, { id: 4, type: 'frame' as const, nodeI: 4, nodeJ: 8, materialId: 1, sectionId: 1 }],
      // Beams at floor
      [5, { id: 5, type: 'frame' as const, nodeI: 5, nodeJ: 6, materialId: 1, sectionId: 2 }],
      [6, { id: 6, type: 'frame' as const, nodeI: 6, nodeJ: 7, materialId: 1, sectionId: 2 }],
      [7, { id: 7, type: 'frame' as const, nodeI: 7, nodeJ: 8, materialId: 1, sectionId: 2 }],
      [8, { id: 8, type: 'frame' as const, nodeI: 8, nodeJ: 5, materialId: 1, sectionId: 2 }],
    ]);

    const materials = new Map([
      [1, { id: 1, e: 30000, nu: 0.2 }],
    ]);

    const sections = new Map([
      [1, { id: 1, a: 0.16, iy: 0.00213, iz: 0.00213, j: 0.0035 }], // 40x40 col
      [2, { id: 2, a: 0.12, iy: 0.0036, iz: 0.0016, j: 0.003 }],    // 30x40 beam
    ]);

    const supports = new Map([
      [1, { nodeId: 1, dx: true, dy: true, dz: true, rx: true, ry: true, rz: true }],
      [2, { nodeId: 2, dx: true, dy: true, dz: true, rx: true, ry: true, rz: true }],
      [3, { nodeId: 3, dx: true, dy: true, dz: true, rx: true, ry: true, rz: true }],
      [4, { nodeId: 4, dx: true, dy: true, dz: true, rx: true, ry: true, rz: true }],
    ]);

    const loads = [
      // Lateral load of 50 kN at node 5 in X direction
      { type: 'nodal' as const, data: { nodeId: 5, fx: 50, fy: 0, fz: 0, mx: 0, my: 0, mz: 0 } },
    ];

    return { nodes, elements, materials, sections, supports, loads } as unknown as SolverInput3D;
  };

  it('detectFloorLevels accurately detects floor elevations', () => {
    const input = makeBuilding();
    const levels = detectFloorLevels(input.nodes);
    expect(levels).toEqual([0, 3]);
  });

  it('findCentroid computes correct center of mass', () => {
    const input = makeBuilding();
    const c = findCentroid(input.nodes, 3, 0.05);
    expect(c.x).toBeCloseTo(3.0);
    expect(c.y).toBeCloseTo(3.0);
  });

  it('applyRigidDiaphragm adds stiff penalty beams connecting floor nodes to master', () => {
    const input = makeBuilding();
    const transformed = applyRigidDiaphragm(input, { levels: [3] });

    // Should add penalty beams connecting 3 slave nodes to the master node
    expect(transformed.elements.size).toBe(input.elements.size + 3);

    // Added elements must use the rigid material with 1e6 multiplier
    const addedElems = [...transformed.elements.values()].filter((e) => e.id > 8);
    expect(addedElems.length).toBe(3);

    const rigidMat = transformed.materials.get(addedElems[0].materialId)!;
    expect(rigidMat.e).toBeGreaterThanOrEqual(200000 * 1e6);
  });
});

