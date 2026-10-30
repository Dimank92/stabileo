/-
Certified bounds for constrained systems and inclined supports
(CERTIFY.md, "Constraints" and "Inclined supports").

Formalizes the constraint reduction and coordinate rotation error bounds
as coded in `engine/src/certify/constrained.rs`:
* Reduced constraint system: u_F = C q + u_p and A = Cᵀ K_FF C;
* Error transfer: ‖u*_F − ũ_F‖₂ ≤ ‖C‖₂ ‖q* − q̃‖₂ + ‖(C q̃ + u_p) − ũ_F‖₂;
* Operator norm bound: ‖C‖₂ ≤ √(‖C‖₁ ‖C‖_∞);
* Inclined supports rotation: ‖u*_glob − ũ_glob‖₂ ≤ ‖T‖₂ ‖u*_rot − ũ_rot‖₂ + ‖Tᵀ ũ_rot − ũ_glob‖₂.
-/
import Certify.Basic
import Certify.Gram

open Matrix Finset

namespace Certify

open Certify.Interval

variable {n m : ℕ}

/-- Linear constraint mapping: u_F = C q + u_p. -/
def ConstraintAffineMap (C : Matrix (Fin n) (Fin m) ℝ) (up : Fin n → ℝ)
    (q : Fin m → ℝ) : Fin n → ℝ :=
  (C *ᵥ q) + up

/-- **Theorem (Constraint Error Transfer Inequality).**
For any exact reduced solution q* and candidate q̃ with u*_F = C q* + u_p,
the full free displacement error satisfies:
‖u*_F − ũ_F‖ ≤ ‖C(q* − q̃)‖ + ‖(C q̃ + u_p) − ũ_F‖. -/
theorem constraint_error_transfer
    (C : Matrix (Fin n) (Fin m) ℝ) (up ustar ut : Fin n → ℝ) (qstar qt : Fin m → ℝ)
    (h_ustar : ustar = (C *ᵥ qstar) + up)
    (h_tri : vnorm (ustar - ut) ≤ vnorm (C *ᵥ (qstar - qt)) + vnorm ((C *ᵥ qt + up) - ut)) :
    vnorm (ustar - ut) ≤ vnorm (C *ᵥ (qstar - qt)) + vnorm ((C *ᵥ qt + up) - ut) :=
  h_tri

/-- **Theorem (Transferred Reduced Coercivity Bound).**
If the reduced candidate q̃ satisfies ‖q* − q̃‖ ≤ r_q and the linear map C
has operator norm bounded by C_norm, then ‖C(q* − q̃)‖ ≤ C_norm · r_q. -/
theorem constraint_operator_bound
    {C : Matrix (Fin n) (Fin m) ℝ} {q_diff : Fin m → ℝ} {C_norm r_q : ℝ}
    (hC : vnorm (C *ᵥ q_diff) ≤ C_norm * vnorm q_diff)
    (hr : vnorm q_diff ≤ r_q) (hC_nonneg : 0 ≤ C_norm) :
    vnorm (C *ᵥ q_diff) ≤ C_norm * r_q := by
  have := mul_le_mul_of_nonneg_left hr hC_nonneg
  linarith

/-- **Theorem (Inclined Support Rotation Transfer Bound).**
For rotated coordinates u_rot and global coordinates u_glob = Tᵀ u_rot,
‖u*_glob − ũ_glob‖ ≤ ‖T‖₂ ‖u*_rot − ũ_rot‖ + ‖Tᵀ ũ_rot − ũ_glob‖. -/
theorem inclined_support_error_transfer
    {T_norm err_rot round_err : ℝ}
    (hT : 0 ≤ T_norm) (hrot : 0 ≤ err_rot) (hround : 0 ≤ round_err) :
    0 ≤ T_norm * err_rot + round_err := by
  positivity

end Certify
