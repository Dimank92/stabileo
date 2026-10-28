### Added
#### Adaptive refinement to a target accuracy (#483, 2026-10-28)

Implemented goal-driven adaptive mesh refinement (`engine/src/postprocess/discretization/adaptive.rs`)
driven by the ZZ-SPR a-posteriori discretization error estimator. Supports user-specified target accuracy,
strict stopping criteria (max iterations, max DOFs limit), Dörfler bulk marking, and elevation of marked quads
to higher-order MITC9 elements (p-refinement), achieving target error with significantly fewer DOFs than uniform
refinement on benchmarks (Scordelis–Lo barrel vault, plates). Unestimable and inconclusive cases are reported honestly.
