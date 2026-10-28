# Postmortem: RC column check evaluating single moment sign

> **Status:** Published  
> **Date:** 2026-10-24  
> **Author:** Stabileo Verification Team  
> **Severity:** Critical  
> **Advisory:** [SA-2026-023](../advisories/SA-2026-023-column-moment-sign.md)  
> **Issue:** D4-01  
> **Fix PR:** [#590](https://github.com/lambdaclass/stabileo/pull/590)  

---

## 1. Executive Summary

Reinforced concrete column capacity checks (CIRSOC 201 / ACI 318) evaluated cross-sectional bending capacity using only the positive sign of the design bending moment ($+M_u$). For columns with asymmetric longitudinal rebar configurations (such as uneven bar distributions across faces: 6, 10, 14 bars, or corner-concentrated reinforcement), the capacity in the negative bending direction is significantly lower. The check erroneously certified columns as safe based on the stronger direction, overstating design capacity $\phi M_n$ by up to 38 %.

---

## 2. Timeline

| Time / Date | Event |
|---|---|
| **2026-05-18** | **Introduced**: P-M interaction diagram evaluator merged in PR #89 |
| **2026-09-01** | **Released**: Shipped in v0.2.0 |
| **2026-10-22** | **Found**: Identified in derivation review audit (item D4-01) |
| **2026-10-23** | **Contained**: Advisory SA-2026-023 drafted, release blocker filed |
| **2026-10-24** | **Fixed**: PR [#590](https://github.com/lambdaclass/stabileo/pull/590) merged to `main` |

---

## 3. Root Cause

The P-M interaction diagram generator constructed the envelope in the first quadrant $(P \ge 0, M \ge 0)$, assuming that all reinforced concrete columns have symmetric cross-sections ($I_y = I_z$, symmetric top/bottom and left/right rebar layers). When checking design demand $(P_u, M_u)$, the algorithm evaluated:
$$\text{utilization} = \frac{|M_u|}{\phi M_n(P_u, \theta = 0^\circ)}$$
For asymmetric bar layouts, $\phi M_n(P_u, 0^\circ) \ne \phi M_n(P_u, 180^\circ)$. Evaluating only $\theta = 0^\circ$ tested only one bending direction, ignoring the critical weaker face.

---

## 4. Why Existing Tests Missed It

- **Unit & Integration Tests**: All regression tests and textbook examples in the test suite utilized 4-bar or 8-bar symmetric column sections.
- **Verification Certificates & Proofs**: Design checks were not part of the Lean certificate proof (which covered structural analysis equilibrium, not code compliance).
- **Differential Runs & Benchmarks**: Differential benchmarks compared against symmetric benchmark cases from ACI SP-17 and PCA notes.
- **Code Review**: The assumption of section symmetry was implicit throughout the helper functions and went unnoticed.

---

## 5. New Kind of Test or Check Added

- **Check Name / Location**: `web/src/lib/engine/__tests__/differential/column-asymmetric-sweep.test.ts`
- **Test Kind**: Asymmetric cross-section bidirectional sign sweep test. Evaluates full 360-degree interaction surfaces for asymmetric bar arrangements, testing both $+M_u$ and $-M_u$ across all axial load levels.
- **Negative Case**: Evaluating an asymmetric 6-bar column with design load $(P_u, -M_u)$ under a single-sign check produces an incorrect pass, whereas the sweep test strictly requires failure.

---

## 6. Variant Analysis (#448)

- **Search Query / Method**: Audit of all design code check implementations for assumed geometric symmetry: `grep -rn "abs(" web/src/lib/engine/design/`.
- **Other Components Examined**: Flanged T-beams, footing punching shear perimeters, composite steel-concrete sections, retaining wall stem design.
- **Additional Deficiencies Found & Fixed**: Corrected sign-dependent tension face identification in CIRSOC flanged beam checks.

---

## 7. Lessons Learned & Action Items

- [x] Never assume cross-sectional or material symmetry in design check routines.
- [x] Require all design check test suites to include explicitly asymmetric geometry test cases.
