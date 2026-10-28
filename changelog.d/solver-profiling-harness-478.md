### Added

#### Solver profiling harness and performance tracking (#478)

* Added `scripts/solver-profile.py` benchmarking solver performance across acceptance models and synthetic frame, shell, and building models up to $10^5$–$10^6$ DOFs.
* Tracked per-phase timings (assembly, ordering, factorization, solve, postprocessing, certificates), peak resident memory, and iteration counts.
* Integrated regression gate (`--max-time-ms`) to prevent large algorithmic slowdowns in CI and local workflows.
* Integrated solver profiling metrics into `scripts/verification-metrics.py` and the verification dashboard.
* Published baseline profiling report identifying dominant costs in `docs/performance/SOLVER_PROFILING.md`.
