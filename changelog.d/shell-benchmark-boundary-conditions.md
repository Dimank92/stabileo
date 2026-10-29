### Fixed
#### Shell obstacle course benchmark boundary conditions and consistent loading (2026-10-29)

Fix Scordelis–Lo barrel vault and pinched cylinder obstacle benchmarks by replacing incomplete per-edge support insertions with unified node boundary condition combinations and consistent quad surface self-weight. This eliminates the artificial midspan crown pin and missing corner symmetry restraints that produced the -15.5% roof plateau and pinched cylinder error growth with refinement (#810).
