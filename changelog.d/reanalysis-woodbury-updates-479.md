### Added

#### Solver re-analysis, Sherman–Morrison–Woodbury low-rank updates, and multi-RHS solves (#479)

* Implemented `ReAnalysisSession3D` in `engine/src/solver/reanalysis.rs` supporting cached factorizations across localized structural edits.
* Added multi-RHS solve pipelining (`solve_multi_rhs`) reusing global Cholesky decompositions across load combinations with $O(\text{nnz}(L))$ triangular substitution time.
* Added low-rank Sherman–Morrison–Woodbury updates for element cross-section and nodal support modifications with automated equilibrium residual checks and full re-solve fallback.
* Published architectural specification `docs/architecture/REANALYSIS.md`.
* Added parity verification test `test_reanalysis_session_3d_parity_and_multi_rhs` in `engine/tests/core/parity.rs`.
