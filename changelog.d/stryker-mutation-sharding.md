### Changed
#### CI: shard TypeScript mutation testing into matrix jobs and deduplicate alert keys (#521)

- `.github/workflows/mutation-ts.yml`: sharded `stryker` mutation testing into 4 balanced matrix shards (`cirsoc-codes`, `detailing`, `design-engine`, `cirsoc-spacing`) to prevent single-runner 345-minute timeouts and job cancellations.
- `web/vitest.mutation.config.ts`: excluded heavyweight multi-member building autodesign benchmarks (`autodesign-regression`, `settle-not-clocked`, `beam-line-flagship`) from mutant runs to keep dry-run and per-mutant cycles focused on clause-level unit tests.
- `.github/scripts/verification-alert.cjs`: updated `dedupeKey` to map `stryker (...)` matrix shards back to `stryker` so nightly mutation alerts cleanly group under a single issue.
