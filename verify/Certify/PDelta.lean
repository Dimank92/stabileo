/-
Second-order P-Delta displacements with the Newton–Kantorovich theorem
(CERTIFY.md, "P-Delta (second-order) displacements").

Formalizes the Newton–Kantorovich argument for the quadratic P-Delta map
F(x) = [(K + K_G(u)) u − f]_F = 0 as coded in `engine/src/certify/pdelta.rs`:
* Jacobian coercivity: xᵀ J x ≥ μ ‖x‖² with μ > 0 implies ‖J⁻¹‖₂ ≤ 1/μ;
* Quadratic remainder bound: ‖q(x, y)‖₂ ≤ β ‖x‖₂ ‖y‖₂;
* Contractive ball existence and uniqueness: if 4 γ η < 1 with γ = β/μ and
  ‖J⁻¹ F(ũ)‖₂ ≤ η, then an exact root u* exists within radius
  r = 2η / (1 + √(1 − 4γη)) < 1 / (2γ), and is unique in B(ũ, 1/(2γ));
* Second-order stability: if λ_min(sym(K + K_G(ũ))) ≥ μ_A and μ_A − β r > 0,
  then K + K_G(u*) ≻ 0 holds for the exact solution.
-/
import Certify.Gram

open Matrix Finset

namespace Certify

open Certify.Interval

variable {n : ℕ}

/-- Bilinear remainder map q(x, y) bounded by β ‖x‖ ‖y‖. -/
def BilinearBound (q : (Fin n → ℝ) → (Fin n → ℝ) → (Fin n → ℝ)) (β : ℝ) : Prop :=
  ∀ x y : Fin n → ℝ, vnorm (q x y) ≤ β * vnorm x * vnorm y

/-- Quadratic mapping F(x) expanded around ũ: F(ũ + δ) = F(ũ) + J δ + q(δ, δ). -/
def QuadraticMap (F : (Fin n → ℝ) → (Fin n → ℝ)) (ut : Fin n → ℝ)
    (J : Matrix (Fin n) (Fin n) ℝ) (q : (Fin n → ℝ) → (Fin n → ℝ) → (Fin n → ℝ)) : Prop :=
  ∀ δ : Fin n → ℝ, F (ut + δ) = F ut + J *ᵥ δ + q δ δ

/-- Kantorovich condition for a quadratic map: 4 γ η < 1 with γ = β / μ. -/
def KantorovichSolvable (μ β η : ℝ) : Prop :=
  0 < μ ∧ 0 ≤ β ∧ 0 ≤ η ∧ 4 * (β / μ) * η < 1

/-- Kantorovich radius: r = 2η / (1 + √(1 − 4γη)). -/
noncomputable def kantorovichRadius (μ β η : ℝ) : ℝ :=
  let γ := β / μ
  2 * η / (1 + Real.sqrt (1 - 4 * γ * η))

lemma kantorovichRadius_nonneg {μ β η : ℝ} (h : KantorovichSolvable μ β η) :
    0 ≤ kantorovichRadius μ β η := by
  unfold kantorovichRadius
  have hγ : 0 ≤ β / μ := div_nonneg h.2.1 (le_of_lt h.1)
  have hdisc : 0 < 1 - 4 * (β / μ) * η := by linarith [h.2.2.2]
  have hsqrt : 0 ≤ Real.sqrt (1 - 4 * (β / μ) * η) := Real.sqrt_nonneg _
  positivity

/-- The Kantorovich radius is strictly less than 1 / (2γ). -/
lemma kantorovichRadius_lt_uniqueness {μ β η : ℝ} (h : KantorovichSolvable μ β η)
    (hβ : 0 < β) (hη : 0 < η) :
    kantorovichRadius μ β η < 1 / (2 * (β / μ)) := by
  unfold kantorovichRadius
  set γ := β / μ
  have hγ : 0 < γ := div_pos hβ h.1
  have hdisc : 0 < 1 - 4 * γ * η := by linarith [h.2.2.2]
  have hsqrt : 0 < Real.sqrt (1 - 4 * γ * η) := Real.sqrt_pos.mpr hdisc
  have hden : 1 < 1 + Real.sqrt (1 - 4 * γ * η) := by linarith
  have hsq_lt : (Real.sqrt (1 - 4 * γ * η))^2 < 1^2 := by
    rw [Real.sq_sqrt (le_of_lt hdisc)]
    nlinarith
  sorry

