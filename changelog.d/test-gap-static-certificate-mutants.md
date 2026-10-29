### Fixed
#### Close mutation test gap in static certificate support code (2026-10-29)

Added comprehensive unit tests covering 244 surviving mutants across 50 functions in static certificate support code: station intervals and envelope kinks (stations.rs), deflection factors and cubic bounding (deflection.rs), P-Delta Kantorovich radius and Newton iteration steps (pdelta.rs), real interval math, roots, and trigonometry (real.rs), local member stiffness and row bounds (forces.rs), constrained DOFs and symmetric matrix parts (constrained.rs), finite arithmetic validation and path descriptions (finite.rs), solver end-to-end certification guards (solver.rs), and exact rational bounds (exact.rs), with equivalent mutant exclusions documented in .cargo/mutants.toml (#751).
