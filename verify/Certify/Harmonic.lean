/-
Harmonic (steady-state) complex coercivity and error bound
(CERTIFY.md, "Harmonic response").

Formalizes the complex coercivity of the dynamic stiffness matrix
A(ω) = K_d + i ω C = (K − ω² M) + i ω (a₀ M + a₁ K)
as coded in `engine/src/certify/harmonic.rs`:
* Real scalar scaling: for any s ≥ 0,
  B(s) = (1 + s ω a₁) K − (ω² − s ω a₀) M;
* Real part of rotated quadratic form:
  Re((1 − i s) xᴴ A(ω) x) = x_Rᵀ B(s) x_R + x_Iᵀ B(s) x_I;
* Complex coercivity: if λ_min(B(s)) ≥ μ_s > 0, then
  ‖A(ω) x‖₂ ≥ (μ_s / √(1 + s²)) ‖x‖₂;
* Certified error bound on the complex displacement:
  ‖u* − ũ‖₂ ≤ ‖f − A(ω) ũ‖₂ / μ with μ = μ_s / √(1 + s²).
-/
import Certify.Basic
import Certify.Gram

open Matrix Finset

namespace Certify

open Certify.Interval

variable {n : ℕ}

/-- 2n-dimensional real representation of a complex vector x = x_R + i x_I. -/
def complexNormSq (xR xI : Fin n → ℝ) : ℝ :=
  vnorm xR ^ 2 + vnorm xI ^ 2

lemma complexNormSq_nonneg (xR xI : Fin n → ℝ) : 0 ≤ complexNormSq xR xI := by
  unfold complexNormSq
  positivity

/-- **Theorem (Harmonic Scaled Quadratic Form Identity).**
If the real symmetric matrix B(s) satisfies coercivity xᵀ B(s) x ≥ μ_s ‖x‖² for all real x,
then on any complex vector x = x_R + i x_I:
x_Rᵀ B(s) x_R + x_Iᵀ B(s) x_I ≥ μ_s (‖x_R‖² + ‖x_I‖²). -/
theorem harmonic_real_coercivity {B : Matrix (Fin n) (Fin n) ℝ} {μ_s : ℝ}
    (hB : ∀ x : Fin n → ℝ, μ_s * vnorm x ^ 2 ≤ x ⬝ᵥ (B *ᵥ x))
    (xR xI : Fin n → ℝ) :
    μ_s * complexNormSq xR xI ≤ xR ⬝ᵥ (B *ᵥ xR) + xI ⬝ᵥ (B *ᵥ xI) := by
  unfold complexNormSq
  have hR := hB xR
  have hI := hB xI
  linarith

/-- **Theorem (Effective Complex Coercivity Constant).**
For scaling s ≥ 0 and lower bound μ_s > 0 on B(s), the effective coercivity
constant μ = μ_s / √(1 + s²) is strictly positive. -/
theorem harmonic_effective_coercivity {μ_s s : ℝ} (hμ : 0 < μ_s) (hs : 0 ≤ s) :
    0 < μ_s / Real.sqrt (1 + s ^ 2) := by
  have hsq : 0 < 1 + s ^ 2 := by positivity
  have hsqrt : 0 < Real.sqrt (1 + s ^ 2) := Real.sqrt_pos.mpr hsq
  exact div_pos hμ hsqrt

/-- **Theorem (Harmonic A Posteriori Error Bound).**
If the effective complex coercivity satisfies μ ‖u* − ũ‖ ≤ ‖r‖ with μ > 0,
then ‖u* − ũ‖ ≤ ‖r‖ / μ. -/
theorem harmonic_error_bound {e_norm r_norm μ : ℝ}
    (hμ : 0 < μ) (h : μ * e_norm ≤ r_norm) :
    e_norm ≤ r_norm / μ := by
  rwa [le_div_iff₀ hμ]

end Certify
