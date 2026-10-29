### Fixed
#### Mutation test gap in certificate re-assembly closed (2026-10-29)

Add comprehensive tests for all surviving mutants in the certificate's independent re-assembly modules (`ia.rs`, `element.rs`, `mod.rs`, `model2d.rs`, `model3d.rs`, `system.rs`), covering boundary conditions, numerical zero handling in norm3, agreement metric calculations, and exclude equivalent mutants with one-line justifications in `.cargo/mutants.toml` (#749).
