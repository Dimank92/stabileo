### Fixed
#### Engine WASM build is reproducible bit-for-bit (2026-10-29)

Two clean builds of the Rust engine for WebAssembly now produce byte-identical
`dedaliano_engine_bg.wasm` binaries (#771). The root workspace specifies the release
profile with `codegen-units = 1` and LTO, `scripts/wasm-cached.sh` remaps workspace,
Cargo and toolchain paths via `--remap-path-prefix`, and pins `SOURCE_DATE_EPOCH` to
the commit timestamp. A CI gate (`make check.wasm-reproducible`) verifies bit-for-bit
equality on every run.
