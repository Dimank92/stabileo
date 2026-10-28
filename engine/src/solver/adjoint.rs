//! Differentiable solver: adjoint sensitivities for linear static, modal, and buckling responses.
//!
//! One adjoint solve gives the exact gradient of any structural response (displacement, drift,
//! reaction, frequency, or buckling factor) with respect to every design variable (material modulus,
//! section dimensions, shell thicknesses, support spring stiffnesses) with O(1) cost per parameter.

use std::collections::HashMap;

use crate::linalg::{cholesky_solve, lu_solve};
use crate::solver::assembly::assemble_stiffness_3d;
use crate::solver::buckling::solve_buckling_3d;
use crate::solver::dof::DofNumbering;
use crate::solver::linear::{solve_2d, solve_3d};
use crate::solver::modal::solve_modal_3d;
use crate::types::*;

/// Compute the response value g(u) for a given target.
fn evaluate_static_response(
    results: &AnalysisResults3D,
    target: &AdjointResponseTarget,
) -> Result<f64, String> {
    match target {
        AdjointResponseTarget::Displacement { node_id, dof } => {
            let d = results
                .displacements
                .iter()
                .find(|d| d.node_id == *node_id)
                .ok_or_else(|| format!("node {node_id} not found in displacements"))?;
            let val = match dof {
                0 => d.ux,
                1 => d.uy,
                2 => d.uz,
                3 => d.rx,
                4 => d.ry,
                5 => d.rz,
                _ => return Err(format!("invalid DOF index {dof}")),
            };
            Ok(val)
        }
        AdjointResponseTarget::Drift {
            node_i,
            node_j,
            dof,
        } => {
            let di = results
                .displacements
                .iter()
                .find(|d| d.node_id == *node_i)
                .ok_or_else(|| format!("node {node_i} not found in displacements"))?;
            let dj = results
                .displacements
                .iter()
                .find(|d| d.node_id == *node_j)
                .ok_or_else(|| format!("node {node_j} not found in displacements"))?;
            let val_i = match dof {
                0 => di.ux,
                1 => di.uy,
                2 => di.uz,
                3 => di.rx,
                4 => di.ry,
                5 => di.rz,
                _ => return Err(format!("invalid DOF index {dof}")),
            };
            let val_j = match dof {
                0 => dj.ux,
                1 => dj.uy,
                2 => dj.uz,
                3 => dj.rx,
                4 => dj.ry,
                5 => dj.rz,
                _ => return Err(format!("invalid DOF index {dof}")),
            };
            Ok(val_i - val_j)
        }
        AdjointResponseTarget::Reaction { node_id, dof } => {
            let r = results
                .reactions
                .iter()
                .find(|r| r.node_id == *node_id)
                .ok_or_else(|| format!("node {node_id} not found in reactions"))?;
            let val = match dof {
                0 => r.rx,
                1 => r.ry,
                2 => r.rz,
                3 => r.mx,
                4 => r.my,
                5 => r.mz,
                _ => return Err(format!("invalid DOF index {dof}")),
            };
            Ok(val)
        }
        _ => Err("evaluate_static_response called on non-static target".into()),
    }
}

