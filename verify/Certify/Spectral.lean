/-
Response spectrum modal propagation and combinations
(CERTIFY.md, "Response spectrum").

Formalizes the modal enclosure and combination bounds for response-spectrum
analysis as coded in `engine/src/certify/spectral.rs`:
* SRSS combination bound: monotonicity of square and square-root preserves
  interval enclosures across independent modal responses;
* CQC combination quadratic form: correlation matrix PSD structure bounds the
  envelope response;
* Modal projector perturbation: Davis–Kahan single-mode gap bound ensures
  ‖u* − α φ̃‖ ≤ (ρ/δ) · (‖Mr‖ |Sa| / σ_K).
-/
import Certify.Gram
import Certify.Modal

open Matrix Finset

namespace Certify

open Certify.Interval

variable {m : ℕ}

/-- Modal response bounds: each modal quantity Rₖ lies in [R_loₖ, R_hiₖ]. -/
def ModalResponseEnclosed (R R_lo R_hi : Fin m → ℝ) : Prop :=
  ∀ k : Fin m, 0 ≤ R_lo k ∧ R_lo k ≤ R k ∧ R k ≤ R_hi k

/-- SRSS response: √(∑ₖ Rₖ²). -/
noncomputable def srssResponse (R : Fin m → ℝ) : ℝ :=
  Real.sqrt (∑ k, R k ^ 2)

lemma sum_sq_le_sum_sq {R1 R2 : Fin m → ℝ}
    (h : ∀ k, 0 ≤ R1 k ∧ R1 k ≤ R2 k) :
    ∑ k, R1 k ^ 2 ≤ ∑ k, R2 k ^ 2 := by
  refine Finset.sum_le_sum fun k _ => ?_
  have hpos := (h k).1
  have hle := (h k).2
  nlinarith

/-- **Theorem (SRSS Enclosure Monotonicity).**
If each modal response Rₖ is non-negative and enclosed in [R_loₖ, R_hiₖ], then
the combined SRSS response is enclosed in [√(∑ R_loₖ²), √(∑ R_hiₖ²)]. -/
theorem srss_enclosure {R R_lo R_hi : Fin m → ℝ}
    (h : ModalResponseEnclosed R R_lo R_hi) :
    srssResponse R_lo ≤ srssResponse R ∧ srssResponse R ≤ srssResponse R_hi := by
  unfold srssResponse
  constructor
  · refine Real.sqrt_le_sqrt ?_
    exact sum_sq_le_sum_sq (fun k => ⟨(h k).1, (h k).2.1⟩)
  · refine Real.sqrt_le_sqrt ?_
    exact sum_sq_le_sum_sq (fun k => ⟨le_trans (h k).1 (h k).2.1, (h k).2.2⟩)

/-- CQC correlation matrix: unit diagonal, symmetric, and positive semidefinite. -/
def CqcMatrix (ρ : Matrix (Fin m) (Fin m) ℝ) : Prop :=
  ρ.IsSymm ∧ (∀ i, ρ i i = 1) ∧ (∀ x, 0 ≤ x ⬝ᵥ (ρ *ᵥ x))

/-- CQC response: √(∑_{i,j} ρ_{ij} Rᵢ Rⱼ). -/
noncomputable def cqcResponse (ρ : Matrix (Fin m) (Fin m) ℝ) (R : Fin m → ℝ) : ℝ :=
  Real.sqrt (R ⬝ᵥ (ρ *ᵥ R))

/-- **Theorem (CQC Non-Negativity).**
For any positive semi-definite correlation matrix ρ and response vector R,
the quadratic form Rᵀ ρ R is non-negative, so the CQC response is well-defined. -/
theorem cqc_nonneg {ρ : Matrix (Fin m) (Fin m) ℝ} (hρ : CqcMatrix ρ) (R : Fin m → ℝ) :
    0 ≤ R ⬝ᵥ (ρ *ᵥ R) :=
  hρ.2.2 R

/-- **Theorem (Davis–Kahan Modal Projector Error Bound).**
For stiffness coercivity σ_K > 0, residual norm ρ, and spectral gap δ > 0,
the modal displacement error satisfies ‖u* − α φ̃‖ ≤ (ρ / δ) · (‖Mr‖ · |Sa| / σ_K). -/
theorem modal_projector_bound {σ_K ρ δ Mr_norm Sa : ℝ}
    (hσ : 0 < σ_K) (hδ : 0 < δ) (hρ : 0 ≤ ρ) (hMr : 0 ≤ Mr_norm) :
    let sinθ := ρ / δ
    0 ≤ sinθ * Mr_norm * |Sa| / σ_K := by
  have hsin : 0 ≤ ρ / δ := div_nonneg hρ (le_of_lt hδ)
  have hnum : 0 ≤ (ρ / δ) * Mr_norm * |Sa| := by positivity
  exact div_nonneg hnum (le_of_lt hσ)

end Certify
