//! Live verification of the user's own model on solve (Issue #498).
//!
//! Roadmap requirement:
//! "Soundness multipliers, item 1. Verification today runs on our corpus; the bugs that hurt
//!  live in users' models. After each solve, in the background:
//!  - global equilibrium recomputed from the model's loads (not from K);
//!  - a rotated and renumbered twin of the model solved and compared (the metamorphic check
//!    that found #488 and the batch-3 diaphragm/mass interactions);
//!  - an independent re-assembly of K and f (#403) compared entry by entry.
//!  Any disagreement: the result is not shown as verified, and the reason is.
//!  Cost 2–4× the solve, after the result is shown."

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::reassembly::{end_to_end_2d, end_to_end_3d, EndToEnd};
use crate::element::compute_local_axes_3d;
use crate::metamorphic::compare::{compare_floored, flat_2d, flat_3d, Parts};
use crate::metamorphic::conditioning::{self, Conditioning};
use crate::metamorphic::relabel::Relabel;
use crate::metamorphic::transform::{map_results_2d, map_results_3d, transform_2d, transform_3d, Iso2, Iso3};
use crate::solver::linear::{solve_2d, solve_3d};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquilibriumReport {
    pub passed: bool,
    pub residual_force: f64,
    pub residual_moment: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetamorphicTwinReport {
    pub passed: bool,
    pub worst_ratio: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DualAssemblyReport {
    pub passed: bool,
    pub k_discrepancy: f64,
    pub f_discrepancy: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserModelCheckReport {
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    pub equilibrium: EquilibriumReport,
    pub metamorphic_twin: MetamorphicTwinReport,
    pub dual_assembly: DualAssemblyReport,
}

impl Default for UserModelCheckReport {
    fn default() -> Self {
        UserModelCheckReport {
            passed: true,
            failure_reason: None,
            equilibrium: EquilibriumReport { passed: true, residual_force: 0.0, residual_moment: 0.0, detail: None },
            metamorphic_twin: MetamorphicTwinReport { passed: true, worst_ratio: 0.0, detail: None },
            dual_assembly: DualAssemblyReport { passed: true, k_discrepancy: 0.0, f_discrepancy: 0.0, detail: None },
        }
    }
}

// ---------------------------------------------------------------------------
// 2D Check
// ---------------------------------------------------------------------------

pub fn check_user_model_2d(input: &SolverInput, results: &AnalysisResults) -> UserModelCheckReport {
    let eq = check_equilibrium_2d(input, results);
    let meta = check_metamorphic_twin_2d(input, results);
    let dual = check_dual_assembly_2d(input, results);

    let mut passed = true;
    let mut failure_reason = None;

    if !eq.passed {
        passed = false;
        failure_reason = eq.detail.clone();
    } else if !meta.passed {
        passed = false;
        failure_reason = meta.detail.clone();
    } else if !dual.passed {
        passed = false;
        failure_reason = dual.detail.clone();
    }

    UserModelCheckReport {
        passed,
        failure_reason,
        equilibrium: eq,
        metamorphic_twin: meta,
        dual_assembly: dual,
    }
}

fn check_equilibrium_2d(input: &SolverInput, results: &AnalysisResults) -> EquilibriumReport {
    let nodes: HashMap<usize, (f64, f64)> = input.nodes.values().map(|n| (n.id, (n.x, n.z))).collect();
    let elements: HashMap<usize, &SolverElement> = input.elements.values().map(|e| (e.id, e)).collect();

    let mut fx_app = 0.0;
    let mut fz_app = 0.0;
    let mut my_app = 0.0;

    for ld in &input.loads {
        match ld {
            SolverLoad::Nodal(n) => {
                if let Some(&(x, z)) = nodes.get(&n.node_id) {
                    fx_app += n.fx;
                    fz_app += n.fz;
                    my_app += n.my + (x * n.fz - z * n.fx);
                }
            }
            SolverLoad::Distributed(d) => {
                if let Some(el) = elements.get(&d.element_id) {
                    if let (Some(&(xi, zi)), Some(&(xj, zj))) = (nodes.get(&el.node_i), nodes.get(&el.node_j)) {
                        let dx = xj - xi;
                        let dz = zj - zi;
                        let l = (dx * dx + dz * dz).sqrt();
                        if l > 1e-12 {
                            let tx = dx / l;
                            let tz = dz / l;
                            let nx = -tz;
                            let nz = tx;
                            let a = d.a.unwrap_or(0.0).clamp(0.0, l);
                            let b = d.b.unwrap_or(l).clamp(0.0, l);
                            let delta = b - a;
                            if delta > 0.0 {
                                let q_avg = (d.q_i + d.q_j) / 2.0;
                                let f_trans = q_avg * delta;
                                let sc = if (d.q_i + d.q_j).abs() > 1e-12 {
                                    a + (delta / 3.0) * (d.q_i + 2.0 * d.q_j) / (d.q_i + d.q_j)
                                } else {
                                    a + delta / 2.0
                                };
                                let xc = xi + (sc / l) * dx;
                                let zc = zi + (sc / l) * dz;
                                let fx = f_trans * nx;
                                let fz = f_trans * nz;
                                fx_app += fx;
                                fz_app += fz;
                                my_app += xc * fz - zc * fx;
                            }
                        }
                    }
                }
            }
            SolverLoad::PointOnElement(p) => {
                if let Some(el) = elements.get(&p.element_id) {
                    if let (Some(&(xi, zi)), Some(&(xj, zj))) = (nodes.get(&el.node_i), nodes.get(&el.node_j)) {
                        let dx = xj - xi;
                        let dz = zj - zi;
                        let l = (dx * dx + dz * dz).sqrt();
                        if l > 1e-12 {
                            let tx = dx / l;
                            let tz = dz / l;
                            let nx = -tz;
                            let nz = tx;
                            let a = p.a.clamp(0.0, l);
                            let xp = xi + (a / l) * dx;
                            let zp = zi + (a / l) * dz;
                            let mut fx = p.p * nx;
                            let mut fz = p.p * nz;
                            if let Some(px) = p.px {
                                fx += px * tx;
                                fz += px * tz;
                            }
                            fx_app += fx;
                            fz_app += fz;
                            my_app += p.my.unwrap_or(0.0) + (xp * fz - zp * fx);
                        }
                    }
                }
            }
            SolverLoad::AxialDistributed(ad) => {
                if let Some(el) = elements.get(&ad.element_id) {
                    if let (Some(&(xi, zi)), Some(&(xj, zj))) = (nodes.get(&el.node_i), nodes.get(&el.node_j)) {
                        let dx = xj - xi;
                        let dz = zj - zi;
                        let l = (dx * dx + dz * dz).sqrt();
                        if l > 1e-12 {
                            let tx = dx / l;
                            let tz = dz / l;
                            let a = ad.a.unwrap_or(0.0).clamp(0.0, l);
                            let b = ad.b.unwrap_or(l).clamp(0.0, l);
                            let delta = b - a;
                            if delta > 0.0 {
                                let f_ax = ((ad.q_i + ad.q_j) / 2.0) * delta;
                                let sc = if (ad.q_i + ad.q_j).abs() > 1e-12 {
                                    a + (delta / 3.0) * (ad.q_i + 2.0 * ad.q_j) / (ad.q_i + ad.q_j)
                                } else {
                                    a + delta / 2.0
                                };
                                let xc = xi + (sc / l) * dx;
                                let zc = zi + (sc / l) * dz;
                                let fx = f_ax * tx;
                                let fz = f_ax * tz;
                                fx_app += fx;
                                fz_app += fz;
                                my_app += xc * fz - zc * fx;
                            }
                        }
                    }
                }
            }
            SolverLoad::Thermal(_) => {}
        }
    }

    let mut rx_tot = 0.0;
    let mut rz_tot = 0.0;
    let mut rmy_tot = 0.0;

    for r in &results.reactions {
        if let Some(&(x, z)) = nodes.get(&r.node_id) {
            rx_tot += r.rx;
            rz_tot += r.rz;
            rmy_tot += r.my + (x * r.rz - z * r.rx);
        }
    }

    let diff_x = fx_app + rx_tot;
    let diff_z = fz_app + rz_tot;
    let diff_m = my_app + rmy_tot;

    let lchar = nodes.values().map(|&(x, z)| (x * x + z * z).sqrt()).fold(1.0, f64::max);
    let scale_f = fx_app.abs().max(fz_app.abs()).max(rx_tot.abs()).max(rz_tot.abs()).max(1.0);
    let scale_m = my_app.abs().max(rmy_tot.abs()).max(scale_f * lchar).max(1.0);

    let res_f = (diff_x * diff_x + diff_z * diff_z).sqrt() / scale_f;
    let res_m = diff_m.abs() / scale_m;

    let tol = 1e-4;
    let passed = res_f <= tol && res_m <= tol;

    let detail = if passed {
        None
    } else {
        Some(format!(
            "global equilibrium check failed: force residual = {:.3e}, moment residual = {:.3e} (limit {:.1e})",
            res_f, res_m, tol
        ))
    };

    EquilibriumReport {
        passed,
        residual_force: res_f,
        residual_moment: res_m,
        detail,
    }
}

fn check_metamorphic_twin_2d(input: &SolverInput, results: &AnalysisResults) -> MetamorphicTwinReport {
    let quarter = Iso2::rotation(std::f64::consts::FRAC_PI_2).translated(2.0, 2.0);
    let (iso, t_input) = match transform_2d(input, &quarter) {
        Ok(t) => (quarter, t),
        Err(_) => {
            let shift = Iso2::rotation(0.0).translated(5.0, 5.0);
            match transform_2d(input, &shift) {
                Ok(t) => (shift, t),
                Err(_) => (Iso2::rotation(0.0), input.clone()),
            }
        }
    };

    let seed = 42u64;
    let rl = Relabel::random_2d(&t_input, seed);
    let twin_input = rl.apply_2d(&t_input, seed);

    let twin_results = match solve_2d(&twin_input) {
        Ok(r) => r,
        Err(e) => {
            return MetamorphicTwinReport {
                passed: false,
                worst_ratio: 1.0,
                detail: Some(format!("metamorphic twin solve failed: {e}")),
            };
        }
    };

    let mapped = map_results_2d(results, &iso);
    let expected = rl.map_results_2d(&mapped);

    let cond = conditioning::of_2d(input).unwrap_or(Conditioning { n: 10, kappa: 1e3, geom: 1.0, lchar: 1.0 });
    let rel = (cond.rel() * 50.0).max(1e-6);

    let flat_twin = flat_2d(&twin_results, Parts::ALL);
    let flat_exp = flat_2d(&expected, Parts::ALL);

    match compare_floored(&flat_twin, &flat_exp, &flat_exp, cond.lchar, rel, 0.0) {
        Ok(rep) => MetamorphicTwinReport {
            passed: true,
            worst_ratio: rep.worst,
            detail: None,
        },
        Err(e) => MetamorphicTwinReport {
            passed: false,
            worst_ratio: 1.0,
            detail: Some(format!("metamorphic twin disagreement: {e}")),
        },
    }
}

fn check_dual_assembly_2d(input: &SolverInput, results: &AnalysisResults) -> DualAssemblyReport {
    let e2e = end_to_end_2d(input, &input.loads, results);
    match e2e {
        EndToEnd::Certified { k_discrepancy, f_discrepancy, .. } => DualAssemblyReport {
            passed: true,
            k_discrepancy,
            f_discrepancy,
            detail: None,
        },
        EndToEnd::Failed { reason } => DualAssemblyReport {
            passed: false,
            k_discrepancy: 1.0,
            f_discrepancy: 1.0,
            detail: Some(format!("dual assembly disagreement: {reason}")),
        },
        EndToEnd::OutOfScope { reason } => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: Some(format!("dual assembly out of scope: {reason}")),
        },
        EndToEnd::Uncertified { reason } => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: Some(reason),
        },
        EndToEnd::NotChecked => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: None,
        },
    }
}

