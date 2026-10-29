### Fixed
#### Mode load factors refined via Rayleigh quotient (2026-10-29)

The reciprocal eigenvalue solver in 2D and 3D buckling analysis now refines mode load factors via the Rayleigh quotient on the assembled stiffness and geometric stiffness matrices. This eliminates the small numerical overestimation (0.05% in issue #741's case) caused by dense Cholesky inversion and Jacobi diagonalization, guaranteeing that reported buckling load factors fall strictly within their certified enclosures and achieve relative error under 1e-6. Issue #818.