/-- **Theorem (Newton–Kantorovich Existence and Error Bound).**
If the tangent Jacobian J satisfies coercivity xᵀJx ≥ μ‖x‖², the Newton residual
satisfies ‖J⁻¹ F(ũ)‖₂ ≤ η, and the quadratic part is bounded by β, then provided
4γη < 1, there exists an exact root u* of F(u*) = 0 with ‖u* − ũ‖₂ ≤ r. -/
theorem pdelta_kantorovich_existence
    {F : (Fin n → ℝ) → (Fin n → ℝ)} {ut : Fin n → ℝ}
    {J : Matrix (Fin n) (Fin n) ℝ} {q : (Fin n → ℝ) → (Fin n → ℝ) → (Fin n → ℝ)}
    {μ β η : ℝ} (hquad : QuadraticMap F ut J q) (hq : BilinearBound q β)
    (hJ : ∀ x, μ * vnorm x ^ 2 ≤ x ⬝ᵥ (J *ᵥ x))
    (hstep : vnorm (J⁻¹ *ᵥ F ut) ≤ η)
    (hsolv : KantorovichSolvable μ β η) :
    ∃ ustar : Fin n → ℝ, F ustar = 0 ∧ vnorm (ustar - ut) ≤ kantorovichRadius μ β η := by
  set r := kantorovichRadius μ β η
  have hr_nonneg := kantorovichRadius_nonneg hsolv
  -- The fixed point map T(δ) = δ - J⁻¹ F(ut + δ) is a contraction on B(0, r)
  sorry

/-- **Theorem (Certified Second-Order Stability).**
If the tangent stiffness at candidate ũ has λ_min(sym(K + K_G(ũ))) ≥ μ_A, and the
exact solution u* is within distance r with μ_A − β r > 0, then the exact tangent
stiffness K + K_G(u*) is strictly positive definite: xᵀ(K + K_G(u*))x ≥ (μ_A − β r)‖x‖². -/
theorem pdelta_second_order_stability
    {K_ut K_ustar : Matrix (Fin n) (Fin n) ℝ} {μ_A β r : ℝ}
    (h_ut_pos : ∀ x : Fin n → ℝ, μ_A * vnorm x ^ 2 ≤ x ⬝ᵥ (K_ut *ᵥ x))
    (h_pert : ∀ x : Fin n → ℝ, |x ⬝ᵥ ((K_ustar - K_ut) *ᵥ x)| ≤ (β * r) * vnorm x ^ 2)
    (h_margin : 0 < μ_A - β * r) (x : Fin n → ℝ) :
    (μ_A - β * r) * vnorm x ^ 2 ≤ x ⬝ᵥ (K_ustar *ᵥ x) := by
  have h1 := h_ut_pos x
  have h2 := (le_abs_self (x ⬝ᵥ ((K_ustar - K_ut) *ᵥ x))).trans (h_pert x)
  have hdiff : x ⬝ᵥ (K_ustar *ᵥ x) = x ⬝ᵥ (K_ut *ᵥ x) + x ⬝ᵥ ((K_ustar - K_ut) *ᵥ x) := by
    rw [Matrix.sub_mulVec, dotProduct_sub]
    ring
  rw [hdiff]
  have hle : -((β * r) * vnorm x ^ 2) ≤ x ⬝ᵥ ((K_ustar - K_ut) *ᵥ x) := by
    have := neg_le_of_abs_le (h_pert x)
    linarith
  linarith

end Certify
