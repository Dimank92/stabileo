### Fixed
#### Modal analysis refuses mechanisms with singular stiffness (#338, 2026-10-28)

Modal analysis (both 2D and 3D) now refuses with `"Singular stiffness matrix — structure is a mechanism"` instead of silently dropping rigid-body mechanism modes ($\lambda \le 10^{-7}$) or failing generically with `"Eigenvalue decomposition failed"`.

