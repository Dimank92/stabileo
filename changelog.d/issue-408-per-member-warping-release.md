### Added

#### Per-member warping releases (`release_warp_start`, `release_warp_end`) for 3D frames (#408)

Added member-end warping releases (`release_warp_start` and `release_warp_end`) to `SolverElement3D` in the 3D frame input, defaulting to `false` (preserving full warping continuity across joints). When released, static condensation in `torsion_warping_block` zeroes bimoment transmission at the joint while preserving St. Venant and warping torsion stiffness. Resolves FR-5.