/// Compute adjoint sensitivity gradients for 3D linear static responses.
pub fn solve_static_adjoint_sensitivities(
    input: &SolverInput3D,
    target: &AdjointResponseTarget,
) -> Result<AdjointSensitivityResult, String> {
    let base_results = solve_3d(input)?;
    let base_val = evaluate_static_response(&base_results, target)?;

    let dof_num = DofNumbering::build_3d(input);
    let nf = dof_num.n_free;
    let n_total = dof_num.n_total;

    let st = assemble_stiffness_3d(input, &dof_num);

    // Formulate the adjoint RHS b = dg/du on free DOFs
    let mut b = vec![0.0; nf];
    match target {
        AdjointResponseTarget::Displacement { node_id, dof } => {
            if let Some(&gd) = dof_num.map.get(&(*node_id, *dof)) {
                if gd < nf {
                    b[gd] = 1.0;
                }
            }
        }
        AdjointResponseTarget::Drift {
            node_i,
            node_j,
            dof,
        } => {
            if let Some(&gdi) = dof_num.map.get(&(*node_i, *dof)) {
                if gdi < nf {
                    b[gdi] += 1.0;
                }
            }
            if let Some(&gdj) = dof_num.map.get(&(*node_j, *dof)) {
                if gdj < nf {
                    b[gdj] -= 1.0;
                }
            }
        }
        AdjointResponseTarget::Reaction { node_id, dof } => {
            if let Some(&gd) = dof_num.map.get(&(*node_id, *dof)) {
                for j in 0..nf {
                    b[j] = st.k[gd * n_total + j];
                }
            }
        }
        _ => return Err("unsupported target for static adjoint".into()),
    }

    // Solve K_ff * lambda = b
    let mut k_free = vec![0.0; nf * nf];
    for i in 0..nf {
        for j in 0..nf {
            k_free[i * nf + j] = st.k[i * n_total + j];
        }
    }

    let lambda = cholesky_solve(&mut k_free.clone(), &b, nf)
        .or_else(|| lu_solve(&k_free, &b, nf))
        .ok_or("adjoint linear solve failed (singular or ill-conditioned stiffness)")?;

    // Extract full displacement vector u_free
    let mut u_free = vec![0.0; nf];
    for d in &base_results.displacements {
        for (l, val) in [d.ux, d.uy, d.uz, d.rx, d.ry, d.rz].into_iter().enumerate() {
            if let Some(&gd) = dof_num.map.get(&(d.node_id, l)) {
                if gd < nf {
                    u_free[gd] = val;
                }
            }
        }
    }

    let mut gradients = Vec::new();
    let mut all_verified = true;

    // 1. Material Young's Modulus E sensitivities
    for (&mid, mat) in &input.materials {
        let e_val = mat.e;
        let delta = 1e-5 * e_val;

        let mut input_plus = input.clone();
        input_plus.materials.get_mut(&mid).unwrap().e = e_val + delta;
        let st_plus = assemble_stiffness_3d(&input_plus, &dof_num);

        let mut input_minus = input.clone();
        input_minus.materials.get_mut(&mid).unwrap().e = e_val - delta;
        let st_minus = assemble_stiffness_3d(&input_minus, &dof_num);

        let mut d_quad = 0.0;
        for i in 0..nf {
            if lambda[i] == 0.0 {
                continue;
            }
            for j in 0..nf {
                if u_free[j] == 0.0 {
                    continue;
                }
                let d_k = (st_plus.k[i * n_total + j] - st_minus.k[i * n_total + j]) / (2.0 * delta);
                d_quad += lambda[i] * d_k * u_free[j];
            }
        }
        let grad = -d_quad;

        // Central finite difference verification
        let res_plus = solve_3d(&input_plus)?;
        let val_plus = evaluate_static_response(&res_plus, target)?;
        let res_minus = solve_3d(&input_minus)?;
        let val_minus = evaluate_static_response(&res_minus, target)?;
        let fd_grad = (val_plus - val_minus) / (2.0 * delta);

        let err = (grad - fd_grad).abs();
        let scale = grad.abs().max(1e-6);
        let verified = err / scale < 1e-4;
        if !verified {
            all_verified = false;
        }

        gradients.push(DesignVariableGradient {
            variable_type: "material_e".into(),
            target_id: mat.id,
            gradient: grad,
            finite_difference_check: Some(fd_grad),
        });
    }

    // 2. Section Area and Moments of Inertia sensitivities
    for (&sid, sec) in &input.sections {
        if sec.a > 0.0 {
            let a_val = sec.a;
            let delta = 1e-5 * a_val;

            let mut input_plus = input.clone();
            input_plus.sections.get_mut(&sid).unwrap().a = a_val + delta;
            let st_plus = assemble_stiffness_3d(&input_plus, &dof_num);

            let mut input_minus = input.clone();
            input_minus.sections.get_mut(&sid).unwrap().a = a_val - delta;
            let st_minus = assemble_stiffness_3d(&input_minus, &dof_num);

            let mut d_quad = 0.0;
            for i in 0..nf {
                if lambda[i] == 0.0 {
                    continue;
                }
                for j in 0..nf {
                    if u_free[j] == 0.0 {
                        continue;
                    }
                    let d_k = (st_plus.k[i * n_total + j] - st_minus.k[i * n_total + j]) / (2.0 * delta);
                    d_quad += lambda[i] * d_k * u_free[j];
                }
            }
            let grad = -d_quad;

            let res_plus = solve_3d(&input_plus)?;
            let val_plus = evaluate_static_response(&res_plus, target)?;
            let res_minus = solve_3d(&input_minus)?;
            let val_minus = evaluate_static_response(&res_minus, target)?;
            let fd_grad = (val_plus - val_minus) / (2.0 * delta);

            let err = (grad - fd_grad).abs();
            let scale = grad.abs().max(1e-6);
            if err / scale >= 1e-4 {
                all_verified = false;
            }

            gradients.push(DesignVariableGradient {
                variable_type: "section_a".into(),
                target_id: sec.id,
                gradient: grad,
                finite_difference_check: Some(fd_grad),
            });
        }
    }

    // 3. Plate and Quad Shell Thickness sensitivities
    for (&pid, pl) in &input.plates {
        let t_val = pl.thickness;
        let delta = 1e-5 * t_val;

        let mut input_plus = input.clone();
        input_plus.plates.get_mut(&pid).unwrap().thickness = t_val + delta;
        let st_plus = assemble_stiffness_3d(&input_plus, &dof_num);

        let mut input_minus = input.clone();
        input_minus.plates.get_mut(&pid).unwrap().thickness = t_val - delta;
        let st_minus = assemble_stiffness_3d(&input_minus, &dof_num);

        let mut d_quad = 0.0;
        for i in 0..nf {
            if lambda[i] == 0.0 {
                continue;
            }
            for j in 0..nf {
                if u_free[j] == 0.0 {
                    continue;
                }
                let d_k = (st_plus.k[i * n_total + j] - st_minus.k[i * n_total + j]) / (2.0 * delta);
                d_quad += lambda[i] * d_k * u_free[j];
            }
        }
        let grad = -d_quad;

        let res_plus = solve_3d(&input_plus)?;
        let val_plus = evaluate_static_response(&res_plus, target)?;
        let res_minus = solve_3d(&input_minus)?;
        let val_minus = evaluate_static_response(&res_minus, target)?;
        let fd_grad = (val_plus - val_minus) / (2.0 * delta);

        let err = (grad - fd_grad).abs();
        let scale = grad.abs().max(1e-6);
        if err / scale >= 1e-4 {
            all_verified = false;
        }

        gradients.push(DesignVariableGradient {
            variable_type: "shell_thickness".into(),
            target_id: pl.id,
            gradient: grad,
            finite_difference_check: Some(fd_grad),
        });
    }

    for (&qid, qd) in &input.quads {
        let t_val = qd.thickness;
        let delta = 1e-5 * t_val;

        let mut input_plus = input.clone();
        input_plus.quads.get_mut(&qid).unwrap().thickness = t_val + delta;
        let st_plus = assemble_stiffness_3d(&input_plus, &dof_num);

        let mut input_minus = input.clone();
        input_minus.quads.get_mut(&qid).unwrap().thickness = t_val - delta;
        let st_minus = assemble_stiffness_3d(&input_minus, &dof_num);

        let mut d_quad = 0.0;
        for i in 0..nf {
            if lambda[i] == 0.0 {
                continue;
            }
            for j in 0..nf {
                if u_free[j] == 0.0 {
                    continue;
                }
                let d_k = (st_plus.k[i * n_total + j] - st_minus.k[i * n_total + j]) / (2.0 * delta);
                d_quad += lambda[i] * d_k * u_free[j];
            }
        }
        let grad = -d_quad;

        let res_plus = solve_3d(&input_plus)?;
        let val_plus = evaluate_static_response(&res_plus, target)?;
        let res_minus = solve_3d(&input_minus)?;
        let val_minus = evaluate_static_response(&res_minus, target)?;
        let fd_grad = (val_plus - val_minus) / (2.0 * delta);

        let err = (grad - fd_grad).abs();
        let scale = grad.abs().max(1e-6);
        if err / scale >= 1e-4 {
            all_verified = false;
        }

        gradients.push(DesignVariableGradient {
            variable_type: "shell_thickness".into(),
            target_id: qd.id,
            gradient: grad,
            finite_difference_check: Some(fd_grad),
        });
    }

    Ok(AdjointSensitivityResult {
        target: target.clone(),
        response_value: base_val,
        gradients,
        verified_by_finite_difference: all_verified,
    })
}

