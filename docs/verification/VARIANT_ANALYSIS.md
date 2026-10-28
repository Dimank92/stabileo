# Variant Analysis: Class-Level Defect Elimination

> **Fix the class, not only the instance.**  
> After every confirmed bug (safety defect or normal issue), search the codebase for the same bug class and add an automated check that catches the whole class.  
> Roadmap: Trust and verification item 14 ([#448](https://github.com/lambdaclass/stabileo/issues/448)). Tracker [#396](https://github.com/lambdaclass/stabileo/issues/396).

---

## 1. Principle

Fixing an isolated bug leaves its siblings alive elsewhere in the repository. When a defect is discovered in one element kernel, design check, or solver pass, developers must perform **variant analysis**:
1. Identify the general **bug class**.
2. **Search the codebase** (using `grep`, `semgrep`, or AST queries) for all locations sharing that pattern.
3. Fix all discovered instances in the same PR or filed as immediate blockers.
4. Add or extend a **class-level automated check in CI** (such as a metamorphic invariant, property fuzz target, or feature-matrix gate) that permanently prevents any instance of that bug class from recurring.

---

## 2. Core Bug Classes & Automated CI Checks

The repository tracks 5 canonical bug classes identified through verification audits and safety postmortems:

| Bug Class | Description & Historical Precedents | Search Methodology | Automated Class-Level Check |
|---|---|---|---|
| **1. Frame / rotation / sign conventions** | Coordinate transformation chirality, mirrored moments (2D-in-3D mirrored moments [#435](https://github.com/lambdaclass/stabileo/issues/435), local axes [#418](https://github.com/lambdaclass/stabileo/issues/418)/[#419](https://github.com/lambdaclass/stabileo/issues/419), member couples [#343](https://github.com/lambdaclass/stabileo/issues/343)). | Grep coordinate transforms and rotation matrices for left-handed cross products: `grep -rn "cross" engine/src/element/`. | `engine/tests/symbolic_elements.rs` (exact symbolic derivation check) |
| **2. Unit conversions** | Inconsistent dimensions (kN vs N, MPa vs Pa, mm vs m, t-z/q-z curve units [#423](https://github.com/lambdaclass/stabileo/issues/423)). | Grep for unit scale factors (`1e3`, `1e6`, `1e-3`, `1e-6`) in calculation loops and check parameter signatures: `grep -rn "1e" engine/src/postprocess/`. | `engine/tests/cirsoc301_check.rs` (dimensional consistency across standard steel cross-sections) |
| **3. Double counting** | Applying the same physical load or reaction multiple times (support reactions lumped and also applied, [#343](https://github.com/lambdaclass/stabileo/issues/343)). | Grep load assembly dispatchers and reaction accumulation loops for duplicated vector adds. | `engine/tests/validation/domains/response/reaction_checks.rs` (global equilibrium reaction sum balance) |
| **4. NaN / degenerate input accepted** | Non-finite values evaluated as passing, division by zero, or degenerate inputs accepted (certificate passing on NaN [#335](https://github.com/lambdaclass/stabileo/issues/335), zero unbraced length $L_b = 0$, zero capacity passing). | Grep float inequality comparisons missing `.is_finite()` guards: `grep -rn " > " engine/src/certify/`. | `engine/tests/integration/design_check_evaluability.rs` (degenerate geometry and NaN refusal checks) |
| **5. Silent drops** | Analyses or checks silently ignoring an unsupported model feature or boundary condition (constraints on supported nodes, couples on members, features ignored by an analysis). | Verify feature matrix coverage for all element and analysis pairings: `UPDATE_FEATURE_MATRIX=1 cargo test`. | `engine/tests/feature_matrix.rs` (exhaustive feature-matrix support gate) |

---

## 3. Pull Request Requirements

Every pull request that resolves a bug must include a **Variant Analysis** section:

```markdown
### Variant Analysis (#448)
- **Bug Class**: <e.g., Frame / rotation / sign conventions>
- **Search Query**: <grep/ast query executed across repo>
- **Instances Found**: <list of additional files/functions examined and results>
- **Class-Level Check**: <link to new or extended automated test/invariant added>
```

PR reviewers verify that:
1. The search was thorough and covered all relevant solver/element modules.
2. The added check tests the class generally rather than asserting on a single hardcoded input.
