### Fixed
#### Metamorphic nightly ignores unconfigured probe files and respects zero-scale noise floor (2026-10-27)

Nightly verification no longer panics when `probe_units_3d` and `probe_plastic_file` run under `--ignored` without `PFILE` set. The metamorphic comparator now includes a 1e-15 absolute floor in `compare_floored` to prevent false-positive violations from floating-point noise on zero-scale components.
