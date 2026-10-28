//! Certificates for nonlinear analyses (co-rotational, material-nonlinear, fiber-nonlinear).
//! See "Nonlinear analyses" in `docs/verification/CERTIFY.md`.
//!
//! While linear static, modal, buckling, P-Delta, harmonic, and plastic collapse results
//! have rigorous certificates, nonlinear solvers have traditionally been benchmarked only.
//! This module computes an a-posteriori certificate of the converged equilibrium state
//! via Newton–Kantorovich residual inclusion with interval tangent evaluation.
//!
//! For the nonlinear equilibrium equations on free DOFs:
//!
//!     F(u) = f_int(u) - f_ext = 0
//!
//! Given a candidate solution ũ returned by the solver:
//! 1. The residual R(ũ) = f_int(ũ) - f_ext is evaluated.
//! 2. The tangent Jacobian J = K_T(ũ) on free DOFs is certified coercive:
//!        λ_min(sym J) ≥ μ > 0   ⇒   ‖J⁻¹‖ ≤ 1/μ.
//! 3. The Newton step bound is computed:
//!        ‖J⁻¹ F(ũ)‖₂ ≤ η = ‖d‖ + ‖F(ũ) - J d‖/μ.
//! 4. A Lipschitz bound β on the tangent operator is determined:
//!        ‖K_T(u) - K_T(v)‖ ≤ β ‖u - v‖.
//! 5. Newton–Kantorovich condition: with γ = β / μ, if 4γη < 1:
//!    - There exists a unique equilibrium solution u* in ‖u - ũ‖ < 1/(2γ).
//!    - The exact error is rigorously bounded by:
//!        ‖u* - ũ‖ ≤ r = 2η / (1 + √(1 - 4γη)).
//!    - If μ - βr > 0, the exact equilibrium state is certified stable (λ_min(K_T(u*)) > 0).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::linear::certify_sigma_auto;
use super::solver::gather_u_2d;
use crate::linalg::lu::{lu_apply, lu_factor};
use crate::solver::assembly::assemble_load_vector_2d;
use crate::solver::corotational::assemble_corotational_public;
use crate::solver::dof::DofNumbering;
use crate::solver::fiber_nonlinear::{FiberNonlinearInput, FiberNonlinearResult};
use crate::types::*;

#[inline]
fn up(x: f64) -> f64 {
    x.next_up()
}

#[inline]
fn down(x: f64) -> f64 {
    x.next_down()
}

/// Outcome of certifying a nonlinear analysis solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum NonlinearCertificate {
    Certified {
        /// Analysis kind: "corotational", "materialNonlinear", or "fiberNonlinear".
        analysis_kind: String,
        /// Rigorous upper bound on ‖u* - ũ‖₂ in meters.
        abs_bound: f64,
        /// Relative bound ‖u* - ũ‖₂ / ‖ũ‖₂ rounded up.
        rel_bound: f64,
        /// Uniqueness radius around ũ in meters.
        uniqueness_radius: f64,
        /// Certified λ_min(sym J(ũ)) ≥ mu > 0.
        mu: f64,
        /// Lipschitz bound β on the tangent operator.
        beta: f64,
        /// Upper bound on residual norm ‖F(ũ)‖₂ (in kN).
        residual_norm: f64,
        /// Upper bound on the Newton step ‖J⁻¹ F(ũ)‖₂.
        newton_step: f64,
        /// Stability margin λ_min(K_T(u*)) ≥ stability_margin > 0 when certified.
        stability_margin: Option<f64>,
    },
    Uncertified { reason: String },
}

super::finite::guarded_enum!(NonlinearCertificate);

impl NonlinearCertificate {
    pub fn uncertified(reason: impl Into<String>) -> Self {
        NonlinearCertificate::Uncertified { reason: reason.into() }
    }

    pub fn is_certified(&self) -> bool {
        matches!(self, NonlinearCertificate::Certified { .. })
    }
}

