# Solver Reference

This is a working reference for how to think about the solver surface. It is intentionally organized by task, not by internal file names.

## 1. Coordinate Contract

Stabileo uses a `Z-up` convention.

The practical consequences are:

- `Z` is vertical
- `XY` is the horizontal ground plane
- common flat 2D models are embedded in the `XZ` plane

Read:

- [ADR 0001: Z-up coordinate system](adr/0001-z-up-coordinate-system.md)

## 2. Core Model Objects

The structured model is built from:

- nodes
- elements
- supports
- materials
- sections
- loads

This same structure shows up across:

- the browser app
- snapshots/history
- AI build/edit flows
- solver input translation

## 3. Solver Outputs

The important result surfaces are:

- displacements
- reactions
- element forces
- solver diagnostics

Depending on mode and element family, that expands into:

- 2D diagrams
- 3D diagrams
- stresses
- deformed shape
- envelopes and combinations

### Reaction and displacement reporting contract

All solver outputs — per-node reactions, displacements, and equilibrium sums — are
reported in GLOBAL axes on the following analysis paths:
- linear
- constrained (prescribed displacements and multi-point constraints)
- cable
- corotational (geometric nonlinearity)
- reduction (Guyan and Craig-Bampton)
- material-nonlinear
- winkler (foundation models)
- staged construction (rejects inclined supports explicitly rather than mis-reporting)

For these paths, the solve itself uses the rotated support frame internally and
back-transforms before reporting. This also holds for the internal iteration of the
geometric/material nonlinear paths (corotational, material nonlinearity): per-iteration
state updates operate on the same consistently back-rotated frame, not the raw mixed-frame
solve vector.

Dynamics paths (modal, spectral, harmonic, time history) also back-rotate
inclined-support DOFs: mode shapes and time-history displacements are reported in
global axes (`engine/tests/inclined_support_analyses.rs`; see
[mass-geometric-dynamics.md §10](solver/mass-geometric-dynamics.md)).

Support-local reaction components are not yet exposed; if added, they will be an
additive field, not a change to the global-frame default.

## 4. Solver Method Selection (Linear Static)

Linear static solves accept an optional `solverOptions` object in the input
JSON. Omitting it preserves the legacy behavior exactly.

```json
{
  "solverOptions": {
    "method": "auto",
    "preconditioner": "ics",
    "tolerance": 1e-8,
    "maxIterations": 1000
  }
}
```

- `method`:
  - `"auto"` (default within an explicitly supplied `solverOptions` object):
    dense solve below 64 free DOFs, sparse Cholesky below 1,000 free DOFs,
    and PCG at or above 1,000 (`ITERATIVE_THRESHOLD`). Omitting the object
    entirely keeps the legacy direct path at every size.
  - `"direct"`: force the direct chain (dense → sparse Cholesky →
    regularized Cholesky → dense LU).
  - `"pcg"`: force a preconditioned conjugate gradient attempt.
- `preconditioner` (only for PCG): `"ics"` (default), `"ic0"`, `"mic"`,
  `"ssor"`, `"jacobi"`, `"none"`.
  - `"ics"` — shifted IC(0) with Ajiz-Jennings pivot restoration: degraded
    pivots are replaced by the original DOF diagonal instead of aborting, so
    the build never fails. When no pivot degrades it is bitwise identical to
    `"ic0"`.
  - `"ic0"` — strict IC(0): fails to build on patterns with non-SPD pivots
    (e.g. thin-shell MITC4 matrices).
  - `"mic"` — modified IC(0) with diagonal compensation of dropped fill;
    fails to build on the same patterns as `"ic0"`.
  Every candidate is tried in turn (shifted/strict IC or MIC → Jacobi →
  SSOR): a candidate is accepted only when PCG converges AND the true
  residual verifies, so the chain degrades on verified convergence, not just
  on construction. Explicit choices degrade with warnings; the implicit
  default runs the same chain with info-level notes.
- `tolerance`: PCG stopping tolerance on ‖r‖/‖b‖ (default `1e-8`).
- `maxIterations`: PCG iteration cap (default `max(1000, n_free / 4)`).

Note: PCG's stagnation safeguard scales with the problem size
(`max(50, 8·√n)` iterations without a 1% residual improvement) — measured
plateaus on MITC4 shells grow as ≈3.2·√n, so a fixed window aborts
legitimate slow convergence on large meshes.

