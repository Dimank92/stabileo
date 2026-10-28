### Changed
#### Dependency upgrades through the same gates; numerics crates must stay bit-identical (#388, 2026-10-28)

Dependency upgrades (including Dependabot PRs) now pass through the same required CI gates as code changes. Added `scripts/check-numerics-deps.py` (`make check.numerics-deps`) to enforce zero duplicate versions among exact arithmetic and geometry kernels (`num-bigint`, `num-rational`, `num-traits`, `num-integer`, `spade`). In `.github/dependabot.yml`, numerics crates are grouped together to prevent broken uncoordinated bumps. In CI, dependency configuration files (`deny.toml`, `.github/dependabot.yml`, `supply-chain.yml`) are added to tooling path filters, and major upgrades of numerics crates require bit-identical results on the golden fixture corpus (`determinism::golden_corpus_fixtures_are_bit_identical_across_solves`).

