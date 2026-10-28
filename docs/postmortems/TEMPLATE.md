# Postmortem: [Title / Defect Slug]

> **Status:** Draft / Published  
> **Date:** YYYY-MM-DD  
> **Author:** [Author Name / Handle]  
> **Severity:** Critical / High / Medium / Low  
> **Advisory:** [SA-YYYY-NNN](../advisories/SA-YYYY-NNN-slug.md)  
> **Issue:** [#Issue](https://github.com/lambdaclass/stabileo/issues/...)  
> **Fix PR:** [#PR](https://github.com/lambdaclass/stabileo/pull/...)  

---

## 1. Executive Summary

[1-2 paragraphs describing what went wrong, what the user exposure was, and the immediate corrective actions taken.]

---

## 2. Timeline

| Time / Date | Event |
|---|---|
| **YYYY-MM-DD** | **Introduced**: Defect merged into `main` via PR #... |
| **YYYY-MM-DD** | **Released**: Deployed in release v... / live on web |
| **YYYY-MM-DD** | **Found**: Reported by ... in issue #... |
| **YYYY-MM-DD** | **Contained**: Workaround or temporary guard established |
| **YYYY-MM-DD** | **Fixed**: PR #... merged to `main` and verified in CI |

---

## 3. Root Cause

[Detailed engineering explanation of the failure mechanism. Include code snippets, mathematical formulations, or sign convention discrepancies that caused the incorrect result.]

---

## 4. Why Existing Tests Missed It

Explain why each existing verification gate failed to catch the issue before it reached `main`:

- **Unit & Integration Tests**: [Why the existing unit test suite did not trigger this code path or condition]
- **Verification Certificates & Proofs**: [Why certificate checks or formal verifiers did not catch this condition]
- **Differential Runs & Benchmarks**: [Why comparisons against reference solvers (CalculiX, Frame3DD, PyNite, etc.) did not detect the discrepancy]
- **Code Review**: [Why the human and automated reviews missed the defect during PR review]

---

## 5. New Kind of Test or Check Added

A defect that reached `main` proves a gap in the test hierarchy. Every postmortem must name a new or extended automated test kind added to prevent this entire bug class:

- **Check Name / Location**: `[path/to/test.rs or test.py]`
- **Test Kind**: [e.g., Metamorphic relation test / Property-based fuzz test / Differential benchmark / Lean axiom assertion / Static lint]
- **Negative Case**: [Exact negative case proving the test fails if the defect is reintroduced, per #451]

---

## 6. Variant Analysis (#448)

Systematic search across the codebase for the same bug class:

- **Search Query / Method**: [e.g., `grep -rn "pattern" engine/src`]
- **Other Components Examined**: [List other element kernels, solvers, or design checks checked]
- **Additional Deficiencies Found & Fixed**: [Any related issues discovered and fixed during this audit]

---

## 7. Lessons Learned & Action Items

- [ ] [Action item 1]
- [ ] [Action item 2]
