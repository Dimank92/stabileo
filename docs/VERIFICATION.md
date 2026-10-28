# Verification Strategy

## Scope

This document explains how Dedaliano should build solver trust.

It is not the benchmark ledger and it is not the product roadmap.

- Use [`BENCHMARKS.md`](../docs/BENCHMARKS.md) for benchmark status and solver maturity.
- Use [`SOLVER_ROADMAP.md`](../docs/roadmap/SOLVER_ROADMAP.md) for solver sequencing and implementation order.
- Use [`PRODUCT_ROADMAP.md`](../docs/roadmap/PRODUCT_ROADMAP.md) for app and workflow sequencing.
- Use [`POSITIONING.md`](../docs/POSITIONING.md) for the business meaning of solver trust.
- Use [`research/lean_formal_verification.md`](../docs/research/lean_formal_verification.md) for the Lean-specific formal-verification research plan and ROI ranking.

## Goal

The goal is not to claim that the entire solver is "formally verified."

The goal is to make correctness visible and credible through a layered verification program:

- analytical validation
- published benchmarks
- cross-validation against trusted references
- regression discipline
- invariant and property-based testing
- fuzzing
- selective proof-oriented rigor on small critical kernels
- real-model acceptance testing

In practice, the strongest identity is:

`an open, browser-native structural solver with unusually deep public proof of correctness`

## Verification Layers

### 1. Unit Tests

Use unit tests for compact mathematical building blocks:

- element stiffness matrices
- fixed-end forces
- coordinate transformations
- mass matrices
- geometric stiffness terms
- damping terms
- postprocessing formulas

These tests catch sign, indexing, and formula errors early.

### 2. Analytical Validation

Use closed-form and textbook references for:

- beams
- frames
- trusses
- buckling
- modal frequencies
- thermal loads
- settlements
- Timoshenko beam behavior
- cable statics
- prestress and staged sanity checks

This is the first public proof layer for structural mechanics correctness.

### 3. Reference Benchmark Validation

Use published benchmark families and solver cross-checks such as:

- NAFEMS
- ANSYS Verification Manual
- Code_Aster
- SAP2000
- OpenSees
- Robot Structural Analysis
- STAAD.Pro

This is the strongest external trust layer and should remain the center of the public correctness story.

### 4. Differential and Consistency Testing

Use differential testing for:

- dense vs sparse assembly
- equivalent 2D vs 3D formulations
- small-load linear vs nonlinear consistency
- reduced vs unreduced systems
- locked fixture baselines

Differential testing should be about consistency in the current engine, not parity with deleted implementations.

### 5. Invariant and Property-Based Testing

Use invariant testing and property-based testing for solver rules that should hold across many models:

- equilibrium
- symmetry
- reciprocal behavior
- rigid-body mode handling
- superposition where applicable
- reaction/load balance
- constraint consistency
- reduced vs unreduced equivalence

This is one of the highest-ROI next steps once the newest solver families are in place.

### 6. Fuzzing

Use fuzzing for randomized, valid model generation to find:

- edge cases
- regression triggers
- indexing failures
- conditioning pathologies
- mixed-feature interaction failures

Best early targets:

- constraints
- contact/gap behavior
- staged construction
- mixed support conditions
- mixed beam/shell assemblies

The linear, modal, spectral, harmonic, time-history, and staged entry points share the
same input-validation gate: referential integrity, physical bounds (E > 0, A > 0, non-zero
element length), and NaN/Inf rejection. `engine/tests/input_validation_gates.rs` enforces
the contract "degenerate input returns Err, never panics", and `engine/tests/fuzz_crash_free.rs`
fuzzes the linear, modal, time-history, and staged paths.

### 7. Selective Formal Verification

Formal verification is valuable, but only selectively.

It should be applied only to small, correctness-critical kernels with stable specifications.

Good candidates:

- DOF numbering and mapping
- constraint transformation logic
- assembly invariants
- compact state-transition kernels

For the current Lean-oriented research plan, theorem ladder, and recommended proof scope, see [`research/lean_formal_verification.md`](../docs/research/lean_formal_verification.md).

Bad candidates:

- full nonlinear solvers
- full shell formulations end to end
- whole staged/PT workflows
- complete contact algorithms

Those are better served by benchmarks, invariants, regressions, and real-model acceptance.

### 8. Performance and Scale Verification

