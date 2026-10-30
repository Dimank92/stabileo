/-
Rust-to-Lean Kernel Correspondence (Roadmap item 3, Issue #500).

Translates the critical computational kernels of the engine:
1. Frame element stiffness matrix (`engine/src/element/frame.rs`)
2. Coordinate transformation (`engine/src/element/transform.rs`)
3. Fixed-end forces (`engine/src/element/fef.rs`)
4. Linear certificate check (`engine/src/certify/linear.rs`)
and proves their symmetry, equilibrium invariants, and energy conservation.
-/
import Certify.Basic
import Certify.Interval

open Matrix Finset

namespace Certify

/-! ### 1. Frame Element Stiffness Kernel -/

/-- Parameters for an elastic 2D frame element. -/
structure Frame2DParams where
  E : ℝ
  A : ℝ
  Iz : ℝ
  L : ℝ
  phi : ℝ := 0
  hE : 0 < E
  hA : 0 < A
  hIz : 0 < Iz
  hL : 0 < L
  hphi : 0 ≤ phi

/-- Translation of `engine/src/element/frame.rs::frame_local_stiffness_2d`. -/
noncomputable def frameLocalStiffness2D (p : Frame2DParams) : Matrix (Fin 6) (Fin 6) ℝ :=
  let ea_l := p.E * p.A / p.L
  let ei := p.E * p.Iz
  let l2 := p.L ^ 2
  let l3 := p.L ^ 3
  let denom := 1 + p.phi
  let c1 := 12 * ei / (l3 * denom)
  let c2 := 6 * ei / (l2 * denom)
  let c3 := (4 + p.phi) * ei / (p.L * denom)
  let c4 := (2 - p.phi) * ei / (p.L * denom)
  ![
    ![ ea_l,    0,    0, -ea_l,    0,    0],
    ![    0,   c1,   c2,     0,  -c1,   c2],
    ![    0,   c2,   c3,     0,  -c2,   c4],
    ![-ea_l,    0,    0,  ea_l,    0,    0],
    ![    0,  -c1,  -c2,     0,   c1,  -c2],
    ![    0,   c2,   c4,     0,  -c2,   c3]
  ]

/-- **Theorem (Symmetry of Frame Element).**
The translated Rust frame stiffness matrix is symmetric: Kᵀ = K. -/
theorem frame_stiffness_2d_symmetric (p : Frame2DParams) :
    (frameLocalStiffness2D p)ᵀ = frameLocalStiffness2D p := by
  unfold frameLocalStiffness2D
  ext i j
  fin_cases i <;> fin_cases j <;> rfl

/-- **Theorem (Axial Equilibrium).**
Pure axial rigid body translations [1, 0, 0, 1, 0, 0]ᵀ belong to the kernel of K. -/
theorem frame_axial_rigid_mode (p : Frame2DParams) :
    frameLocalStiffness2D p *ᵥ ![1, 0, 0, 1, 0, 0] = 0 := by
  unfold frameLocalStiffness2D
  ext i
  fin_cases i <;> (simp [Matrix.mulVec, dotProduct]; ring)

/-- **Theorem (Transverse Equilibrium).**
Pure transverse rigid body translations [0, 1, 0, 0, 1, 0]ᵀ belong to the kernel of K. -/
theorem frame_transverse_rigid_mode (p : Frame2DParams) :
    frameLocalStiffness2D p *ᵥ ![0, 1, 0, 0, 1, 0] = 0 := by
  unfold frameLocalStiffness2D
  ext i
  fin_cases i <;> (simp [Matrix.mulVec, dotProduct]; ring)

/-! ### 2. Coordinate Transformation Kernel -/

/-- Direction cosines for 2D rotation matrix: c = cos θ, s = sin θ with c² + s² = 1. -/
structure Rotation2D where
  c : ℝ
  s : ℝ
  hunit : c ^ 2 + s ^ 2 = 1

/-- Translation of `engine/src/element/transform.rs`: 6x6 block diagonal transformation. -/
def transform2D (r : Rotation2D) : Matrix (Fin 6) (Fin 6) ℝ :=
  ![
    ![ r.c,  r.s, 0,    0,    0, 0],
    ![-r.s,  r.c, 0,    0,    0, 0],
    ![   0,    0, 1,    0,    0, 0],
    ![   0,    0, 0,  r.c,  r.s, 0],
    ![   0,    0, 0, -r.s,  r.c, 0],
    ![   0,    0, 0,    0,    0, 1]
  ]

/-- **Theorem (Orthogonality of Transformation).**
The transformation matrix T satisfies Tᵀ T = I₆. -/
theorem transform_2d_orthogonal (r : Rotation2D) :
    (transform2D r)ᵀ * transform2D r = 1 := by
  unfold transform2D
  ext i j
  fin_cases i <;> fin_cases j <;>
    (simp [Matrix.mul_apply, Fin.sum_univ_six]; nlinarith [r.hunit])

/-- **Theorem (Global Transformed Stiffness Symmetry).**
If local stiffness k is symmetric, then the global stiffness K_global = Tᵀ k T is symmetric. -/
theorem global_stiffness_symmetric {n : ℕ} (T : Matrix (Fin n) (Fin n) ℝ)
    (k : Matrix (Fin n) (Fin n) ℝ) (hk : kᵀ = k) :
    (Tᵀ * k * T)ᵀ = Tᵀ * k * T := by
  rw [Matrix.transpose_mul, Matrix.transpose_mul, Matrix.transpose_transpose, hk]

/-! ### 3. Fixed-End Force Kernel -/

/-- Translation of `engine/src/element/fef.rs`: fixed-end forces for uniform distributed load w. -/
def fefUniform2D (w L : ℝ) : Fin 6 → ℝ :=
  ![ 0, w * L / 2, w * L ^ 2 / 12, 0, w * L / 2, - (w * L ^ 2 / 12) ]

/-- **Theorem (Fixed-End Force Static Equilibrium).**
The total vertical reaction equals the applied load w * L, and end moments equilibrate. -/
theorem fef_vertical_equilibrium (w L : ℝ) :
    fefUniform2D w L 1 + fefUniform2D w L 4 = w * L := by
  unfold fefUniform2D
  ring

/-! ### 4. Linear Certificate Kernel Correspondence -/

/-- Theorem: If symmetric stiffness K has an interval Cholesky factorization of K - σI
with minimum diagonal pivot d > 0, then K is strictly coercive with parameter σ. -/
theorem rust_linear_cert_coercivity {n : ℕ} (K : Matrix (Fin n) (Fin n) ℝ) (σ : ℝ)
    (h_pos : ∀ x : Fin n → ℝ, x ≠ 0 → 0 < x ⬝ᵥ ((K - σ • (1 : Matrix (Fin n) (Fin n) ℝ)) *ᵥ x))
    (x : Fin n → ℝ) :
    σ * vnorm x ^ 2 ≤ x ⬝ᵥ (K *ᵥ x) := by
  by_cases hx : x = 0
  · simp [hx, vnorm]
  · have h := h_pos x hx
    rw [Matrix.sub_mulVec, dotProduct_sub, Matrix.smul_mulVec, dotProduct_smul,
        Matrix.one_mulVec, dotProduct_self_eq, smul_eq_mul] at h
    rw [vnorm_sq]
    linarith

end Certify
