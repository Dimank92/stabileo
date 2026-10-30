export const MESHES = {
  coarse: { label: 'coarseMesh', mesh: { nx: 40, ny: 40, nx_fine: 16, ny_fine: 8, h_fine: 0.0025 } },
  medium: { label: 'mediumMesh', mesh: { nx: 50, ny: 60, nx_fine: 24, ny_fine: 12, h_fine: 1 / 600 } },
  reference: { label: 'referenceMesh', mesh: { nx: 70, ny: 80, nx_fine: 32, ny_fine: 16, h_fine: 0.00125 } },
} as const;

export type MeshKey = keyof typeof MESHES;

export const DEFAULTS = {
  e: 210,
  nu: 0.3,
  gc: 0.0027,
  l0: 0.0075,
  u: 0.01,
  dt: 0.01,
  mesh: 'reference' as MeshKey,
  history: false,
};

export const LIMITS = {
  e: [10, 400],
  nu: [0, 0.45],
  gc: [1e-4, 1e-2],
  l0: [0.003, 0.03],
  u: [0.004, 0.02],
} as const;

export const inRange = (v: number, [lo, hi]: readonly [number, number]) =>
  Number.isFinite(v) && v >= lo && v <= hi;
