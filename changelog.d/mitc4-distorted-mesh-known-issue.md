### Added
#### Containment: known-issue warning on distorted MITC4 shell meshes (2026-10-29)

MITC4 four-node quad shells undergo severe in-plane shear locking under bending when distorted
(MacNeal–Harder straight cantilever: −92 % on trapezoidal meshes, −90 % on parallelogram meshes).
Until an enhanced-membrane formulation is integrated, any model containing distorted MITC4
quads carries the known-issue warning `SH-17` with the distorted element IDs and instructions
to refine to near-rectangular elements (#798).

