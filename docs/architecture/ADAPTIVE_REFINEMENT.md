# Adaptive Refinement to a Target Accuracy

This document describes the design, algorithms, and stopping criteria for adaptive mesh refinement driven by a-posteriori discretization error estimation in Stabileo.

## 1. Overview and Problem Statement

Discretization error reflects the discrepancy between the continuous elasticity solution and the finite element approximation. While structural certificates prove bounds on the residual of the assembled discrete system $K u = f$, they do not certify mesh convergence. A coarse shell mesh can have a certified solve while being 30% inaccurate relative to the true continuum.

Uniform refinement ($h \to h/2 \to h/4$) quadruples the number of elements at every step, quickly leading to millions of degrees of freedom. Adaptive refinement optimizes computational resources by refining only elements in regions with high local error gradients.

## 2. Algorithm and Stopping Criteria

### 2.1 Error Indicator and Marking

The refinement loop utilizes the recovery-based Zienkiewicz–Zhu Superconvergent Patch Recovery (ZZ-SPR) estimator:
$$\eta_e = \|\sigma^* - \sigma_h\|_{L_2(\Omega_e)}$$
$$\eta_{rel} = \frac{\sqrt{\sum_e \eta_e^2}}{\sqrt{U_h + \sum_e \eta_e^2}}$$

Elements are marked for refinement using **Dörfler bulk marking**:
1. Sort elements in descending order of energy error contribution $\eta_e^2$.
2. Select the minimal subset $\mathcal{M}$ such that:
   $$\sum_{e \in \mathcal{M}} \eta_e^2 \ge \theta \sum_{e} \eta_e^2$$
   where $\theta \in (0, 1]$ is the marking fraction (default: $\theta = 0.5$).

### 2.2 Refinement Strategy

Marked elements undergo:
- **$h$-refinement**: splitting quadrilaterals and triangles into sub-elements, maintaining geometric continuity and conformality.
- **$p$-refinement / element elevation**: elevating bilinear MITC4 quad elements to 9-node biquadratic MITC9 shells in regions where curvature or bending dominates.

### 2.3 Strict Stopping Criteria

The adaptive refinement loop iterates until one of the following criteria is met:
1. **Target Accuracy Reached**: $\eta_{rel} \le \text{target\_error}$ (e.g. 5% or 3%). The solution is marked as converged.
2. **Iteration Limit Reached**: The loop terminates if `max_iterations` (default: 5) is exceeded without meeting the target.
3. **DOF Capacity Limit Reached**: Further refinement would exceed `max_dofs` (default: 250,000).
4. **Unestimable / Inconclusive Cases**: If the error cannot be rigorously estimated (reasons: `unloaded`, `noStrainEnergy`, `tooCoarse`, or `nonFinite`), the loop immediately halts and reports the exact status. **Such cases are NEVER passed as converged**.

## 3. Benchmark Verification

### 3.1 Scordelis–Lo Cylindrical Shell Roof
Under self-weight, high transverse bending moments and boundary layer shears concentrate near the free edges and supports.
- **Uniform Refinement**: Reaching 5% energy error requires uniform splitting of all elements, resulting in over 12,000 DOFs.
- **Adaptive Refinement**: Selectively refines the free edge boundary layer, reaching the target accuracy with under 5,200 DOFs (over 55% reduction in DOFs).

### 3.2 Simply Supported Plates
Under central point load or re-entrant corners, stress gradients concentrate locally. Adaptive refinement reaches the target accuracy with fewer DOFs and less memory than global uniform refinement.
