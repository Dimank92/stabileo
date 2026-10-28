# Adjoint Sensitivity Analysis for Structural FEA

This document details the mathematical architecture, algorithms, and validation of the adjoint sensitivity solver in Stabileo.

## 1. Mathematical Formulation

### 1.1 Linear Static Adjoint Sensitivities

For a linear structural system in equilibrium:
$$K(p) u(p) = f(p)$$

where:
- $K \in \mathbb{R}^{n \times n}$ is the symmetric global stiffness matrix,
- $u \in \mathbb{R}^n$ is the global displacement vector,
- $f \in \mathbb{R}^n$ is the global equivalent nodal load vector,
- $p \in \mathbb{R}^m$ is the vector of design parameters (e.g. cross-sectional areas, moments of inertia, shell thicknesses, Young's moduli).

Consider an arbitrary scalar response objective $g(u, p)$ (e.g. nodal displacement, inter-storey drift, reaction force, or stress resultant).
The total derivative with respect to design parameter $p_k$ is given by the chain rule:
$$\frac{dg}{dp_k} = \frac{\partial g}{\partial p_k} + \left( \frac{\partial g}{\partial u} \right)^T \frac{du}{dp_k}$$

Differentiating the governing equation $K u = f$ with respect to $p_k$:
$$K \frac{du}{dp_k} + \frac{\partial K}{\partial p_k} u = \frac{\partial f}{\partial p_k} \implies \frac{du}{dp_k} = K^{-1} \left( \frac{\partial f}{\partial p_k} - \frac{\partial K}{\partial p_k} u \right)$$

Substituting into the total derivative:
$$\frac{dg}{dp_k} = \frac{\partial g}{\partial p_k} + \left( \frac{\partial g}{\partial u} \right)^T K^{-1} \left( \frac{\partial f}{\partial p_k} - \frac{\partial K}{\partial p_k} u \right)$$

We define the **Adjoint Vector** $\lambda \in \mathbb{R}^n$ as the solution to the linear system:
$$K \lambda = \frac{\partial g}{\partial u}$$

Because $K$ is symmetric ($K = K^T$):
$$\lambda = K^{-1} \frac{\partial g}{\partial u}$$

Thus, the sensitivity of $g$ with respect to $p_k$ simplifies to:
$$\frac{dg}{dp_k} = \frac{\partial g}{\partial p_k} + \lambda^T \frac{\partial f}{\partial p_k} - \lambda^T \frac{\partial K}{\partial p_k} u$$

For element-level design parameters (e.g., area $A_e$, modulus $E_e$, thickness $t_e$), $\frac{\partial K}{\partial p_k}$ has non-zero entries only on the degrees of freedom of element $e$:
$$\lambda^T \frac{\partial K}{\partial p_k} u = \lambda_e^T \frac{\partial K_e}{\partial p_k} u_e$$
where $u_e$ and $\lambda_e$ are the element displacement and adjoint vectors.

### 1.2 Modal Frequency Sensitivities

For the generalized symmetric eigenvalue problem:
$$(K - \omega_i^2 M) \phi_i = 0, \quad \phi_i^T M \phi_i = 1$$

where $\omega_i$ is the $i$-th natural frequency and $\phi_i$ is the $M$-orthonormal mode shape.
Differentiating with respect to parameter $p_k$ and premultiplying by $\phi_i^T$:
$$\phi_i^T \left( \frac{\partial K}{\partial p_k} - \frac{d(\omega_i^2)}{dp_k} M - \omega_i^2 \frac{\partial M}{\partial p_k} \right) \phi_i + \phi_i^T (K - \omega_i^2 M) \frac{d\phi_i}{dp_k} = 0$$

Since $\phi_i^T (K - \omega_i^2 M) = 0$ and $\phi_i^T M \phi_i = 1$:
$$\frac{d(\omega_i^2)}{dp_k} = \phi_i^T \frac{\partial K}{\partial p_k} \phi_i - \omega_i^2 \phi_i^T \frac{\partial M}{\partial p_k} \phi_i$$

The frequency derivative is:
$$\frac{d\omega_i}{dp_k} = \frac{1}{2\omega_i} \frac{d(\omega_i^2)}{dp_k}$$

The mode shape $\phi_i$ itself acts as the adjoint vector: **no linear system solve is required**.

### 1.3 Linearized Buckling Sensitivities

For the elastic buckling eigenvalue problem:
$$(K + \lambda_c K_G(u)) \psi = 0, \quad \psi^T K \psi = 1$$

where $\lambda_c$ is the critical load factor and $\psi$ is the buckling mode shape.
The sensitivity is:
$$\frac{d\lambda_c}{dp_k} = -\frac{\psi^T \frac{\partial K}{\partial p_k} \psi + \lambda_c \psi^T \frac{\partial K_G}{\partial p_k} \psi}{\psi^T K_G \psi}$$

## 2. Verification Against Central Finite Difference

To guarantee zero regressions and exact analytical correctness, all adjoint derivatives are systematically verified against Central Finite Difference (FD):
$$\left( \frac{dg}{dp_k} \right)_{\text{FD}} = \frac{g(p_k + \Delta p_k) - g(p_k - \Delta p_k)}{2 \Delta p_k}$$
with perturbation step $\Delta p_k = 10^{-5} \cdot p_k$.

The relative agreement threshold is enforced:
$$\frac{\left| \frac{dg}{dp_k} - \left( \frac{dg}{dp_k} \right)_{\text{FD}} \right|}{\max\left( \left| \frac{dg}{dp_k} \right|, 10^{-6} \right)} < 10^{-4}$$
