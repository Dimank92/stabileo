### Changed
#### Lift tower-3d-2 modal motion check quarantine in E2E sweep (2026-10-27)

Lifts the `@flaky` quarantine on `tower-3d-2` in the Basic advanced sweep (`web/e2e/basic-advanced-sweep.ts`). The mode shape motion check has been verified stable on CI following the full-oscillation-period sampling update in `motion()`, resolving intermittent extremum delta underruns (#607).
