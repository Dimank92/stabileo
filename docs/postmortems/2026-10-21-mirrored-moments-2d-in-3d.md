# Postmortem: Mirrored moments in 2D-in-3D models

> **Status:** Published  
> **Date:** 2026-10-21  
> **Author:** Stabileo Verification Team  
> **Severity:** High  
> **Advisory:** [SA-2026-006](../advisories/SA-2026-006-mirrored-moments-2d-in-3d.md)  
> **Issue:** [#435](https://github.com/lambdaclass/stabileo/issues/435)  
> **Fix PR:** [#436](https://github.com/lambdaclass/stabileo/pull/436)  

---

## 1. Executive Summary

When solving a 2D frame model embedded into the 3D solver coordinate space, out-of-plane rotational DOF transformations inverted the local moment direction around the transverse axis. As a result, applied nodal moments and bending moments in plane had opposite signs compared to equivalent 2D solves, leading to incorrect deflection curvatures and inverted moment envelopes.

---

## 2. Timeline

| Time / Date | Event |
|---|---|
| **2026-08-14** | **Introduced**: 2D-to-3D projection bridge merged in PR #182 |
| **2026-09-01** | **Released**: Deployed in release v0.2.0 |
| **2026-10-15** | **Found**: Reported in [#435](https://github.com/lambdaclass/stabileo/issues/435) during frame validation |
| **2026-10-16** | **Contained**: Advisory draft created, warning added in documentation |
| **2026-10-21** | **Fixed**: PR [#436](https://github.com/lambdaclass/stabileo/pull/436) merged and released in v0.3.0 |

---

## 3. Root Cause

In 2D planar analysis, rotations about the Z axis follow standard counter-clockwise conventions $(+Z = X \times Y)$. When inflating a 2D model into a 3D coordinate system where the frame lies on the $X\text{–}Z$ plane, the out-of-plane axis was mapped to $+Y$. Because the local frame transformation matrix used a left-handed cross-product convention for transverse projections, the sign of the transformed moment was flipped by $-1$.

---

## 4. Why Existing Tests Missed It

- **Unit & Integration Tests**: 2D models were tested exclusively with the 2D linear solver, and 3D models were tested with symmetric loads where moment inversion on symmetric spans masked the asymmetry.
- **Verification Certificates & Proofs**: The Lean certificate checked global equilibrium $\mathbf{K}\mathbf{u} = \mathbf{f}$ within the 3D frame, which satisfied numerical equilibrium internally despite the inverted physical interpretation.
- **Differential Runs & Benchmarks**: Differential benchmarks ran pure 2D models against PyNite/Frame3DD 2D and pure 3D models against CalculiX, but had no metamorphic test comparing a 2D solve directly against the same model embedded into 3D.
- **Code Review**: The coordinate transformation matrix arithmetic appeared mathematically valid in isolation without verifying global frame projection chirality.

---

## 5. New Kind of Test or Check Added

- **Check Name / Location**: `engine/tests/metamorphic/transform.rs`
- **Test Kind**: Metamorphic coordinate transformation invariance test. A random 2D frame is solved in 2D and then rotated arbitrarily into 3D space $(X\text{–}Y, X\text{–}Z, Y\text{–}Z$, and arbitrary orientations); the transformed internal forces and displacements must match within $10^{-9}$.
- **Negative Case**: Intentionally inverting a local axis in the rotation matrix causes `test_2d_in_3d_rotation_invariance` to fail immediately with an explicit parity mismatch.

---

## 6. Variant Analysis (#448)

- **Search Query / Method**: Codebase grep for coordinate rotation matrices and local axis cross products: `grep -rn "cross" engine/src/element/`.
- **Other Components Examined**: Curved shell directors, solid-shell Jacobian mappings, Timoshenko 3D beam element kernels.
- **Additional Deficiencies Found & Fixed**: Uncovered sign inconsistencies in curved member local axes (#418, fixed in #523) and torsional buckling mode sign definitions (#404, fixed in #434).

---

## 7. Lessons Learned & Action Items

- [x] Embed metamorphic transformation tests into the continuous integration suite for all elements.
- [x] Prohibit manual coordinate transforms without verified right-handedness invariants.
