### Fixed
#### Modal and static analysis refuse mechanisms with singular stiffness or hinged free tips (#338, 2026-10-28)

Modal and static analyses (both 2D and 3D) now refuse with `"Singular stiffness matrix — structure is a mechanism"` when encountering mechanisms such as hinged members with free tips, instead of silently dropping rigid-body mechanism modes or yielding runaway displacements.


