### Added
#### Native and server engine build, cross-platform determinism and large-model runtime (#486)

Closes #486 (roadmap:solver-performance, tracker #396). Provides one unified structural engine
across browser, desktop, and server:
- Added `stabileo-engine` standalone CLI and HTTP RPC daemon binary target in `engine/Cargo.toml`
  and `engine/src/bin/stabileo_engine.rs`, supporting `--solve`, `--server`, `--sanity`, and
  `--build-info`.
- Added cross-platform bit-identical determinism test suite in
  `engine/tests/cross_platform_determinism.rs` ensuring identical results across native and WASM.
- Documented the large-model runtime architecture, browser memory limits, and server offload path
  in `docs/architecture/SOLVER_RUNTIME.md`.
- Enforced native and server binary builds in `.github/workflows/ci.yml` and `Makefile`.
