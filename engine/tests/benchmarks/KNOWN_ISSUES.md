# Benchmark accuracy findings

This test-owned ledger records accuracy failures without changing runtime diagnostics.
The production registry is [`engine/src/known_issues.rs`](../../src/known_issues.rs).
It currently has no open shell entry. These findings need solver-owner triage before any
runtime registry entry is added; no runtime issue number is implied here.

## MH-MITC4-DISTORTION

On the MacNeal–Harder straight cantilever (6 × 1, full root clamp), production MITC4 misses
the page's strict 2% accuracy target for both trapezoidal and parallelogram meshes:

| Mesh | Load | Reference | Computed | Relative error |
|---|---|---:|---:|---:|
| Trapezoidal | Out of plane | 0.4321 | 0.4162053612497649 | 3.678463% |
| Trapezoidal | In plane | 0.1081 | 0.008443013508022026 | 92.189627% |
| Parallelogram | Out of plane | 0.4321 | 0.42257240188170075 | 2.204952% |
| Parallelogram | In plane | 0.1081 | 0.010379694543545988 | 90.398062% |

Native and production WASM agree. Reproduce with `shells macneal_harder::browser_cases`
and `research/benchmarks/macneal-harder-1985/wasm/check.mjs`. Inputs, references and the
accuracy threshold are in `macneal_harder.json`. These measurements are evidence of the
specified mesh's accuracy, not proof of a new implementation defect. Existing solid-shell
regression floors document related distortion sensitivity and remain unchanged.

## MH-REFERENCE-PROVENANCE

Geometry of the straight cantilever is Fig. 4, p. 7; patch geometry is Fig. 2, p. 5, and
patch boundary fields are Table 2, p. 10. A preview of the paper supports these locators.
The numerical tables were not accessible. Values 0.4321 and 0.1081 retain provenance from
`solid_shell_locking.rs`; verify the numerical table/page against the full paper before
claiming a verified transcription. Frame cases are explicitly analytical adaptations.

## MATTIASSON-REFERENCE-AND-ACCURACY

The Mattiasson page remains **proposed**. Its four reference pairs are copied
unchanged from `BENCHMARK_REFERENCES.md`, which says to verify them against the
original paper. The Table 1 attribution and numerical transcription are not
verified. Citation metadata was checked through the catalogue checker; this is
not a check of published numbers.

Fresh production native results (L = 2, 16 elements, settings in
`mattiasson_elastica.json`):

| PL²/EI | Reference u/L, v/L | Computed u/L, v/L | Relative errors | Page classification |
|---:|---|---|---|---|
| 1 | 0.0566, 0.3015 | 0.0557827218712, 0.302243545400 | 1.443954%, 0.246615% | Pass (provisional) |
| 2 | 0.1632, 0.4929 | 0.159191217091, 0.495808283573 | 2.456362%, 0.590035% | Fail |
| 5 | 0.4054, 0.6647 | 0.385315745120, 0.722972888604 | 4.954182%, 8.766795% | Fail |
| 10 | 0.5584, 0.6854 | Not computed at requested load | — | Fail: nonconvergence, load factor 0.1 |

Native and production WASM agree on all 21 cases, including every nodal
displacement and rotation for complete solves. Parameters 7.5 and 9.5 also fail
to converge; the curves show gaps, never a line through a missing result.

For parameter 10 the solver returns a last-converged state at 10% of the requested
load. The page intentionally does not plot or compare that state at parameter 10.
The page target is strict relative error below 2% in **both signed components**;
it is a new exploratory comparison, not an existing native accuracy assertion.
Every existing native assertion, tolerance and regression floor is unchanged.
The moment regression (ML/EI = 1) converges with u/L = 0.158392041418 and
v/L = -0.459772523245 under the page's downward-positive convention. It has no
paper accuracy reference and does not demonstrate a full circle.

Do not infer a solver defect solely from discrepancies with these unverified
references. Confirm paper transcription, load convention and extensibility
assumptions; then ask the solver owner to triage accuracy and convergence.
The production runtime registry has not been changed and no runtime issue ID is
claimed. Reproduce via `research/benchmarks/mattiasson-elastica/README.md`.
