### Fixed
#### Include warping DOFs in 3D modal mass-orthogonality verification (#759)

The metamorphic modal verification check omitted the 7th DOF (`warping`) when reconstructing mode shapes from 3D displacements. Models with thin-walled warping members evaluated mass products against a truncated vector, producing false mass-orthogonality failures.

