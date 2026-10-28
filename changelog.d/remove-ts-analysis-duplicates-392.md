### Fixed
#### Analysis results from the engine: remove TypeScript analysis duplicates (#392, 2026-10-28)

Legacy penalty-beam rigid diaphragm preprocessor `applyRigidDiaphragm` moved from production `rigid-diaphragm.ts` to `__tests__/differential/oracle/rigid-diaphragm-oracle.ts` as an isolated test oracle. Multi-point constraint solving is performed natively by the Rust engine (`solver/constraints.rs Constraint::Diaphragm`). Enforced with a strict lint guard in `concrete-oracles-are-test-only.test.ts` ensuring no production module imports or re-defines `applyRigidDiaphragm`. Added differential parity test `rigid-diaphragm.test.ts` verifying floor detection, centroid calculation, and penalty beam element generation against the geometric model.