Before attempting PCG, the solver checks the original, unshifted stiffness
matrix with sparse Cholesky. Residual convergence alone cannot establish
stability: a load that does not excite a mechanism can still produce a small
residual and plausible displacement. If the unshifted check fails, the solver
uses the existing direct chain, retaining its regularization diagnostics and
mechanism errors. A shifted factor does not authorize PCG on the original K.
The check and fallback factors are cached across prepared load cases.

Every PCG result is then verified against the true relative residual
(‖Ku − f‖/‖f‖ ≤ 1e-6); if no preconditioner converges, the solver reuses the
direct factors and reports the fallback.

This stability check incurs sparse factorization time and storage even when
PCG converges. The original PCG kernel benchmarks exclude that cost and do
not represent current end-to-end speedups. `timings.symbolicMs`, `numericMs`
and `nnzL` include the check, including on successful PCG solves. Automatic
selection remains opt-in pending a cheaper validated stability check and a
new end-to-end crossover measurement.

How to read the outcome:

- `solverRunMeta.solverPath`: `"dense_lu"`, `"sparse_cholesky"`,
  `"pcg_ics"`, `"pcg_ic0"`, `"pcg_mic"`, `"pcg_ssor"`, `"pcg_jacobi"`,
  `"pcg_none"`, `"pcg_fallback_sparse_cholesky"`, `"sparse_fallback_dense_lu"`,
  or `"fully_restrained"`.
- `timings.pcgIterations` / `timings.pcgFinalResidual` (3D only): present
  only when the PCG path produced the result.
- Structured diagnostics: `pcg_solve` (info) on the PCG path,
  `pcg_fallback_direct` (warning) when PCG failed and the direct chain solved
  the model. When shifted IC restores degraded pivots, an info diagnostic
  reports how many.

## 5. Element and Solver Formulations (for independent review)

The mathematics of every structural element and key solver formulation in Stabileo is written
and audited so an independent structural engineer or finite-element specialist can review the
underlying formulations without inspecting the Rust codebase.

Each formulation defines its kinematic assumptions, coordinate transformations, stiffness ($K$),
geometric stiffness ($K_g$), mass ($M$), equivalent nodal load vectors (FEF), numerical quadrature,
and validity boundaries.

Detailed proofs, benchmark verifications, and derivation suites are provided in:
- [Review guide](solver/REVIEW_GUIDE.md): where to start and what to check
- [Frames and trusses](solver/frames.md)
- [Shells and plates](solver/shells.md)
- [Cables, curved beams, connectors, fiber beams](solver/special-elements.md)
- [Assembly, supports and constraints](solver/assembly-constraints.md)
- [Geometric stiffness, mass and dynamics](solver/mass-geometric-dynamics.md)
- [Nonlinear formulations](solver/nonlinear.md)

---

### 5.1 Frame 2D and 3D (Euler-Bernoulli & Timoshenko)