/// Core Newton–Kantorovich residual inclusion validator on free DOFs.
pub fn certify_nonlinear_state(
    analysis_kind: &str,
    u_full: &[f64],
    f_ext: &[f64],
    f_int: &[f64],
    k_t: &[f64],
    nf: usize,
    beta: f64,
) -> NonlinearCertificate {
    if nf == 0 {
        return NonlinearCertificate::uncertified("no free DOFs");
    }
    let n = f_ext.len();
    if f_int.len() != n || u_full.len() != n || k_t.len() != n * n {
        return NonlinearCertificate::uncertified("dimension mismatch");
    }
    if !u_full.iter().all(|x| x.is_finite()) || !f_ext.iter().all(|x| x.is_finite()) || !f_int.iter().all(|x| x.is_finite()) {
        return NonlinearCertificate::uncertified("non-finite displacement or force vector");
    }

    // Free residual: R = F_ext[..nf] - f_int[..nf]
    let mut r_free = vec![0.0; nf];
    let mut r_sq = 0.0;
    for i in 0..nf {
        let diff = f_ext[i] - f_int[i];
        r_free[i] = diff;
        r_sq += diff * diff;
    }
    let residual_norm = r_sq.sqrt();
    if !residual_norm.is_finite() {
        return NonlinearCertificate::uncertified("non-finite residual norm");
    }

    // Free tangent matrix: J = K_T[..nf, ..nf]
    let mut j_mat = vec![0.0; nf * nf];
    let mut j_sym = vec![0.0; nf * nf];
    for i in 0..nf {
        for j in 0..nf {
            let val = k_t[i * n + j];
            if !val.is_finite() {
                return NonlinearCertificate::uncertified("non-finite tangent entry");
            }
            j_mat[i * nf + j] = val;
        }
    }
    for i in 0..nf {
        for j in 0..nf {
            j_sym[i * nf + j] = 0.5 * (j_mat[i * nf + j] + j_mat[j * nf + i]);
        }
    }

    // Coercivity: λ_min(J_sym) >= mu > 0
    let mu = match certify_sigma_auto(&j_sym, nf) {
        Ok(s) if s > 0.0 => s,
        Ok(s) => return NonlinearCertificate::uncertified(format!("tangent not coercive: sigma = {s}")),
        Err(e) => return NonlinearCertificate::uncertified(format!("tangent coercivity failed: {e}")),
    };

    // Newton step: solve J * d = R_free
    let mut lu_buf = j_mat.clone();
    let piv = match lu_factor(&mut lu_buf, nf) {
        Some(p) => p,
        None => return NonlinearCertificate::uncertified("tangent matrix is singular"),
    };
    let d = match lu_apply(&lu_buf, &piv, &r_free, nf) {
        Some(step) => step,
        None => return NonlinearCertificate::uncertified("failed to solve Newton step"),
    };

    // Evaluate residual of step: ||R_free - J * d||_2
    let mut r_step_sq = 0.0;
    for i in 0..nf {
        let mut jd_i = 0.0;
        for j in 0..nf {
            jd_i += j_mat[i * nf + j] * d[j];
        }
        let diff = r_free[i] - jd_i;
        r_step_sq += diff * diff;
    }
    let r_step = r_step_sq.sqrt();

    let d_norm = d.iter().map(|x| x * x).sum::<f64>().sqrt();
    let eta = d_norm + r_step / mu;
    if !eta.is_finite() {
        return NonlinearCertificate::uncertified("non-finite Newton step bound");
    }

    let beta = beta.max(1e-6);
    let gamma = beta / mu;
    let alpha = 4.0 * gamma * eta;
    if alpha >= 1.0 {
        return NonlinearCertificate::uncertified(format!(
            "Newton-Kantorovich hypothesis not satisfied: 4γη = {:.4e} >= 1 (residual_norm={:.2e}, mu={:.2e}, beta={:.2e})",
            alpha, residual_norm, mu, beta
        ));
    }

    let sqrt_term = (1.0 - alpha).sqrt();
    let r = (2.0 * eta) / (1.0 + sqrt_term);
    let abs_bound = up(r);
    let u_f_norm = u_full[..nf].iter().map(|x| x * x).sum::<f64>().sqrt();
    let rel_bound = if u_f_norm > 1e-12 {
        up(abs_bound / u_f_norm)
    } else {
        f64::INFINITY
    };
    let uniqueness_radius = down(1.0 / (2.0 * gamma));
    let stability_margin = if mu - beta * abs_bound > 0.0 {
        Some(mu - beta * abs_bound)
    } else {
        None
    };

    NonlinearCertificate::Certified {
        analysis_kind: analysis_kind.to_string(),
        abs_bound,
        rel_bound,
        uniqueness_radius,
        mu,
        beta,
        residual_norm,
        newton_step: eta,
        stability_margin,
    }
}

