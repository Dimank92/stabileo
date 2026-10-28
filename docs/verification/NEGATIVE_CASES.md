# Negative cases: proving checks can fail

> **A check that cannot fail protects nothing.**
> Every new design check, verification test, certificate check, CI gate or alert ships with a negative case: an input or mutation that must make it fail (wrong value, missing feature, injected NaN, mutated kernel, red job), run in CI.
> Roadmap: Trust and verification item 16 ([#451](https://github.com/lambdaclass/stabileo/issues/451)).

## Principle

A green test proves only that the current codebase satisfies the current assertions. It does not prove that the test would notice a defect. Without a negative case, tests can quietly become tautologies:
- An assertion comparing `actual == actual`.
- An early return before verification executes.
- An exception handler that swallows failures into passes.
- A gate that exits with code 0 on missing tools or malformed configurations.

To prevent tautological checks, **every verification check must ship with a demonstrated negative case**: an explicit input, mutation, or planted bug that triggers a hard failure.

## PR Requirement & Review Checklist

1. **PR Template Line**: Every PR must include:
   ```markdown
   - negative case: <test name or reproduction step>
   - evidence: <how the negative case was verified failing>
   ```
2. **Review Checklist**: Reviewers verify that:
   - The negative case is executed in CI or has an automated test harness.
   - The failure mode is specific and names the defect (not an unrelated crash).
   - No mutants in the gate survive without justification.

---

## Audit of Existing CI Gates

The repository's existing verification gates were audited to ensure each has an automated negative case proving it fails when tampered with.

| Gate | Purpose | Negative case proof | Automated harness |
|---|---|---|---|
| **Lean axioms check** | Proves result certificates rely only on standard Lean axioms (`propext`, `Classical.choice`, `Quot.sound`) without `sorry` or extra axioms. | Plants an `axiom`, `@[implemented_by]`, and `@[extern]` in `Certify/BundleParse.lean` and verifies rejection by both scan and grep. | `verify/scripts/test-axiom-gate.sh` (executed in CI under `verify.yml`) |
| **Symbolic `--check`** | Verifies that element stiffness matrices match symbolic derivations in `tools/symbolic/derive.py`. | Tampering with or drifting `engine/tests/symbolic_elements.rs` causes `tools/symbolic/gen.py --check` to reject the diff and exit with code 1. | `scripts/tests/test_negative_cases_audit.py` (`test_symbolic_check_negative_case`) |
| **Feature matrix** | Verifies every model feature is supported or refused by name across all analyses, with 0 ignored features. | Modifying `docs/verification/FEATURE_MATRIX.md` or introducing an ignored feature causes `engine/tests/feature_matrix.rs` to fail. | `scripts/tests/test_negative_cases_audit.py` (`test_feature_matrix_negative_case`) |
| **Differential exit codes** | Compares stabileo against independent solvers (Frame3DD, PyNite, MYSTRAN, Kratos, CalculiX). | A discrepancy between stabileo and the reference solver triggers exit code 1 in `tools/differential/run.py`. | `scripts/tests/test_negative_cases_audit.py` (`test_differential_exit_code_negative_case`) |
| **Verification alerts** | Alerts on failures in scheduled verification jobs (`nightly.yml`, `verify.yml`, etc.) by opening `verification-failure` issues. | Simulating a failed job step triggers issue creation/comment; a green run produces no alert. | `scripts/tests/test_negative_cases_audit.py` (`test_verification_alert_negative_case`) |

---

## How to add a negative case to a new check

When adding a check or test:
1. **Engine/Solver**: Introduce a mutation (e.g. invert a sign in the element kernel or omit a load term) and verify the test fails with a meaningful error message.
2. **Design Checks**: Pass an undersized section or excessive demand that violates the governing code clause, and verify the check emits `fail` (never `ok`).
3. **CI Gate**: Provide a self-test script or test fixture (like `verify/scripts/test-axiom-gate.sh`) that plants invalid syntax or mismatched states and asserts a non-zero exit code.
