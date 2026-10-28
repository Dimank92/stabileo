# Postmortem: Spectral certificate passing on non-finite values

> **Status:** Published  
> **Date:** 2026-10-21  
> **Author:** Stabileo Verification Team  
> **Severity:** Medium  
> **Advisory:** [SA-2026-010](../advisories/SA-2026-010-spectral-certificate-nonfinite.md)  
> **Issue:** [#335](https://github.com/lambdaclass/stabileo/issues/335)  
> **Fix PR:** [#398](https://github.com/lambdaclass/stabileo/pull/398)  

---

## 1. Executive Summary

A response-spectrum modal analysis certificate bundle accepted solution vectors containing `NaN` and `+Inf` values without emitting a verification failure. The downstream certificate checker evaluated relational comparisons against non-finite floating point numbers as false, which bypassed threshold validation and allowed an unverified model to claim a verified certificate status.

---

## 2. Timeline

| Time / Date | Event |
|---|---|
| **2026-07-22** | **Introduced**: Initial response spectrum certificate validator added in PR #210 |
| **2026-09-01** | **Released**: Included in v0.2.0 |
| **2026-10-02** | **Found**: Reported in [#335](https://github.com/lambdaclass/stabileo/issues/335) by fuzz testing certificate generation |
| **2026-10-04** | **Contained**: Added temporary assertion in certificate exporter |
| **2026-10-21** | **Fixed**: PR [#398](https://github.com/lambdaclass/stabileo/pull/398) merged and released in v0.3.0 |

---

## 3. Root Cause

In IEEE 754 floating point arithmetic, comparisons with `NaN` (such as `x > tol` or `x < tol`) evaluate to `false`. The certificate verification logic checked:
```rust
if residual > MAX_ALLOWED_RESIDUAL {
    return Err(VerificationError::ResidualExceeded);
}
```
When `residual` was `NaN`, the conditional was skipped, and the routine returned `Ok(Certificate)`. The certificate validator lacked an explicit `.is_finite()` guard.

---

## 4. Why Existing Tests Missed It

- **Unit & Integration Tests**: All unit tests tested physical, well-conditioned models with valid numerical solutions where residuals were always normal positive floats.
- **Verification Certificates & Proofs**: The Lean kernel assumes real arithmetic $\mathbb{R}$, where non-finite values do not exist; floating point translation into Lean did not explicitly model IEEE 754 exceptional values.
- **Differential Runs & Benchmarks**: Differential runs execute well-posed benchmark problems without numerical singularities.
- **Code Review**: The comparison `residual > MAX_ALLOWED_RESIDUAL` appeared intuitively sufficient without considering IEEE NaN comparison semantics.

---

## 5. New Kind of Test or Check Added

- **Check Name / Location**: `engine/tests/fuzz_certificates.rs`
- **Test Kind**: Property-based fuzz testing with NaN and Infinity injection. Every certificate verification entry point is subjected to randomized IEEE exceptional payloads to ensure immediate and deterministic rejection.
- **Negative Case**: Injecting `f64::NAN` or `f64::INFINITY` into any component of the certificate input vector must cause `verify_certificate` to return `Err(VerificationError::NonFinite)`.

---

## 6. Variant Analysis (#448)

- **Search Query / Method**: Codebase audit for comparisons on float residuals without `.is_finite()`: `grep -rn " > " engine/src/certify/`.
- **Other Components Examined**: Linear static certificate, P-Delta stability enclosure, modal orthogonality verification, design check utilization ratios.
- **Additional Deficiencies Found & Fixed**: Added strict `.is_finite()` checks across all certificate parsers and check rows in `engine/src/certify/` and `web/src/lib/verification/`.

---

## 7. Lessons Learned & Action Items

- [x] Mandate `.is_finite()` guards on all floating point bounds and comparisons in verification kernels.
- [x] Integrate IEEE NaN fuzzing as a standard CI gate for all certificate kinds.
