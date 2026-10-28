# Safety Postmortems

Every `safety` defect that reached `main` gets a blameless postmortem in `docs/postmortems/YYYY-MM-DD-<slug>.md` within **10 working days of the fix**.

Each postmortem analyzes:
1. **Timeline**: Introduced, released, found, contained, fixed.
2. **Root Cause**: The exact mechanical or algorithmic defect.
3. **Why existing tests missed it**: Why unit tests, certificates, differential suites, and code reviews failed to detect it.
4. **New kind of test or check added**: The new automated test class now enforced in CI to catch this bug class forever.
5. **Variant analysis (#448)**: A codebase-wide audit searching for the same bug class across all other solvers, elements, and design checks.
6. **Advisory link**: Bidirectional link to the corresponding public advisory in [`docs/advisories/`](../advisories/README.md).

Template: [`TEMPLATE.md`](TEMPLATE.md).

---

## Postmortems Index

| Date | Defect / Title | Severity | Advisory | Issue / Fix | Test kind added |
|---|---|---|---|---|---|
| 2026-10-21 | [Mirrored moments in 2D-in-3D models](2026-10-21-mirrored-moments-2d-in-3d.md) | High | [SA-2026-006](../advisories/SA-2026-006-mirrored-moments-2d-in-3d.md) | [#435](https://github.com/lambdaclass/stabileo/issues/435) / [#436](https://github.com/lambdaclass/stabileo/pull/436) | Metamorphic coordinate transform invariant test |
| 2026-10-21 | [Spectral certificate passing on non-finite values](2026-10-21-spectral-certificate-nonfinite.md) | Medium | [SA-2026-010](../advisories/SA-2026-010-spectral-certificate-nonfinite.md) | [#335](https://github.com/lambdaclass/stabileo/issues/335) / [#398](https://github.com/lambdaclass/stabileo/pull/398) | Property fuzzing with IEEE NaN/Inf injection into certificate gates |
| 2026-10-21 | [Inclined member loads missing axial projection](2026-10-21-inclined-member-loads.md) | High | [SA-2026-005](../advisories/SA-2026-005-inclined-member-loads.md) | [#343](https://github.com/lambdaclass/stabileo/issues/343) / [#437](https://github.com/lambdaclass/stabileo/pull/437) | Differential equilibrium balance & rotation invariance harness |
| 2026-10-24 | [RC column check evaluating single moment sign](2026-10-24-column-moment-sign.md) | Critical | [SA-2026-023](../advisories/SA-2026-023-column-moment-sign.md) | D4-01 / [#590](https://github.com/lambdaclass/stabileo/pull/590) | Asymmetric cross-section bidirectional sign sweep test |