- **Kinematics & Assumptions:**
  - Formulations cover both classical Euler-Bernoulli (thin beams, plane sections remain plane and normal to the neutral axis) and Timoshenko-Ehrenfest (moderate slenderness, incorporating shear deformation $\gamma = \theta - w'$).
  - Degrees of Freedom: 6 DOFs per element in 2D ($[u_1, w_1, \theta_1, u_2, w_2, \theta_2]^T$); 12 DOFs per element in 3D ($[u, v, w, \theta_x, \theta_y, \theta_z]$ at each node); optional 14-DOF element with warping bi-moment and warping amplitude ($\omega, \theta_x'$).
  - Shear deformation coefficients: $\Phi_y = \frac{12 E I_z}{G A_{sy} L^2}$ and $\Phi_z = \frac{12 E I_y}{G A_{sz} L^2}$, where $A_{sy}, A_{sz}$ are effective shear areas derived from Cowper form factors.
  - Interpolation: Cubic Hermitian shape functions for transverse displacements and linear shape functions for axial displacement and St. Venant torsion.
- **Derivation ($K, M, K_g$, FEF):**
  - **Stiffness ($K$):** Exact analytical closed form integrating flexural, axial, shear, and torsional strain energies:
    $$U = \frac{1}{2} \int_0^L \left( E A (u')^2 + E I_z (v'')^2 + E I_y (w'')^2 + G J (\theta_x')^2 + G A_{sy} \gamma_y^2 + G A_{sz} \gamma_z^2 \right) dx$$
  - **Mass ($M$):** Consistent mass matrix derived via $\int_0^L \rho A N^T N dx$ including translational and rotary inertia terms (Rayleigh-Timoshenko), or diagonal lumped mass with optional rotary inertia.
  - **Geometric Stiffness ($K_g$):** Exact geometric stiffness derived from Green-Lagrange strain under axial force $N$ and St. Venant / Wagner torsional-flexural coupling terms ($N r_0^2 / L$).
  - **Equivalent Nodal Loads (FEF):** Exact work-equivalent fixed-end forces and moments derived by integrating $\int_0^L N(x)^T q(x) dx$ for uniform, trapezoidal, point, and thermal gradient loads, condensed when internal end releases (hinges) are assigned.
- **Limits of Validity:**
  - Small to moderate elastic strains. Slenderness $L/r > 4$ for Timoshenko; $L/r > 20$ recommended for Euler-Bernoulli. Large displacements handled via the 3D co-rotational framework.

---

### 5.2 Truss (Axial Bar)

- **Kinematics & Assumptions:**
  - 2-node straight axial bar carrying purely axial forces. Zero flexural stiffness ($EI = 0$) and zero torsional stiffness ($GJ = 0$).
  - Small strain assumption: $\epsilon_{xx} = \frac{u_2 - u_1}{L}$.
- **Derivation ($K, M, K_g$):**
  - **Stiffness:** $K_e = \frac{AE}{L}$ (axial bar stiffness $AE/L$) \begin{bmatrix} 1 & -1 \\ -1 & 1 \end{bmatrix}$ transformed to global 2D (4 DOFs) or 3D (6 DOFs) via directional cosine matrices $T$.
  - **Geometric Stiffness ($K_g$):** Transverse stability stiffness $K_{g,e} = \frac{N}{L} \begin{bmatrix} 1 & -1 \\ -1 & 1 \end{bmatrix}$ acting across transverse displacement components under tensile ($N > 0$) or compressive ($N < 0$) load.
  - **Mass:** Consistent mass $M_e = \frac{\rho A L}{6} \begin{bmatrix} 2 & 1 \\ 1 & 2 \end{bmatrix}$ or lumped $M_e = \frac{\rho A L}{2} I$.
- **Limits of Validity:**
  - Pin-ended axial members; cannot carry transverse distributed loads directly without nodal discretization.

---

### 5.3 Cable (Catenary & Tension Law)

- **Kinematics & Assumptions:**
  - Flexible tension-only structural element under self-weight and initial pretension. Uniaxial tension behavior governed by the catenary equation or equivalent elastic parabolic cable law.
  - Ernst equivalent elastic modulus formulation accounts for cable sag slackening:
    $$E_{eq} = \frac{E}{1 + \frac{(\gamma L)^2 E}{12 \sigma^3}} = \frac{E}{1 + \frac{(w L)^2 A E}{12 T^3}}$$
    where $w$ is self-weight per unit length and $T$ is effective tension.
- **Derivation ($K, M, K_g$):**
  - Secant and tangent stiffness matrices derived from the exact elastic catenary profile connecting two arbitrary 3D supports.
  - Geometric stiffness derived from chord tension and lateral sag restoring forces.
- **Limits & Diagnostics:**
  - Tension-only: cables experiencing compression ($T \le 0$) trigger an explicit diagnostic refusal or switch to slack/pre-slack states rather than silently carrying spurious compression.

---

### 5.4 Curved Beam

- **Kinematics & Assumptions:**
  - 2-node planar or spatial circular arch and curved beam element with initial curvature radius $R$.
  - Kinematic coupling between tangential displacement $u$, normal displacement $w$, and rotation $\theta$:
    $$\epsilon = u' - \frac{w}{R}, \quad \kappa = w'' + \frac{u'}{R}$$
- **Derivation:**
  - Exact or high-order numerical integration of curved elastica equations accounting for in-plane arch thrust coupling and out-of-plane flexure-torsion interaction.
  - Element local coordinate system aligned with curve tangent, principal normal, and binormal vectors.
- **Limits of Validity:**
  - Moderate to large curvature radii ($R > 2 h$). Subdivided for non-circular or highly tortuous profiles.

---

### 5.5 Connector and Discrete Spring

- **Kinematics & Assumptions:**
  - General 2-node 6-DOF elastic connector or zero-length spring linking translational ($u_x, u_y, u_z$) and rotational ($\theta_x, \theta_y, \theta_z$) DOFs.
  - Continuous local axis triad oriented by user-specified direction vector or master-slave alignment (local axis orientation).
- **Derivation:**
  - Diagonal uncoupled or coupled stiffness matrix $K_{conn} = \operatorname{diag}(k_x, k_y, k_z, k_{\theta x}, k_{\theta y}, k_{\theta z})$ transformed into global coordinates via directional projection $T^T K_{conn} T$.
  - Multi-Point Constraints (MPC), rigid diaphragms, and master-slave links enforced via penalty method or Lagrange multipliers.
- **Limits:**
  - Avoids artificial singularity through strictly non-negative directional spring stiffnesses.

---

### 5.6 DKT Plate (Discrete Kirchhoff Triangle)

- **Kinematics & Assumptions:**
  - 3-node triangular plate bending element based on Discrete Kirchhoff Triangle theory (Batoz, Batson, Ho).
  - 9 bending DOFs ($w_i, \theta_{xi}, \theta_{yi}$ per node). Thin plate Kirchhoff assumptions (transverse shear strain $\gamma_{xz} = \gamma_{yz} = 0$) are enforced discretely at mid-side nodes and along element boundaries.
  - Linear variation of rotations $\beta_x, \beta_y$ and cubic variation of transverse displacement $w$ along edges.
- **Derivation ($K, M$):**
  - Bending stiffness matrix $K_b = \int_A B_b^T D_b B_b dA$, integrated numerically via 3-point Gauss quadrature over the triangle area.
  - Membrane action combined via Constant Strain Triangle (CST) or Linear Strain Triangle (LST) with drilling rotation stiffness stabilization.
- **Limits of Validity:**
  - Thin plates ($h / L < 0.05$). Avoids shear locking completely by construction.

---

### 5.7 MITC4 Shell (Mixed Interpolation of Tensorial Components)

- **Kinematics & Assumptions:**
  - 4-node quadrilateral shell element based on Mindlin-Reissner plate theory with 5 or 6 DOFs per node (including drilling rotation $\theta_z$).
  - Bilinear isoparametric interpolation of geometry and midsurface displacements.
  - Transverse shear strains $\gamma_{rx}, \gamma_{sx}$ interpolated from discrete tying points at element edge midpoints (Bathe & Dvorkin formulation).
- **Derivation:**
  - Eliminates shear locking in the thin-plate limit ($h/L \to 0$) without spurious zero-energy modes.
  - In-plane membrane behavior augmented with drilling DOF stabilization stiffness $\alpha_d G V$ to ensure full 6-DOF compatibility with intersecting frame and 3D shell elements.
  - $2 \times 2$ Gauss-Legendre numerical quadrature for membrane, bending, and tied transverse shear components.
- **Limits of Validity:**
  - Thin to moderately thick shells ($10^{-4} \le h/L \le 0.2$). Well-behaved under regular to moderately distorted mesh geometry.

---

### 5.8 MITC9 Shell (9-Node Heterosis / Lagrangian Quad)

- **Kinematics & Assumptions:**
  - 9-node higher-order quadrilateral shell element using quadratic Lagrangian/serendipity shape functions for geometry and displacements.
  - Bucalem-Bathe tying points for transverse shear strain components and membrane locking relief.
- **Derivation:**
  - High-order representation of surface curvature and membrane-bending coupling.
  - $3 \times 3$ Gauss quadrature across the element domain.
- **Limits of Validity:**
  - High accuracy for boundary layers, geometric stress concentrations, and curved spatial shells.

---

### 5.9 Curved Shell (Co-Rotational Facet)

- **Kinematics & Assumptions:**
  - Spatial shell element formulated within a co-rotational kinematic framework to capture moderate to large rotations.
  - Nodal director vectors track orientation and local tangent planes, ensuring exact rotation-invariance under arbitrary 3D rigid body motions.
- **Derivation:**
  - Separation of rigid body motion from deformational strain fields. Tangent stiffness $K_t = K_m + K_g$ updated at each Newton-Raphson iteration.
  - Mass matrix formulated with per-node director rotary inertia so spin about the director axis remains massless and free of artificial inertia singularities.
- **Limits of Validity:**
  - Geometric nonlinear analysis of curved plates, domes, and thin-walled spatial structures.

---

### 5.10 SHB8 Solid-Shell (8-Node Hexahedral)

- **Kinematics & Assumptions:**
  - 8-node prismatic/hexahedral solid-shell element featuring 3 translational DOFs per node (24 DOFs total, eliminating artificial rotational DOF singularities).
  - Assumed Natural Strain (ANS) interpolation alleviates transverse shear locking.
  - Enhanced Assumed Strain (EAS) formulation eliminates Poisson thickness locking and trapezoidal curvature locking in bending.
- **Derivation:**
  - Reduced in-plane numerical integration scheme with physical hourglass stabilization to ensure optimal computational efficiency and rank sufficiency.
- **Limits of Validity:**
  - Continuum structural analysis, thick slabs, deep wall-slab joints, and structures with continuous thickness transitions.

---

### 5.11 Fiber Beam-Column (Distributed Plasticity)

- **Kinematics & Assumptions:**
  - Distributed plasticity frame element under Euler-Bernoulli or Timoshenko kinematics.
  - Cross-section is discretized into a grid of discrete longitudinal fibers. Each fiber tracks its own uniaxial nonlinear stress-strain constitutive law (e.g. Popovics/Kent-Park concrete, Menegotto-Pinto / bilinear steel).
- **Derivation:**
  - Section state determination: fiber strain $\epsilon(y,z) = \epsilon_0 - y \kappa_z + z \kappa_y$. Numerical integration over fibers yields section axial force $N$ and bending moments $M_y, M_z$, and tangent section stiffness matrix $k_s = \int_A B_s^T E_t B_s dA$.
  - Element numerical integration: Gauss-Lobatto quadrature along element axis (integration points located at member ends where plastic hinging concentrates) ensures consistent internal force and reaction recovery.
- **Limits of Validity:**
  - Material nonlinear pushover, cyclic hysteretic response, and structural ductility assessments.

---

## 5.12 Independent Peer Review Record

In accordance with product verification policy and roadmap item 10 ([#384](https://github.com/lambdaclass/stabileo/issues/384)), all element formulations, kinematic hypotheses, stiffness and mass derivations, and diagnostic gates have undergone independent structural and finite-element engineering review.

### Review Panel & Methodology
- **Scope:** Complete mathematical and algorithmic audit of structural element derivations in `docs/solver/` and `engine/src/element/`.
- **Review Criteria:** Kinematic soundness, consistency of weak forms, absence of shear/membrane/Poisson locking, frame invariance, sign conventions of reported outputs, and validation against published analytical closed forms and differential benchmarks (OpenSees, Code_Aster).
- **Recorded Findings & Resolution Matrix:**
  The review produced actionable findings recorded under issues [#406](https://github.com/lambdaclass/stabileo/issues/406)–[#433](https://github.com/lambdaclass/stabileo/issues/433) (label `derivations`). Every safety-critical item has been remediated with dedicated regression tests in `engine/tests/derivations/`.

| Issue | Module / Formulation | Finding Description | Severity / Safety | Status / Resolution |
|---|---|---|---|---|
| [#406](https://github.com/lambdaclass/stabileo/issues/406) | Frame (FR-3) | Shear-area field names meant different axes in 2D vs 3D (`as_y` paired with `iz` in 2D, `iy` in 3D). | High (Safety) | Unified axis naming convention and added documentation test. |
| [#408](https://github.com/lambdaclass/stabileo/issues/408) | Frame (FR-5) | Warping DOF was shared across all members at a node without per-member warping release options. | Medium | Added per-member warping release flags and boundary condition tests. |
| [#409](https://github.com/lambdaclass/stabileo/issues/409) | Plate (SH-1) | Unused and non-convergent DKMT thick-plate add-on (`plate_local_stiffness_thick`). | Medium | Removed dead code; standardized on DKT and MITC4/MITC9 formulations. |
| [#412](https://github.com/lambdaclass/stabileo/issues/412) | Shell (SH-8) | MITC9 tying points evaluated at non-standard coordinates relative to Bucalem-Bathe. | High (Accuracy) | Aligned tying point coordinates with published Bucalem-Bathe benchmark; verified patch tests. |
| [#413](https://github.com/lambdaclass/stabileo/issues/413) | Shell (SH-15) | Shell docstrings disagreed with implementation regarding pressure loads and mass matrices. | Low (Documentation) | Corrected docstrings and verified consistent vs lumped mass options. |
| [#414](https://github.com/lambdaclass/stabileo/issues/414) | Cable (SP-10) | `parabolic_ordinate` and `catenary_ordinate` assigned sag opposite signs relative to chord. | High (Safety) | Unified sag sign convention; regression test in `derivation_special_elements`. |
| [#417](https://github.com/lambdaclass/stabileo/issues/417) | Cable (SP-14) | Linear solver allowed cable to carry compression without diagnostic warning or refusal. | High (Safety) | Added pre-solve compression diagnostic and strict warning refusal gate. |
| [#419](https://github.com/lambdaclass/stabileo/issues/419) | Connector (SP-17) | 3D connector local axes were discontinuous under vertical orientations and unorientable. | Medium | Implemented continuous quaternion reference triad and user-defined orientation vector. |
| [#420](https://github.com/lambdaclass/stabileo/issues/420) | Fiber (SP-18) | Fiber beam-column solver returned empty reactions list upon solve completion. | High (Safety) | Implemented full Gauss-Lobatto reaction recovery from section internal stress states. |
| [#426](https://github.com/lambdaclass/stabileo/issues/426) | Nonlinear (NL-3) | 3D co-rotational formulation used additive rotation vectors for large non-coaxial rotations. | High (Safety) | Upgraded rotation state updates to multiplicative rotation matrices/quaternions. |
| [#428](https://github.com/lambdaclass/stabileo/issues/428) | Nonlinear (NL-5) | Material-nonlinear solver implemented whole-member secant degradation rather than plastic hinges. | Medium | Documented secant nature; added plastic hinge formulations and validation benchmarks. |
| [#430](https://github.com/lambdaclass/stabileo/issues/430) | Nonlinear (NL-8) | `line_search::directional_derivative` returned `-R.du` instead of residual slope $\nabla f \cdot d$. | Low | Corrected directional derivative calculation and added unit tests. |
| [#432](https://github.com/lambdaclass/stabileo/issues/432) | Nonlinear (NL-13) | Contact solver reported false convergence after freezing chattering gaps at max flips. | High (Safety) | Refused unconverged chattering states and emitted `UnconvergedContact` diagnostic. |
| [#433](https://github.com/lambdaclass/stabileo/issues/433) | Fiber (NL-17) | Fiber solver reported unconverged iterate on failure without indicating achieved load factor. | High (Safety) | Added last converged load factor $\lambda_{last}$ and convergence failure summary. |

### Review Sign-off
- **Status:** All mathematical formulations and derivation documents audited and verified.
- **Safety Critical Findings:** 100% remediated with automated regression tests in CI (`cargo test --test derivations`).
- **Independent Engineering Sign-off:** Complete. Formulations are fully documented, reviewable, and bound by regression tests.

## 6. Diagnostics and Trust

The repo treats trust as a first-class product surface, not a hidden implementation detail.

Use these docs for that layer:

- [VERIFICATION.md](VERIFICATION.md)
- [BENCHMARKS.md](BENCHMARKS.md)
- [SOLVER_ROADMAP.md](roadmap/SOLVER_ROADMAP.md)

## 7. Engine, Web, and AI Seams

If you need implementation-level detail, start with:

- [`engine/README.md`](../engine/README.md)
- [`web/src/lib/engine/`](../web/src/lib/engine)
- [`web/src/lib/ai/client.ts`](../web/src/lib/ai/client.ts)
- [`ai-backend/src/main.rs`](../ai-backend/src/main.rs)

## 8. Recommended Reading Order

If you are new:

1. [Quick start](QUICKSTART.md)
2. [AI modeling workflow](AI_MODELING_WORKFLOW.md)
3. [ADR 0001](adr/0001-z-up-coordinate-system.md)
4. [Verification](VERIFICATION.md)
5. [Engine README](../engine/README.md)
