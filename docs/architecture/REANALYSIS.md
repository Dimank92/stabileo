# Solver Re-Analysis, Sherman-Morrison-Woodbury Updates, and Multi-RHS Solves

> Architectural specification for fast re-analysis, localized low-rank stiffness updates,
> and multi-right-hand-side solves in the Stabileo structural analysis engine.
> Implements Issue #479; tracks Section "Solver performance and capability" in [`PRODUCT_ROADMAP.md`](../roadmap/PRODUCT_ROADMAP.md).

---

## 1. Executive Summary & Design Motivation

During iterative structural design, engineers frequently modify individual members (e.g. up-sizing a girder or column), adjust support spring stiffnesses, or analyze dozens of serviceability and ultimate limit state load combinations.

In conventional finite element workflows, any minor edit triggers a full global stiffness re-assembly and Cholesky factorization ($O(N^{1.5} - N^2)$ complexity). On 10,000–100,000 DOF models, this creates hundreds of milliseconds to seconds of latency, breaking interactive UI responsiveness.

The Stabileo **Re-Analysis Engine** introduces:
1. **Multi-RHS Solve Pipelining:** Factorizing the global stiffness matrix once with `prepare_static_3d` and reusing the triangular Cholesky factors ($L \cdot L^T$) across arbitrary numbers of load cases ($O(\text{nnz}(L))$ triangular substitutions per load case).
2. **Sherman-Morrison-Woodbury Low-Rank Updates:** When a member's cross-section or a nodal support spring changes, the stiffness modification $\Delta K$ is of low rank ($k \le 12$). By the Woodbury matrix identity, the updated displacement field is computed using the cached baseline factorization plus a tiny $k \times k$ capacitance matrix solve, delivering **100×–1000× speedups**.
3. **Rigorous Residual Verification & Automatic Fallback:** The updated displacements are checked against the exact equilibrium equation $\|\tilde{K} u - f\|_\infty / \|f\|_\infty < 10^{-7}$. If ill-conditioning or numerical loss of precision is detected, the engine transparently falls back to a clean full re-solve.
4. **End-to-End Certification:** All re-analyzed results pass through `certify_results_3d`, guaranteeing that the certificate bundle covers the updated result.

---

## 2. Mathematical Formulation

### 2.1 The Sherman-Morrison-Woodbury Identity

Given a baseline structural stiffness system $K_0 u_0 = f$ whose Cholesky decomposition $K_0 = L_0 L_0^T$ is already computed and cached, suppose a design edit introduces a rank-$k$ stiffness change $\Delta K$:

$$\tilde{K} = K_0 + U C V^T$$

where:
- $U \in \mathbb{R}^{n \times k}$ and $V \in \mathbb{R}^{n \times k}$ are degree-of-freedom selector matrices mapping the localized edit DOFs to global system coordinates.
- $C \in \mathbb{R}^{k \times k}$ is the compact local stiffness change matrix ($k \le 12$ for a 3D frame element, $k \le 3$ for a nodal support spring).

By the Woodbury matrix identity:

$$\tilde{K}^{-1} = K_0^{-1} - K_0^{-1} U \left( C^{-1} + V^T K_0^{-1} U \right)^{-1} V^T K_0^{-1}$$

### 2.2 Re-Analysis Algorithm

When solving $\tilde{K} u = f$:
1. **Baseline Displacement:** Retrieve or compute $u_0 = K_0^{-1} f$ via forward/backward substitution with $L_0$.
2. **Auxiliary Response Matrix:** Compute $Z = K_0^{-1} U$ by solving $k$ right-hand-sides using the cached factorization $L_0$. Since $k \le 12$, this step requires negligible CPU time.
3. **Capacitance System:** Form the small $k \times k$ matrix:
   $$M = C^{-1} + V^T Z$$
4. **Local Update Vector:** Solve the small $k \times k$ dense system:
   $$M w = V^T u_0$$
5. **Global Solution Reconstruction:**
   $$u = u_0 - Z w$$

---

## 3. Residual Verification and Fallback Protocol

Low-rank updates are subject to ill-conditioning when the structural edit induces extreme stiffness contrasts (e.g. releasing a rigid brace or adding near-infinite support stiffness).

To guarantee safety and prevent unconservative numerical drift:
1. Every re-analysis step computes the scaled equilibrium residual:
   $$\eta = \frac{\|\tilde{K} u - f\|_\infty}{\|f\|_\infty + 1}$$
2. **Acceptance Threshold:** If $\eta < 10^{-7}$, the Woodbury update is accepted as exact to round-off.
3. **Automatic Fallback:** If $\eta \ge 10^{-7}$ or the capacitance matrix $M$ is singular, the engine immediately abandons the low-rank update and executes a full re-solve (`solve_3d`).

---

## 4. Multi-RHS Solve Pipelining

For multiple load combinations:

$$K \begin{bmatrix} u_1 & u_2 & \dots & u_m \end{bmatrix} = \begin{bmatrix} f_1 & f_2 & \dots & f_m \end{bmatrix}$$

`ReAnalysisSession3D::solve_multi_rhs` executes consecutive triangular solves over the loaded right-hand sides, achieving optimal CPU cache locality and zero matrix re-assembly overhead.
