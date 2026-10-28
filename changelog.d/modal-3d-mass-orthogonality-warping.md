### Fixed
#### Include warping degrees of freedom in 3D modal mass-orthogonality verification (#759)

Included the 7th warping degree of freedom when reconstructing modal displacement vectors in the metamorphic modal orthogonality check. Previously, models with warping sections had their warping coordinates omitted during mode reconstruction, projecting 7-DOF mass-orthogonal modes onto 6-DOF subspaces where they appeared non-orthogonal or duplicate.

