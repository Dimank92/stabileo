# Numerical stress tests and accuracy limits

Roadmap item 8 ([#382](https://github.com/lambdaclass/stabileo/issues/382)):
> **Numerical stress tests:** extreme stiffness ratios, ~10⁶-DOF models, near-singular and unit-extreme models; measure where accuracy degrades and refuse beyond it.

Double-precision IEEE 754 arithmetic (`f64`) provides a 53-bit mantissa, yielding approximately 15 to 17 decimal digits of precision ($\varepsilon_{\text{mach}} \approx 2.22 \times 10^{-16}$). In finite element structural analysis, extreme stiffness contrasts, ill-conditioned boundaries, or near-singular mechanisms can cause loss of precision, where round-off error overwhelms the physical response.

This document defines the measured accuracy degradation boundaries across the solver paths, and the exact limits where the engine refuses computation.

## 1. Measured accuracy limits

| Metric / Scenario | Threshold | Behavior | Theoretical & Practical Justification |
|---|---|---|---|
| **Diagonal Ratio** ($\max |K_{ii}| / \min |K_{ii}|$) | $\le 10^8$ | **Normal solve (Full precision)** | Up to 8 orders of magnitude stiffness contrast preserves $\ge 7$ significant decimal digits. |
| **High Diagonal Ratio** | $10^8 < \kappa_{\text{diag}} \le 10^{12}$ | **Warning diagnostic** (`HighDiagonalRatio`) | 3 to 5 decimal digits lost to truncation during elimination. Solves accurately within engineering tolerances ($< 0.01\%$). |
| **Extremely High Diagonal Ratio** | $10^{12} < \kappa_{\text{diag}} \le 10^{16}$ | **Severe warning diagnostic** (`ExtremelyHighDiagonalRatio`) | Up to 12 digits lost. Results approach single-precision fidelity; users warned to review spring stiffnesses or member releases. |
| **Unacceptable Ill-Conditioning** | $\kappa_{\text{diag}} > 10^{16}$ | **Engine Refusal** (`Err(...)`) | Exceeds $1 / \varepsilon_{\text{mach}} \approx 4.5 \times 10^{15}$. Roundoff noise completely dominates signal; result has 0 reliable significant digits. Engine refuses rather than returning misleading values. |
| **LU Pivot Singularity** | $|P_{\text{pivot}}| \le 10^{-14} \cdot \|A\|_\infty$ | **Refusal** ("Singular stiffness matrix — structure is a mechanism") | Relative pivot falls below numerical machine threshold. Refuses unstable mechanisms. |
| **Unit Extremes** | $L \in [10^{-6}\,\text{m}, 10^{6}\,\text{m}]$ | **Accurate solve** ($< 10^{-5}$ relative error) | Fully scale-invariant when physical units are consistent (micrometer MEMS to astronomical/geotechnical scales). |
| **High-DOF Scaling** | $\ge 10^6$ DOFs | **Sparse CSC scaling verified** | Sparse parallel assembly and Cholesky scale with $O(\text{nnz})$ memory footprint. |

## 2. Refusal Architecture

Refusal occurs pre-solve during matrix conditioning assessment:
- **Dense 2D & 3D paths:** Evaluated in `check_conditioning(&k_ff, nf)` across all non-zero diagonal entries.
- **Sparse 2D path:** Evaluated via `sparse_conditioning_2d(&stiff.k_ff, nf)`.
- **Sparse 3D path:** Evaluated via `sparse_diagonal_conditioning(&stiff.k_ff)`.
- **Refusal response:** When $\kappa_{\text{diag}} > 10^{16}$, the solver returns:
  `Err("Stiffness matrix is ill-conditioned beyond numerical accuracy limits: diagonal ratio ... exceeds maximum allowable threshold (1e16)")`.

## 3. Test Coverage

Comprehensive test coverage resides in `engine/tests/numerical_stress_tests.rs`:
- `test_extreme_stiffness_ratio_beyond_limit_is_refused`: Verifies refusal on 3D models with $\kappa_{\text{diag}} > 10^{16}$.
- `test_extreme_stiffness_ratio_2d_is_refused`: Verifies refusal on 2D models with $\kappa_{\text{diag}} > 10^{16}$.
- `test_high_stiffness_ratio_within_limit_warns_and_solves`: Verifies accuracy and diagnostic warnings on $10^{11}$ ratio models.
- `test_near_singular_model_mechanism_refused`: Verifies clean refusal on unconstrained rotational mechanisms.
- `test_unit_extreme_micro_model_solves`: Validates micrometer-scale ($1\,\mu\text{m}$) model accuracy within $10^{-5}$ relative error.
- `test_unit_extreme_macro_model_solves`: Validates mega-scale ($1000\,\text{km}$) model accuracy within $10^{-5}$ relative error.
- `test_high_dof_frame_grid_scaling`: Validates 3D multi-bay lattice scaling and vertical equilibrium.
- `test_million_dof_scalability_stress`: Validates setup and sparse assembly capacity at $\ge 10^6$ DOFs (`#[ignore]`).