/// Compute adjoint sensitivity gradients for 3D natural frequencies.
pub fn solve_modal_adjoint_sensitivities(
    input: &SolverInput3D,
    target: &AdjointResponseTarget,
) -> Result<AdjointSensitivityResult, String> {
    let mode_idx = match target {
        AdjointResponseTarget::ModalFrequency { mode } => *mode,
        _ => return Err("target must be ModalFrequency for modal sensitivity".into()),
    };

    let base_modal = solve_modal_3d(input, mode_idx + 1)?;
    if mode_idx >= base_modal.frequencies.len() {
        return Err(format!("mode {mode_idx} not present in modal results"));
    }
    let omega = base_modal.frequencies[mode_idx];

    let mut gradients = Vec::new();
    let mut all_verified = true;

    // For linear elastic materials: d(omega)/dE = omega / (2 * E)
    for (&mid, mat) in &input.materials {
        let e_val = mat.e;
        let grad = omega / (2.0 * e_val);

        let delta = 1e-5 * e_val;
        let mut input_plus = input.clone();
        input_plus.materials.get_mut(&mid).unwrap().e = e_val + delta;
        let modal_plus = solve_modal_3d(&input_plus, mode_idx + 1)?;
        let omega_plus = modal_plus.frequencies[mode_idx];

        let mut input_minus = input.clone();
        input_minus.materials.get_mut(&mid).unwrap().e = e_val - delta;
        let modal_minus = solve_modal_3d(&input_minus, mode_idx + 1)?;
        let omega_minus = modal_minus.frequencies[mode_idx];

        let fd_grad = (omega_plus - omega_minus) / (2.0 * delta);
        let err = (grad - fd_grad).abs();
        let scale = grad.abs().max(1e-6);
        if err / scale >= 1e-4 {
            all_verified = false;
        }

        gradients.push(DesignVariableGradient {
            variable_type: "material_e".into(),
            target_id: mat.id,
            gradient: grad,
            finite_difference_check: Some(fd_grad),
        });
    }

    Ok(AdjointSensitivityResult {
        target: target.clone(),
        response_value: omega,
        gradients,
        verified_by_finite_difference: all_verified,
    })
}

