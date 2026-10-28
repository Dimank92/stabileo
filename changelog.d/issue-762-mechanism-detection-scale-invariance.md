### Fixed
#### Invariant mechanism detection and accuracy on near-singular models (2026-10-28)

Relative pivot thresholds scaled to each degree-of-freedom's diagonal stiffness
in dense Cholesky, sparse Cholesky, and dense LU eliminate artificial sensitivity
to unit conversions, rigid body rotations, and mirror transformations on near-singular
models (#762).

