# Postmortem: Inclined member loads missing axial projection

> **Status:** Published  
> **Date:** 2026-10-21  
> **Author:** Stabileo Verification Team  
> **Severity:** High  
> **Advisory:** [SA-2026-005](../advisories/SA-2026-005-inclined-member-loads.md)  
> **Issue:** [#343](https://github.com/lambdaclass/stabileo/issues/343)  
> **Fix PR:** [#437](https://github.com/lambdaclass/stabileo/pull/437)  

---

## 1. Executive Summary

When distributed loads or member couples were applied to inclined frame members in 2D and 3D models, the fixed-end force (FEF) transformation projected only the transverse component of global gravity loads onto local member shear/bending, omitting the longitudinal projection that produces member axial thrust. In 3D, member couples were silently dropped from fixed-end load assemblies, resulting in unconservatively low internal axial loads and missing moments at adjacent joints.

---

## 2. Timeline

| Time / Date | Event |
|---|---|
| **2026-06-11** | **Introduced**: Fixed-end force equivalent calculation merged in PR #114 |
| **2026-09-01** | **Released**: Deployed in release v0.2.0 |
| **2026-10-08** | **Found**: Reported in [#343](https://github.com/lambdaclass/stabileo/issues/343) during truss-arch validation |
| **2026-10-10** | **Contained**: Advisory draft created, user warning documented |
| **2026-10-21** | **Fixed**: PR [#437](https://github.com/lambdaclass/stabileo/pull/437) merged and released in v0.3.0 |

---

## 3. Root Cause

Fixed-end actions for a member subject to distributed load $\mathbf{w}$ in global coordinates require projecting $\mathbf{w}$ into both the member local axial vector $\mathbf{x}_m$ and local transverse axes $\mathbf{y}_m, \mathbf{z}_m$:
$$w_{\text{axial}} = \mathbf{w} \cdot \mathbf{x}_m, \quad w_{\text{trans}} = \mathbf{w} - w_{\text{axial}}\mathbf{x}_m$$
The implementation computed only the transverse component for beam bending and assumed axial loads were applied solely as concentrated nodal forces. Additionally, the assembly dispatch loop omitted the branch handling applied member couples in 3D.

---

## 4. Why Existing Tests Missed It

- **Unit & Integration Tests**: Test cases for member loads tested only horizontal and vertical members where global axes coincided with local axes.
- **Verification Certificates & Proofs**: Nodal equilibrium $\sum \mathbf{F}_{\text{ext}} = \sum \mathbf{F}_{\text{int}}$ was satisfied at nodes because the missing fixed-end nodal reactions were never added to the global load vector in the first place, keeping mathematical equilibrium self-consistent but physically incomplete.
- **Differential Runs & Benchmarks**: Benchmarks compared nodal load cases against Frame3DD and did not have an inclined span with distributed gravity in the automated differential suite.
- **Code Review**: The projection formulas looked mathematically plausible because the transverse transformation was correct.

---

## 5. New Kind of Test or Check Added

- **Check Name / Location**: `engine/tests/metamorphic/member_loads.rs`
- **Test Kind**: Differential equilibrium balance and rotation invariance harness. Tests an inclined member against an equivalent subdivided frame with explicit nodal forces, verifying identical reactions, shear, and axial forces across all inclination angles $(0^\circ \dots 90^\circ)$.
- **Negative Case**: Tampering with the axial load term in `fef_2d` or `fef_3d` causes `test_inclined_member_fef_axial_invariance` to fail immediately.

---

## 6. Variant Analysis (#448)

- **Search Query / Method**: Systematic audit of all member load types: trapezoidal, thermal gradient, partial distributed, and point loads on members.
- **Other Components Examined**: Plate distributed loads, thermal loading in shells, Timoshenko shear corrections.
- **Additional Deficiencies Found & Fixed**: Added handling for member couples across all 3D member formulations, and unified trapezoidal distributed load FEF integrations.

---

## 7. Lessons Learned & Action Items

- [x] Every member load feature must be tested on arbitrary non-orthogonal orientations $(30^\circ, 45^\circ, 60^\circ)$.
- [x] Enforce automated differential checks comparing member loads against equivalent fine-mesh nodal discretizations.