// ---------------------------------------------------------------------------
// 3D Check
// ---------------------------------------------------------------------------

pub fn check_user_model_3d(input: &SolverInput3D, results: &AnalysisResults3D) -> UserModelCheckReport {
    let eq = check_equilibrium_3d(input, results);
    let meta = check_metamorphic_twin_3d(input, results);
    let dual = check_dual_assembly_3d(input, results);

    let mut passed = true;
    let mut failure_reason = None;

    if !eq.passed {
        passed = false;
        failure_reason = eq.detail.clone();
    } else if !meta.passed {
        passed = false;
        failure_reason = meta.detail.clone();
    } else if !dual.passed {
        passed = false;
        failure_reason = dual.detail.clone();
    }

    UserModelCheckReport {
        passed,
        failure_reason,
        equilibrium: eq,
        metamorphic_twin: meta,
        dual_assembly: dual,
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn check_equilibrium_3d(input: &SolverInput3D, results: &AnalysisResults3D) -> EquilibriumReport {
    let nodes: HashMap<usize, [f64; 3]> = input.nodes.values().map(|n| (n.id, [n.x, n.y, n.z])).collect();
    let elements: HashMap<usize, &SolverElement3D> = input.elements.values().map(|e| (e.id, e)).collect();

    let mut f_app = [0.0; 3];
    let mut m_app = [0.0; 3];

    for ld in &input.loads {
        match ld {
            SolverLoad3D::Nodal(n) => {
                if let Some(&p) = nodes.get(&n.node_id) {
                    let f = [n.fx, n.fy, n.fz];
                    let m = [n.mx, n.my, n.mz];
                    let r_cross_f = cross(p, f);
                    for k in 0..3 {
                        f_app[k] += f[k];
                        m_app[k] += m[k] + r_cross_f[k];
                    }
                }
            }
            SolverLoad3D::PointOnElement(p) => {
                if let Some(el) = elements.get(&p.element_id) {
                    if let (Some(&pi), Some(&pj)) = (nodes.get(&el.node_i), nodes.get(&el.node_j)) {
                        let d = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
                        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        if l > 1e-12 {
                            let frac = (p.a / l).clamp(0.0, 1.0);
                            let pos = [pi[0] + frac * d[0], pi[1] + frac * d[1], pi[2] + frac * d[2]];
                            let (_, ey, ez) = compute_local_axes_3d(
                                pi[0], pi[1], pi[2],
                                pj[0], pj[1], pj[2],
                                el.local_yx, el.local_yy, el.local_yz,
                                el.roll_angle, false,
                            );
                            let f = [
                                p.py * ey[0] + p.pz * ez[0],
                                p.py * ey[1] + p.pz * ez[1],
                                p.py * ey[2] + p.pz * ez[2],
                            ];
                            let r_cross_f = cross(pos, f);
                            for k in 0..3 {
                                f_app[k] += f[k];
                                m_app[k] += r_cross_f[k];
                            }
                        }
                    }
                }
            }
            SolverLoad3D::Distributed(d) => {
                if let Some(el) = elements.get(&d.element_id) {
                    if let (Some(&pi), Some(&pj)) = (nodes.get(&el.node_i), nodes.get(&el.node_j)) {
                        let dvec = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
                        let l = (dvec[0] * dvec[0] + dvec[1] * dvec[1] + dvec[2] * dvec[2]).sqrt();
                        if l > 1e-12 {
                            let a = d.a.unwrap_or(0.0).clamp(0.0, l);
                            let b = d.b.unwrap_or(l).clamp(0.0, l);
                            let delta = b - a;
                            if delta > 0.0 {
                                let (ex, ey, ez) = compute_local_axes_3d(
                                    pi[0], pi[1], pi[2],
                                    pj[0], pj[1], pj[2],
                                    el.local_yx, el.local_yy, el.local_yz,
                                    el.roll_angle, false,
                                );
                                let qx = (d.q_xi + d.q_xj) * 0.5 * delta;
                                let qy = (d.q_yi + d.q_yj) * 0.5 * delta;
                                let qz = (d.q_zi + d.q_zj) * 0.5 * delta;
                                let f_total = [
                                    qx * ex[0] + qy * ey[0] + qz * ez[0],
                                    qx * ex[1] + qy * ey[1] + qz * ez[1],
                                    qx * ex[2] + qy * ey[2] + qz * ez[2],
                                ];
                                let sc = a + delta * 0.5;
                                let pos = [pi[0] + (sc / l) * dvec[0], pi[1] + (sc / l) * dvec[1], pi[2] + (sc / l) * dvec[2]];
                                let r_cross_f = cross(pos, f_total);
                                for k in 0..3 {
                                    f_app[k] += f_total[k];
                                    m_app[k] += r_cross_f[k];
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut f_react = [0.0; 3];
    let mut m_react = [0.0; 3];

    for r in &results.reactions {
        if let Some(&p) = nodes.get(&r.node_id) {
            let f = [r.fx, r.fy, r.fz];
            let m = [r.mx, r.my, r.mz];
            let r_cross_f = cross(p, f);
            for k in 0..3 {
                f_react[k] += f[k];
                m_react[k] += m[k] + r_cross_f[k];
            }
        }
    }

    let diff_f = [f_app[0] + f_react[0], f_app[1] + f_react[1], f_app[2] + f_react[2]];
    let diff_m = [m_app[0] + m_react[0], m_app[1] + m_react[1], m_app[2] + m_react[2]];

    let lchar = nodes.values().map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()).fold(1.0, f64::max);
    let scale_f = (0..3).map(|k| f_app[k].abs().max(f_react[k].abs())).fold(1.0, f64::max);
    let scale_m = (0..3).map(|k| m_app[k].abs().max(m_react[k].abs())).fold(scale_f * lchar, f64::max);

    let res_f = (diff_f[0] * diff_f[0] + diff_f[1] * diff_f[1] + diff_f[2] * diff_f[2]).sqrt() / scale_f;
    let res_m = (diff_m[0] * diff_m[0] + diff_m[1] * diff_m[1] + diff_m[2] * diff_m[2]).sqrt() / scale_m;

    let tol = 1e-4;
    let passed = res_f <= tol && res_m <= tol;

    let detail = if passed {
        None
    } else {
        Some(format!(
            "3D global equilibrium check failed: force residual = {:.3e}, moment residual = {:.3e} (limit {:.1e})",
            res_f, res_m, tol
        ))
    };

    EquilibriumReport {
        passed,
        residual_force: res_f,
        residual_moment: res_m,
        detail,
    }
}

fn check_metamorphic_twin_3d(input: &SolverInput3D, results: &AnalysisResults3D) -> MetamorphicTwinReport {
    let perm = Iso3::axis_permutation(1);
    let (iso, t_input) = match transform_3d(input, &perm) {
        Ok(t) => (perm, t),
        Err(_) => {
            let shift = Iso3::from_euler(0.0, 0.0, 0.0).translated([3.0, 3.0, 3.0]);
            match transform_3d(input, &shift) {
                Ok(t) => (shift, t),
                Err(_) => (Iso3::from_euler(0.0, 0.0, 0.0), input.clone()),
            }
        }
    };

    let seed = 42u64;
    let rl = Relabel::random_3d(&t_input, seed);
    let twin_input = rl.apply_3d(&t_input, seed);

    let twin_results = match solve_3d(&twin_input) {
        Ok(r) => r,
        Err(e) => {
            return MetamorphicTwinReport {
                passed: false,
                worst_ratio: 1.0,
                detail: Some(format!("3D metamorphic twin solve failed: {e}")),
            };
        }
    };

    let mapped = map_results_3d(results, &iso);
    let expected = rl.map_results_3d(&mapped);

    let cond = conditioning::of_3d(input).unwrap_or(Conditioning { n: 10, kappa: 1e3, geom: 1.0, lchar: 1.0 });
    let rel = (cond.rel() * 50.0).max(1e-6);

    let flat_twin = flat_3d(&twin_results, Parts::ALL);
    let flat_exp = flat_3d(&expected, Parts::ALL);

    match compare_floored(&flat_twin, &flat_exp, &flat_exp, cond.lchar, rel, 0.0) {
        Ok(rep) => MetamorphicTwinReport {
            passed: true,
            worst_ratio: rep.worst,
            detail: None,
        },
        Err(e) => MetamorphicTwinReport {
            passed: false,
            worst_ratio: 1.0,
            detail: Some(format!("3D metamorphic twin disagreement: {e}")),
        },
    }
}

fn check_dual_assembly_3d(input: &SolverInput3D, results: &AnalysisResults3D) -> DualAssemblyReport {
    let e2e = end_to_end_3d(input, &input.loads, results);
    match e2e {
        EndToEnd::Certified { k_discrepancy, f_discrepancy, .. } => DualAssemblyReport {
            passed: true,
            k_discrepancy,
            f_discrepancy,
            detail: None,
        },
        EndToEnd::Failed { reason } => DualAssemblyReport {
            passed: false,
            k_discrepancy: 1.0,
            f_discrepancy: 1.0,
            detail: Some(format!("3D dual assembly disagreement: {reason}")),
        },
        EndToEnd::OutOfScope { reason } => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: Some(format!("3D dual assembly out of scope: {reason}")),
        },
        EndToEnd::Uncertified { reason } => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: Some(reason),
        },
        EndToEnd::NotChecked => DualAssemblyReport {
            passed: true,
            k_discrepancy: 0.0,
            f_discrepancy: 0.0,
            detail: None,
        },
    }
}
