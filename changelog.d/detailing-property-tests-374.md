### Fixed
#### Detailing property invariants and nightly Stryker hardening (#374, 2026-10-28)

Property tests for physical detailing invariants (`src/lib/engine/detailing/__tests__/detailing-invariants.property.test.ts`)
verify across randomized physical beam and column configurations that:
- As provided >= As required and As >= As,min across span sagging and support hogging stations.
- Required concrete cover is satisfied without breaches (`checkCover`) and clear horizontal and vertical spacing satisfy CIRSOC 201 §25.2.
- 3D physical bar generation produces zero overlap conflicts (`detectCollisions`), eliminating head-on collisions when top bars from opposite supports meet in the span.
- Bar cutoffs extend past theoretical points by d or 12 db (§9.7.3.3) and column splices satisfy Class A/B lap lengths (§25.5.2).

The remaining TypeScript detailing modules (`collision.ts`, `support-anchorage.ts`, `constructibility.ts`, `generate-beam.ts`) are added to `stryker.config.json` and `vitest.mutation.config.ts`, with surviving mutants triaged in `docs/verification/TS_ENGINE_LOGIC.md`.
