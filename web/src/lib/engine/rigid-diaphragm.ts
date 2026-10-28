// Rigid Diaphragm Utilities
// Pure geometric helpers for floor level detection and floor properties.
// Diaphragm constraint solving is performed natively by the Rust engine
// (`solver/constraints.rs Constraint::Diaphragm`).
// The legacy penalty-beam approximation is kept only as a test oracle
// (`__tests__/differential/oracle/rigid-diaphragm-oracle.ts`).

import type { SolverNode3D } from './types-3d';

export interface DiaphragmLevel {
  z: number;            // floor elevation (m)
  nodeIds: number[];    // nodes at this level
  masterNodeId: number; // center of mass node
  tolerance: number;    // z-tolerance for grouping
}

export interface DiaphragmConfig {
  levels: number[];           // Z elevations to apply diaphragm
  tolerance?: number;         // default 0.05m (5cm) for grouping
  stiffnessMultiplier?: number;
}

/**
 * Detect floor levels by grouping nodes at similar Z coordinates
 */
export function detectFloorLevels(
  nodes: Map<number, SolverNode3D>,
  tolerance: number = 0.05,
): number[] {
  const zValues: number[] = [];
  for (const n of nodes.values()) {
    const z = n.z ?? 0;
    if (!zValues.some(zv => Math.abs(zv - z) < tolerance)) {
      zValues.push(z);
    }
  }
  zValues.sort((a, b) => a - b);
  // Return levels with more than 1 node (exclude base)
  return zValues.filter(z => {
    let count = 0;
    for (const n of nodes.values()) {
      if (Math.abs((n.z ?? 0) - z) < tolerance) count++;
    }
    return count >= 2;
  });
}

/**
 * Find the centroid (center of mass) of nodes at a given Z level
 */
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
 * Get center of rigidity and center of mass for each floor level
 * (useful for eccentricity calculations per CIRSOC 103)
 */
export function getFloorProperties(
  nodes: Map<number, SolverNode3D>,
  levels: number[],
  tolerance: number = 0.05,
): Array<{ z: number; centerOfMass: { x: number; y: number }; nodeCount: number }> {
  return levels.map(z => {
    const cm = findCentroid(nodes, z, tolerance);
    let count = 0;
    for (const n of nodes.values()) {
      if (Math.abs((n.z ?? 0) - z) < tolerance) count++;
    }
    return { z, centerOfMass: cm, nodeCount: count };
  });
}