Solver quality is not only correctness at small scale.

Track:

- solve time
- memory use
- iteration counts
- sparse vs dense crossover behavior
- large-model reliability

This is part of solver trust, not a separate concern.

### 9. Real-Model Acceptance Testing

Maintain a suite of representative models that look like real engineering work:

- building frames
- slab/shell systems
- staged and prestressed models
- cable and SSI cases
- nonlinear benchmark structures

These should act as release gates for major solver changes.

The release-gate suite of public benchmark buildings with published reference
results (SAC steel frames, IASC-ASCE SHM frame, ...) is described in
[`verification/ACCEPTANCE.md`](verification/ACCEPTANCE.md) and runs as
`cargo test --release --test gates acceptance::`.

### 10. Validation Against Physical Experiments (ASME V&V 10 / NAFEMS)

While verification confirms that the numerical equations are solved accurately ("solving the equations right"),
validation demonstrates that the mathematical models adequately represent real-world physical behavior
("solving the right equations"). For nonlinear material models, physical laboratory experiments are essential.

Following **ASME V&V 10-2019** (*Standard for Verification and Validation in Computational Solid Mechanics*)
and **NAFEMS** guidelines, Stabileo evaluates its nonlinear constitutive laws (concrete, fiber sections,
plastic hinges, and contact) against published physical laboratory specimens:

| Experimental Domain | Published Benchmark Campaign | Key Physical Phenomena | Quantitative Validation Metric | Acceptance Tolerance |
|---|---|---|---|---|
| **RC Beams** | Bresler-Scordelis (Bresler–Scordelis) (UC Berkeley OA-1 / OA-2 series) | Flexural tensile cracking, rebar yield, compressive crushing, midspan load-deflection | Peak capacity relative error $E = \frac{|P_{sim} - P_{exp}|}{P_{exp}}$ | $\le 8\%$ |
| **RC Columns** | Burns & Siess / Saatcioglu & Razvi | Combined axial load ratio ($P/A_g f_c' = 0.20$), cyclic flexure, plastic hinge rotation, ductility | Peak moment $M_n$ error and plastic hinge rotation $\theta_p$ | $\le 10\%$ moment, $\le 12\%$ rotation |
| **Shake-Table Frames** | E-Defense & PEER full-scale shake table dynamic tests | Multi-story dynamic time-history response, modal frequency shift and period elongation under plastic damage, roof drift hysteretic response | Period shift error $E_T$, peak dynamic drift ratio error $E_{drift}$ | $\le 5\%$ period, $\le 8\%$ drift |
| **Vecchio-Collins Panels** (Vecchio–Collins) | Univ. of Toronto Shell Element Tester (PV19, PV20, PV27) | Membrane pure shear, concrete compression softening (MCFT), crack rotation, steel yielding | Ultimate shear stress $v_u$ relative error | $\le 9\%$ |
| **Connection Tests** | SAC Joint Venture & Popov cyclic moment connections | Cyclic moment-rotation hysteresis, beam-column flange contact gap opening/closing, contact slip | Plastic moment capacity $M_p$, contact gap opening error | $\le 6\%$ capacity, $\le 10\%$ gap |

#### Validation Methodology and Acceptance Tolerances
- **Error Metric:** Follows ASME V&V 10 quantitative error formulation:
  $$E = \frac{|y_{sim} - y_{exp}|}{|y_{exp}|}$$
- **Validation Uncertainty:** Evaluates combined experimental measurement uncertainty ($U_{exp}$), material input uncertainty ($U_{input}$), and numerical discretization error ($U_{num}$).
- **Automated Regression Suite:** These published experimental benchmarks run continuously in the test suite under `engine/tests/validation/physical_experiments.rs` (`cargo test --test validation physical_experiments`).

## Priority Order

The best ROI verification order is:

1. benchmark hardening on newest solver families
2. invariant and property-based testing
3. real-model acceptance testing
4. performance and scale regression tracking
5. selective formal verification on small kernels

This order gives more trust per unit of effort than trying to formally verify large solver components too early.

## What Verification Means Here

Verification does not mean proving every line of the solver mathematically.

It means building a correctness program strong enough that:

- errors are found early
- regressions are trapped permanently
- public benchmark evidence is deep
- engineering users can audit trust instead of taking it on faith

That is the standard Dedaliano should aim for.
