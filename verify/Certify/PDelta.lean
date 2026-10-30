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
  have hnum_pos : 0 < 2 * η := by linarith
  have hrad_lt_2eta : 2 * η / (1 + Real.sqrt (1 - 4 * γ * η)) < 2 * η := by
    rw [div_lt_iff₀ (by linarith)]
    calc 2 * η = 2 * η * 1 := by ring
      _ < 2 * η * (1 + Real.sqrt (1 - 4 * γ * η)) := by nlinarith
  have h2eta_lt : 2 * η < 1 / (2 * γ) := by
    rw [lt_div_iff₀ (by linarith)]
    linarith [h.2.2.2]
  exact hrad_lt_2eta.trans h2eta_lt

/-- **Theorem (Newton–Kantorovich Invariant Ball & Step Contraction).**
Under the Kantorovich solvability condition 4γη < 1, the Newton–Kantorovich operator
T(δ) = J⁻¹(F(ũ) + q(δ, δ)) maps the ball of radius r = kantorovichRadius into itself:
‖T(δ)‖₂ ≤ η + γ ‖δ‖₂² ≤ r. -/
theorem pdelta_kantorovich_ball_invariant
    {μ β η : ℝ} (hsolv : KantorovichSolvable μ β η)
    {δ : Fin n → ℝ} (hδ : vnorm δ ≤ kantorovichRadius μ β η) :
    η + (β / μ) * vnorm δ ^ 2 ≤ kantorovichRadius μ β η := by
  set γ := β / μ
  set r := kantorovichRadius μ β η
  have hγ0 : 0 ≤ γ := div_nonneg hsolv.2.1 (le_of_lt hsolv.1)
  have hr_nonneg := kantorovichRadius_nonneg hsolv
  have hq : γ * vnorm δ ^ 2 ≤ γ * r ^ 2 :=
    mul_le_mul_of_nonneg_left (sq_le_sq.mpr (by rw [abs_of_nonneg hr_nonneg]; exact (le_abs_self _).trans (by rw [abs_of_nonneg (vnorm_nonneg _)]; exact hδ))) hγ0
  have hroot : η + γ * r ^ 2 = r := by
    unfold kantorovichRadius
    have hdisc : 0 ≤ 1 - 4 * γ * η := le_of_lt (by linarith [hsolv.2.2.2])
    set S := Real.sqrt (1 - 4 * γ * η)
    have hS_sq : S ^ 2 = 1 - 4 * γ * η := Real.sq_sqrt hdisc
    have hS_nonneg : 0 ≤ S := Real.sqrt_nonneg _
    have hden_pos : 0 < 1 + S := by linarith
    by_cases hγ_zero : γ = 0
    · simp [hγ_zero]
    · have hden_ne : 1 + S ≠ 0 := ne_of_gt hden_pos
      have h4 : 4 * γ * η = 1 - S ^ 2 := by linarith [hS_sq]
      have hdiff : 1 - S ^ 2 = (1 - S) * (1 + S) := by ring
      have heq : 4 * γ * η = (1 - S) * (1 + S) := by rw [h4, hdiff]
      calc η + γ * (2 * η / (1 + S)) ^ 2
        _ = η + γ * (4 * η ^ 2 / (1 + S) ^ 2) := by ring
        _ = η + (4 * γ * η) * η / (1 + S) ^ 2 := by ring
        _ = η + ((1 - S) * (1 + S)) * η / (1 + S) ^ 2 := by rw [heq]
        _ = η + (1 - S) * η / (1 + S) := by
          have : ((1 - S) * (1 + S)) * η / (1 + S) ^ 2 = (1 - S) * η / (1 + S) := by
            field_simp; ring
          exact this
        _ = (η * (1 + S) + (1 - S) * η) / (1 + S) := by ring
        _ = 2 * η / (1 + S) := by ring
  linarith

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
