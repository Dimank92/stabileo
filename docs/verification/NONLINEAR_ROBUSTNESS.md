# Nonlinear and Dynamic Robustness Verification

This document audits convergence criteria, step-size sensitivities, energy drift monitoring, and limit-point path following in the Stabileo FE engine, satisfying Item 7 of the [Solver and proof depth roadmap](../roadmap/PRODUCT_ROADMAP.md#solver-and-proof-depth) (Issue #381).

---

## 1. Convergence Criteria Audit

Nonlinear Newton-Raphson solvers must guard against premature termination under ill-conditioned or path-dependent deformation. Three complementary norms govern convergence:

### 1.1 Force Residual Norm
$$\frac{\|R\|_2}{\|F_{\text{ext}}\|_2} < \text{tol}$$
Measures dynamic/static unbalance between external applied loads and internal resisting forces. Implemented in `engine/src/solver/material_nonlinear.rs`, `engine/src/solver/corotational.rs`, and `engine/src/solver/arc_length.rs`. When $\|F_{\text{ext}}\|_2 \approx 0$, a reference denominator guard prevents division by zero.

### 1.2 Displacement Increment Norm
$$\frac{\|\delta u\|_2}{\|u\|_2} < \text{tol}_u$$
Measures whether the iterative correction alters the state meaningfully. Guarded against zero total displacement at initial load steps.

### 1.3 Incremental Work / Energy Norm
$$\frac{|\delta u^T R|}{\Delta u_0^T R_0} < \text{tol}_e$$
Quantifies residual work relative to the initial work increment of the step. Prevents false convergence in stiff structural elements where small force imbalances cause negligible deformation.

---

## 2. Dynamic Energy-Drift Monitoring & Diagnostics

In time-history direct integration (`engine/src/solver/time_integration.rs`), mechanical energy conservation is tracked at every step:
- **Kinetic Energy:** $E_k(t) = \frac{1}{2} v(t)^T M v(t)$
- **Strain Energy:** $E_s(t) = \frac{1}{2} u(t)^T K u(t)$
- **External Work:** $W_{\text{ext}}(t) = \sum_{k=1}^n \frac{1}{2} (f_k + f_{k-1})^T (u_k - u_{k-1})$
- **Damping Dissipation:** $D_{\text{diss}}(t) = \sum_{k=1}^n \Delta t \, \bar{v}_k^T C \bar{v}_k$

### Energy Imbalance
$$I(t) = |E_k(t) + E_s(t) - E(0) + D_{\text{diss}}(t) - W_{\text{ext}}(t)|$$
$$\text{Relative Drift} = \frac{\max_t I(t)}{\max(E_{\max}, |W_{\text{ext}}|, 1.0)}$$

### Diagnostic Reporting
The solver populates `DynamicEnergyBalance` in `TimeHistoryResult` and `TimeHistoryResult3D`:
```rust
pub struct DynamicEnergyBalance {
    pub initial_energy: f64,
    pub final_energy: f64,
    pub external_work: f64,
    pub damping_dissipation: f64,
    pub energy_imbalance: f64,
    pub relative_drift: f64,
}
```
If relative drift exceeds 5% (`> 0.05`), the solver generates an actionable warning diagnostic:
`"Energy drift of X% detected during time-history integration — consider reducing the time step size or using algorithmic damping (HHT-alpha)"`.

---

## 3. Step Sensitivity Benchmarks

### 3.1 Load-Step Sensitivity (Geometric Nonlinear Corotational)
Refining load steps ($N = 5, 10, 20, 40$) on cantilever structures demonstrates asymptotic convergence to machine precision:
- $N = 5$ vs $N = 40$: relative displacement difference $< 10^{-6}$.
- $N = 20$ vs $N = 40$: relative displacement difference $< 10^{-5}$.
Newton-Raphson iterations preserve path consistency under moderate load steps without divergence.

### 3.2 Time-Step Sensitivity (Newmark Average Acceleration)
Under Newmark integration ($\beta = 0.25, \gamma = 0.5$):
- Global error shrinks monotonically with time-step halving: $\Delta t = 4\,\text{ms} \to 2\,\text{ms} \to 1\,\text{ms} \to 0.5\,\text{ms}$.
- Empirical convergence rate $p = \log_2(\text{err}_1 / \text{err}_2) \approx 2.0$, confirming theoretical second-order temporal accuracy.

---

## 4. Path Following Past Limit Points (Snap-Through)

Shallow toggle arches subjected to apex transverse loading exhibit limit-point instability (snap-through):
- Standard load control fails as the tangent stiffness matrix $K_T$ approaches singularity ($\det K_T \to 0$).
- Displacement control (`solve_displacement_control`) and arc-length methods successfully trace the full equilibrium path across the apex:
  1. Pre-buckling ascending branch ($\Delta \lambda > 0$).
  2. Limit point / peak load factor $\lambda_{\text{peak}}$ at horizontal alignment.
  3. Post-buckling softening branch with negative load increments ($\Delta \lambda < 0$).
  4. Inverted post-snap-through tension recovery.
Verified in `engine/tests/nonlinear_dynamic_robustness.rs::test_path_following_past_limit_point_snap_through`.