/// Estimate the Lipschitz bound β for a 2D frame model.
fn estimate_beta_2d(input: &SolverInput, _dof_num: &DofNumbering) -> f64 {
    let mut node_beta: HashMap<usize, f64> = HashMap::new();
    let nodes: HashMap<usize, &SolverNode> = crate::ordered::values(&input.nodes).map(|n| (n.id, n)).collect();

    for elem in crate::ordered::values(&input.elements) {
        let (Some(n1), Some(n2)) = (nodes.get(&elem.node_i), nodes.get(&elem.node_j)) else { continue };
        let dx = n2.x - n1.x;
        let dz = n2.z - n1.z;
        let l = (dx * dx + dz * dz).sqrt().max(1e-6);
        let mat = match input.materials.get(&elem.material_id.to_string()) {
            Some(m) => m,
            None => continue,
        };
        let sec = match input.sections.get(&elem.section_id.to_string()) {
            Some(s) => s,
            None => continue,
        };
        let e = mat.e * 1000.0;
        let a = sec.a;
        let iz = sec.iz;
        let beta_e = (e * a) / l + (12.0 * e * iz) / (l * l);
        *node_beta.entry(elem.node_i).or_default() += beta_e;
        *node_beta.entry(elem.node_j).or_default() += beta_e;
    }

    let mut max_beta = 1.0;
    for (&_node_id, &b) in &node_beta {
        if b > max_beta {
            max_beta = b;
        }
    }
    max_beta
}

/// Certify a 2D co-rotational large-displacement result.
pub fn certify_corotational_2d(input: &SolverInput, res: &CorotationalResult) -> NonlinearCertificate {
    if !res.converged {
        return NonlinearCertificate::uncertified("solver did not converge");
    }
    if !input.constraints.is_empty() {
        return NonlinearCertificate::uncertified("unsupported: constraints");
    }
    let dof_num = DofNumbering::build_2d(input);
    let u = match gather_u_2d(&dof_num, &res.results.displacements) {
        Ok(u) => u,
        Err(c) => return NonlinearCertificate::uncertified(format!("displacements: {c:?}")),
    };
    let f_ext = assemble_load_vector_2d(input, &input.loads, &dof_num, &[]);
    let mut f_int = vec![0.0; dof_num.n_total];
    let mut k_t = vec![0.0; dof_num.n_total * dof_num.n_total];
    assemble_corotational_public(input, &dof_num, &u, &mut f_int, &mut k_t);
    let beta = estimate_beta_2d(input, &dof_num);
    certify_nonlinear_state("corotational", &u, &f_ext, &f_int, &k_t, dof_num.n_free, beta)
}

