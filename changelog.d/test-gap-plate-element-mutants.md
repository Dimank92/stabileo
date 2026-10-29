### Fixed
#### Close mutation test gap in plate element kernels (2026-10-29)

Added comprehensive unit tests covering 133 surviving mutants across 12 functions in plate element kernels (`engine/src/element/plate.rs`): CST membrane B-matrix and stiffness, DKT bending geometry, B-matrix, and stiffness, drilling stabilization, 3D coordinate transformations, pressure and thermal loads, stress recovery at centroid and nodes, and DKMT transverse shear stiffness (#746).

