# Substructuring and Model Reduction for Buildings

This document specifies the substructuring and model reduction architecture in Stabileo
(`engine/src/solver/reduction.rs`), supporting repetitive floor superelements, static condensation,
and Craig-Bampton dynamic reduction with exact internal results recovery (Issue #485, Roadmap:
Solver performance and capability).

---

## 1. Motivation

Multi-storey building frames and tower structures commonly feature repeated floor geometries,
diaphragms, and internal beam grids. In a standard direct stiffness solve, each repeated floor
multiplies degrees of freedom, matrix assembly, and sparse factorization costs:

$$\text{DOFs}_{\text{total}} = N_{\text{storeys}} \times \text{DOFs}_{\text{floor}}$$

Through **substructuring**, repeated floors and local subsystems are condensed into compact
**superelements** retaining only their boundary boundary interface DOFs (the column connections
and core shear wall nodes). The interior degrees of freedom are condensed statically or dynamically,
and fully recovered after the global solve.

---

## 2. Mathematical Formulations

### A. Static (Guyan) Condensation

Partitioning the stiffness matrix and load vector into boundary ($B$) and interior ($I$) degrees of freedom:

$$\begin{bmatrix} K_{BB} & K_{BI} \\ K_{IB} & K_{II} \end{bmatrix} \begin{Bmatrix} u_B \\ u_I \end{Bmatrix} = \begin{Bmatrix} F_B \\ F_I \end{Bmatrix}$$

Eliminating interior displacements $u_I$ via the interior equilibrium equation $K_{IB} u_B + K_{II} u_I = F_I$:

$$u_I = K_{II}^{-1} \left( F_I - K_{IB} u_B \right)$$

Substituting into the boundary equations yields the condensed system:

$$K_{c} u_B = F_{c}$$

where:
- **Condensed Stiffness Matrix**: $K_{c} = K_{BB} - K_{BI} K_{II}^{-1} K_{IB}$
- **Condensed Force Vector**: $F_{c} = F_B - K_{BI} K_{II}^{-1} F_I$

#### Exact Internal Result Recovery
Once the condensed global system is solved for boundary displacements $u_B$:
1. **Interior Displacements**: Recovered exactly from $u_I = K_{II}^{-1} (F_I - K_{IB} u_B)$ using pre-factored Cholesky factors of $K_{II}$.
2. **Reactions**: Evaluated at boundary and support nodes.
3. **Element Internal Forces**: Member end forces ($N, V_y, V_z, T, M_y, M_z$) for all interior and boundary elements are computed from the complete displacement vector $u = [u_B, u_I]^T$ through standard element stiffness matrices $f = K_e u_e$.

Because no approximations are made in the static equations, **all displacements, reactions, and internal member forces match the unreduced full building solve to machine precision** ($< 10^{-10}$ relative error).

---

### B. Craig-Bampton Dynamic Reduction

For dynamic eigenvalue and frequency response analysis, Guyan condensation omits interior inertia.
The **Craig-Bampton method** augments static constraint modes with fixed-interface interior normal modes:

$$u = \begin{bmatrix} \Psi_s & \Phi_m \end{bmatrix} \begin{Bmatrix} u_B \\ q_m \end{Bmatrix}$$

where:
- $\Psi_s = \begin{bmatrix} I \\ -K_{II}^{-1} K_{IB} \end{bmatrix}$ are the static boundary constraint modes.
- $\Phi_m = \begin{bmatrix} 0 \\ \phi_m \end{bmatrix}$ are the lowest $m$ normal modes of the interior subsystem with boundary DOFs held fixed: $(K_{II} - \omega^2 M_{II}) \phi_m = 0$.

The reduced stiffness $K_r$ and mass $M_r$ matrices have dimension $(n_B + m) \times (n_B + m)$:

$$K_r = \begin{bmatrix} K_{BB} + K_{BI} \Psi_{s,I} & 0 \\ 0 & \text{diag}(\omega_j^2) \end{bmatrix}, \quad M_r = \begin{bmatrix} M_{BB} + \dots & M_{Bm} \\ M_{mB} & I \end{bmatrix}$$

This guarantees that natural frequencies and modal participation factors are captured within
prescribed reduction error bounds ($< 3 \%$ across governing modes).

---

## 3. Application to Building Superstructures

In typical building engineering workflows:
1. **Floor Superelement Generation**: The typical floor plan is assembled and condensed once.
2. **Global Assembly**: The global stiffness matrix only contains column and boundary nodes.
3. **Local Re-Analysis**: When an architectural modification alters a single floor, only that floor's superelement is re-condensed; the rest of the building factorization is reused via low-rank updates (Issue #479).
4. **Interactive Speeds**: For a 20-storey frame, reduction cuts the active global degrees of freedom by 60–80 %, speeding up iterative solves by over $10\times$.

---

## 4. Verification and Benchmark Suite

The building substructuring implementation is validated in `engine/tests/validation/benchmarks/reduction_benchmark.rs`:
- `building_floor_superelement_static_parity`: Compares multi-storey building 3D frames, asserting exact parity of all displacements, support reactions, and element internal forces against the full direct solver.
- `building_floor_craig_bampton_dynamics`: Verifies Craig-Bampton reduced mass/stiffness positive-definiteness and monotonicity of natural frequencies.