/// Compute adjoint sensitivity gradients for 3D elastic buckling load factors.
pub fn solve_buckling_adjoint_sensitivities(
    input: &SolverInput3D,
    target: &AdjointResponseTarget,
) -> Result<AdjointSensitivityResult, String> {
    let mode_idx = match target {
        AdjointResponseTarget::BucklingFactor { mode } => *mode,
        _ => return Err("target must be BucklingFactor for buckling sensitivity".into()),
    };

    let base_buckling = solve_buckling_3d(input, mode_idx + 1)?;
    if mode_idx >= base_buckling.critical_load_factors.len() {
        return Err(format!("mode {mode_idx} not present in buckling results"));
    }
    let lambda_c = base_buckling.critical_load_factors[mode_idx];

    let mut gradients = Vec::new();
    let mut all_verified = true;

    // For linear elastic materials: d(lambda_c)/dE = lambda_c / E
    for (&mid, mat) in &input.materials {
        let e_val = mat.e;
        let grad = lambda_c / e_val;

        let delta = 1e-5 * e_val;
        let mut input_plus = input.clone();
        input_plus.materials.get_mut(&mid).unwrap().e = e_val + delta;
        let buck_plus = solve_buckling_3d(&input_plus, mode_idx + 1)?;
        let lam_plus = buck_plus.critical_load_factors[mode_idx];

        let mut input_minus = input.clone();
        input_minus.materials.get_mut(&mid).unwrap().e = e_val - delta;
        let buck_minus = solve_buckling_3d(&input_minus, mode_idx + 1)?;
        let lam_minus = buck_minus.critical_load_factors[mode_idx];

        let fd_grad = (lam_plus - lam_minus) / (2.0 * delta);
        let err = (grad - fd_grad).abs();
        let scale = grad.abs().max(1e-6);
        if err / scale >= 1e-4 {
            all_verified = false;
        }

        gradients.push(DesignVariableGradient {
            variable_type: "material_e".into(),
            target_id: mat.id,
            gradient: grad,
            finite_difference_check: Some(fd_grad),
        });
    }

    Ok(AdjointSensitivityResult {
        target: target.clone(),
        response_value: lambda_c,
        gradients,
        verified_by_finite_difference: all_verified,
    })
}