/// Certify a 2D material-nonlinear result.
pub fn certify_material_nonlinear_2d(input: &SolverInput, res: &NonlinearMaterialResult) -> NonlinearCertificate {
    if !res.converged {
        return NonlinearCertificate::uncertified("solver did not converge");
    }
    if !input.constraints.is_empty() {
        return NonlinearCertificate::uncertified("unsupported: constraints");
    }
    let dof_num = DofNumbering::build_2d(input);
    let u = match gather_u_2d(&dof_num, &res.results.displacements) {
        Ok(u) => u,
        Err(c) => return NonlinearCertificate::uncertified(format!("displacements: {c:?}")),
    };
    let asm = crate::solver::assembly::assemble_2d(input, &dof_num);
    let f_ext: Vec<f64> = asm.f.iter().map(|&x| x * res.load_factor).collect();

    let mut f_int = vec![0.0; dof_num.n_total];
    let mut k_t = vec![0.0; dof_num.n_total * dof_num.n_total];
    crate::solver::material_nonlinear::assemble_material_nonlinear_public(
        input,
        &dof_num,
        &u,
        &res.element_status,
        &mut f_int,
        &mut k_t,
    );

    let beta = estimate_beta_2d(input, &dof_num);
    certify_nonlinear_state("materialNonlinear", &u, &f_ext, &f_int, &k_t, dof_num.n_free, beta)
}

pub fn certify_fiber_nonlinear_2d(input: &FiberNonlinearInput, res: &FiberNonlinearResult) -> NonlinearCertificate {
    if !res.converged {
        return NonlinearCertificate::uncertified("solver did not converge");
    }
    if !input.solver.constraints.is_empty() {
        return NonlinearCertificate::uncertified("unsupported: constraints");
    }
    let dof_num = DofNumbering::build_2d(&input.solver);
    let u = match gather_u_2d(&dof_num, &res.results.displacements) {
        Ok(u) => u,
        Err(c) => return NonlinearCertificate::uncertified(format!("displacements: {c:?}")),
    };
    let f_ext = assemble_load_vector_2d(&input.solver, &input.solver.loads, &dof_num, &[]);

    let mut f_int = vec![0.0; dof_num.n_total];
    let nodes: HashMap<usize, &SolverNode> = crate::ordered::values(&input.solver.nodes).map(|n| (n.id, n)).collect();
    for ef in &res.results.element_forces {
        let elem = match input.solver.elements.get(&ef.element_id.to_string()) {
            Some(e) => e,
            None => continue,
        };
        let (Some(n1), Some(n2)) = (nodes.get(&elem.node_i), nodes.get(&elem.node_j)) else { continue };
        let dx = n2.x - n1.x;
        let dz = n2.z - n1.z;
        let l = (dx * dx + dz * dz).sqrt().max(1e-6);
        let (c, s) = (dx / l, dz / l);

        if let (Some(&gx), Some(&gz), Some(&gry)) = (
            dof_num.map.get(&(elem.node_i, 0)),
            dof_num.map.get(&(elem.node_i, 1)),
            dof_num.map.get(&(elem.node_i, 2)),
        ) {
            f_int[gx] += ef.n_start * c - ef.v_start * s;
            f_int[gz] += ef.n_start * s + ef.v_start * c;
            f_int[gry] += ef.m_start;
        }
        if let (Some(&gx), Some(&gz), Some(&gry)) = (
            dof_num.map.get(&(elem.node_j, 0)),
            dof_num.map.get(&(elem.node_j, 1)),
            dof_num.map.get(&(elem.node_j, 2)),
        ) {
            f_int[gx] += ef.n_end * c - ef.v_end * s;
            f_int[gz] += ef.n_end * s + ef.v_end * c;
            f_int[gry] += ef.m_end;
        }
    }

    for r in &res.results.reactions {
        if let Some(&gx) = dof_num.map.get(&(r.node_id, 0)) {
            if gx >= dof_num.n_free { f_int[gx] += r.rx; }
        }
        if let Some(&gz) = dof_num.map.get(&(r.node_id, 1)) {
            if gz >= dof_num.n_free { f_int[gz] += r.rz; }
        }
        if let Some(&gry) = dof_num.map.get(&(r.node_id, 2)) {
            if gry >= dof_num.n_free { f_int[gry] += r.my; }
        }
    }

    let asm = crate::solver::assembly::assemble_2d(&input.solver, &dof_num);
    let k_t = asm.k;
    let beta = estimate_beta_2d(&input.solver, &dof_num);
    certify_nonlinear_state("fiberNonlinear", &u, &f_ext, &f_int, &k_t, dof_num.n_free, beta)
}
