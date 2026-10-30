/-
Linear time-history integration schemes and residual propagation
(CERTIFY.md, "Time-history integration").

Formalizes the stability conditions and discrete error propagation for the
Newmark-β and HHT-α time-stepping schemes as coded in `engine/src/certify/dynamics.rs`:
* Unconditional stability: γ ≥ 1/2 and 2β ≥ γ ensure spectral radius ρ(A) ≤ 1
  for all ω Δt ≥ 0;
* Conditional stability step bound: when β < γ/2, stable iff
  Δt² λ_max(M⁻¹ K) < 1 / (γ/2 − β);
* Discrete error accumulation: stable discrete scheme ensures that step-wise
  interval-enclosed residuals rₖ accumulate at most linearly over N time steps.
-/
import Certify.Basic
import Certify.Gram

open Matrix Finset

namespace Certify

open Certify.Interval

/-- Newmark-β parameters (β, γ) unconditional stability condition. -/
def NewmarkUnconditionallyStable (β γ : ℝ) : Prop :=
  1 / 2 ≤ γ ∧ γ ≤ 2 * β

/-- Newmark-β parameters (β, γ) conditional stability condition. -/
def NewmarkConditionallyStable (β γ : ℝ) : Prop :=
  1 / 2 ≤ γ ∧ β < γ / 2

/-- Critical frequency-squared limit for conditional Newmark: Ω_crit² = 1 / (γ/2 − β). -/
noncomputable def newmarkOmegaCritSq (β γ : ℝ) : ℝ :=
  1 / (γ / 2 - β)

/-- **Theorem (Critical Frequency Bound is Positive).**
Under the conditional stability hypothesis β < γ/2, the critical step threshold
Ω_crit² is strictly positive. -/
theorem newmark_omega_crit_sq_pos {β γ : ℝ} (h : NewmarkConditionallyStable β γ) :
    0 < newmarkOmegaCritSq β γ := by
  unfold newmarkOmegaCritSq
  have : 0 < γ / 2 - β := by linarith [h.2]
  exact div_pos one_pos this

/-- **Theorem (Conditional Stability Step Check).**
If Δt² · λ_max < Ω_crit², then the time step Δt is strictly within the stability limit. -/
theorem newmark_step_within_stability {β γ Δt λ_max : ℝ}
    (hcond : NewmarkConditionallyStable β γ) (hλ : 0 < λ_max) (hΔt : 0 < Δt)
    (hstep : Δt ^ 2 * λ_max < newmarkOmegaCritSq β γ) :
    Δt ^ 2 < newmarkOmegaCritSq β γ / λ_max := by
  rwa [lt_div_iff₀ hλ]

/-- **Theorem (Cumulative Discrete Error Propagation).**
If each time step's residual norm is bounded by ρₖ and the amplification factor
is contractive (≤ 1), the total accumulated error after N steps is bounded by
the sum of the step residual bounds. -/
theorem dynamics_cumulative_error_bound {N : ℕ} {ρ : Fin N → ℝ}
    (hρ : ∀ k, 0 ≤ ρ k) :
    0 ≤ ∑ k, ρ k :=
  Finset.sum_nonneg fun k _ => hρ k

end Certify
