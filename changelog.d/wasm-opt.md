### Performance
#### The engine ships optimized: 25 % smaller, same results bit for bit (2026-10-23)

The deployed WASM engine was built with `--no-opt` (12.0 MB). It is now optimized with
`wasm-opt` (9.0 MB). CI checks every run that the optimized and the unoptimized builds give the
same results on the 184 determinism cases, bit for bit, so the optimizer cannot change a number
without failing the build.
