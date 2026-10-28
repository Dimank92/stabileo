### Added
#### Engine code hygiene: workspace overflow checks, unsafe forbid, and hygiene gate (2026-10-28)

Enforces engine code hygiene across the workspace (#383):
- Workspace root `Cargo.toml` configures `overflow-checks = true` for `release`, `bench`, and `test` profiles so index and stride arithmetic never silently wrap on overflow in optimized builds.
- Removed ignored child profile definitions from `engine/Cargo.toml`.
- Added `#![forbid(unsafe_code)]` to `engine/src/lib.rs` ensuring compiler-level enforcement of memory safety across all solver and element modules.
- Added `scripts/check-engine-hygiene.py` and `make check.engine.hygiene` verifying coverage (llvm-cov), unsafe / Miri readiness, overflow checks, cargo audit, and clippy gates.

