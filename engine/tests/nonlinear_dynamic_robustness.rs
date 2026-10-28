//! Tests for Issue #381: Nonlinear and dynamic robustness
//! - Convergence criteria audited (force, displacement, energy norms)
//! - Load-step and time-step sensitivity tests
//! - Energy-drift monitoring in time history, reported to the user
//! - Path following past limit points

use dedaliano_engine::solver::arc_length::{solve_displacement_control, DisplacementControlInput};
use dedaliano_engine::solver::corotational::solve_corotational_2d;
use dedaliano_engine::solver::time_integration::solve_time_history_2d;
use dedaliano_engine::types::*;
use std::collections::HashMap;

fn make_dynamic_cantilever() -> TimeHistoryInput {
    let nodes: HashMap<String, SolverNode> = [
        ("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 }),
        ("2".into(), SolverNode { id: 2, x: 3.0, z: 0.0 }),
    ].into_iter().collect();

    let materials: HashMap<String, SolverMaterial> = [(
        "1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 },
    )].into_iter().collect();

    let sections: HashMap<String, SolverSection> = [(
        "1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None },
    )].into_iter().collect();

    let elements: HashMap<String, SolverElement> = [(
        "1".into(), SolverElement {
            id: 1, elem_type: "frame".into(),
            node_i: 1, node_j: 2, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        },
    )].into_iter().collect();

    let supports: HashMap<String, SolverSupport> = [(
        "1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        },
    )].into_iter().collect();

    let solver = SolverInput {
        solver_options: None, masses: Default::default(),
        nodes, materials, sections, elements, supports,
        loads: vec![],
        constraints: vec![],
        connectors: HashMap::new(),
    };

    let mut densities = HashMap::new();
    densities.insert("1".to_string(), 7850.0); // Steel density kg/m^3

    TimeHistoryInput {
        solver,
        densities,
        time_step: 0.001,
        n_steps: 100,
        method: "newmark".to_string(),
        beta: 0.25,
        gamma: 0.5,
        alpha: None,
        damping_xi: None,
        ground_accel: None,
        ground_direction: None,
        force_history: None,
    }
}

/// Acceptance criterion 3: Energy-drift monitoring in time history, reported to the user.
/// Checks that TimeHistoryResult contains an energy_balance field monitoring drift.
#[test]
fn test_energy_drift_monitoring_reported_to_user() {
    let mut input = make_dynamic_cantilever();
    input.force_history = Some(vec![
        TimeForceRecord {
            time: 0.0,
            loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: -10.0, my: 0.0 }],
        },
        TimeForceRecord {
            time: 0.001,
            loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: 0.0, my: 0.0 }],
        },
    ]);
    input.damping_xi = None; // Undamped Newmark average acceleration
    input.time_step = 0.001;
    input.n_steps = 200;

    let result = solve_time_history_2d(&input).expect("Time history should solve");
    
    let energy = result.energy_balance.expect("Time history must monitor and report energy balance to user");
    assert!(
        energy.relative_drift < 0.01,
        "Undamped Newmark average acceleration should conserve energy (drift < 1%), got: {:.4e}",
        energy.relative_drift
    );
    assert!(energy.initial_energy >= 0.0);
    assert!(energy.final_energy >= 0.0);
}

/// Dynamic robustness: checks that algorithmic damping or artificial dissipation
/// produces detectable energy drift and emits a warning diagnostic.
#[test]
fn test_energy_drift_warning_on_large_time_step() {
    let mut input = make_dynamic_cantilever();
    // Large impulse
    input.force_history = Some(vec![
        TimeForceRecord {
            time: 0.0,
            loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: -100.0, my: 0.0 }],
        },
        TimeForceRecord {
            time: 0.01,
            loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: 0.0, my: 0.0 }],
        },
    ]);
    input.damping_xi = None;
    // Newmark with excessive algorithmic dissipation (gamma = 0.8, beta = 0.45)
    input.gamma = 0.8;
    input.beta = 0.45;
    input.time_step = 0.01;
    input.n_steps = 100;

    let result = solve_time_history_2d(&input).expect("Time history should solve");
    let energy = result.energy_balance.expect("Energy balance must be present");
    assert!(
        energy.relative_drift > 0.01,
        "Expected measurable energy drift with dissipative scheme, got {:.4e}",
        energy.relative_drift
    );
    if energy.relative_drift > 0.05 {
        assert!(
            result.diagnostics.iter().any(|d| d.category == "energy_drift"),
            "Energy drift > 5% must emit a warning diagnostic to the user"
        );
    }
}

/// Acceptance criterion 2: Time-step sensitivity and convergence rate.
/// Solves dynamic cantilever with successive dt refinements and tests second-order convergence.
#[test]
fn test_time_step_sensitivity_newmark_convergence() {
    let run = |dt: f64| -> f64 {
        let mut input = make_dynamic_cantilever();
        let total_time = 0.1;
        let n_steps = (total_time / dt).round() as usize;
        input.force_history = Some(vec![
            TimeForceRecord {
                time: 0.0,
                loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: -10.0, my: 0.0 }],
            },
            TimeForceRecord {
                time: 0.01,
                loads: vec![SolverNodalLoad { node_id: 2, fx: 0.0, fz: 0.0, my: 0.0 }],
            },
        ]);
        input.time_step = dt;
        input.n_steps = n_steps;
        let res = solve_time_history_2d(&input).expect("solve");
        let n2 = res.node_histories.iter().find(|h| h.node_id == 2).expect("node 2");
        *n2.uz.last().expect("last uz")
    };

    let dt1 = 0.004;
    let dt2 = 0.002;
    let dt3 = 0.001;
    let dt4 = 0.0005;

    let u1 = run(dt1);
    let u2 = run(dt2);
    let u3 = run(dt3);
    let u4 = run(dt4);

    let err1 = (u1 - u4).abs();
    let err2 = (u2 - u4).abs();
    let err3 = (u3 - u4).abs();

    // Monotonic convergence with time-step refinement
    assert!(err2 < err1, "Error should decrease with smaller time step: err2={err2:.2e}, err1={err1:.2e}");
    assert!(err3 < err2, "Error should decrease with smaller time step: err3={err3:.2e}, err2={err2:.2e}");

    // Rate of convergence p = ln(err1 / err2) / ln(2) ~ 2 for Newmark average acceleration
    let rate = (err1 / err2).log2();
    assert!(rate > 1.2, "Newmark average acceleration should exhibit second-order convergence rate, got: {rate:.2}");
}

/// Acceptance criterion 2: Load-step sensitivity in geometric nonlinear analysis.
/// Checks that increasing load steps converges monotonically to the asymptotic solution.
#[test]
fn test_load_step_sensitivity_corotational_cantilever() {
    let nodes: HashMap<String, SolverNode> = [
        ("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 }),
        ("2".into(), SolverNode { id: 2, x: 2.0, z: 0.0 }),
        ("3".into(), SolverNode { id: 3, x: 4.0, z: 0.0 }),
    ].into_iter().collect();

    let materials: HashMap<String, SolverMaterial> = [(
        "1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 },
    )].into_iter().collect();

    let sections: HashMap<String, SolverSection> = [(
        "1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None },
    )].into_iter().collect();

    let elements: HashMap<String, SolverElement> = [
        ("1".into(), SolverElement {
            id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        }),
        ("2".into(), SolverElement {
            id: 2, elem_type: "frame".into(), node_i: 2, node_j: 3, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        }),
    ].into_iter().collect();

    let supports: HashMap<String, SolverSupport> = [(
        "1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        },
    )].into_iter().collect();

    let loads = vec![SolverLoad::Nodal(SolverNodalLoad {
        node_id: 3, fx: 0.0, fz: -80.0, my: 0.0,
    })];

    let solver = SolverInput {
        solver_options: None, masses: Default::default(),
        nodes, materials, sections, elements, supports, loads,
        constraints: vec![], connectors: HashMap::new(),
    };

    let run = |steps: usize| -> f64 {
        let res = solve_corotational_2d(&solver, 60, 1e-8, steps, false)
            .expect("Corotational solve should succeed");
        let n3 = res.results.displacements.iter().find(|d| d.node_id == 3).expect("node 3");
        n3.uz
    };

    let u_5 = run(5);
    let u_10 = run(10);
    let u_20 = run(20);
    let u_40 = run(40);

    // Difference between fine steps (N=20 vs N=40) should be negligible (< 0.001%)
    let diff_rel = (u_20 - u_40).abs() / u_40.abs();
    assert!(
        diff_rel < 1e-5,
        "Load-step sensitivity between 20 and 40 increments should be < 1e-5, got: {diff_rel:.2e}"
    );

    let err_5 = (u_5 - u_40).abs();
    let err_10 = (u_10 - u_40).abs();
    assert!(err_5 < 1e-6, "N=5 should match N=40 to high precision: err_5={err_5:.2e}");
    assert!(err_10 < 1e-6, "N=10 should match N=40 to high precision: err_10={err_10:.2e}");
}

/// Acceptance criterion 4: Path following past limit points.
/// A shallow arch / toggle frame traces past the limit point where load factor decreases (snap-through).
#[test]
fn test_path_following_past_limit_point_snap_through() {
    let rise = 0.25;
    let half_span = 4.0;
    let nodes: HashMap<String, SolverNode> = [
        ("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 }),
        ("2".into(), SolverNode { id: 2, x: half_span, z: rise }),
        ("3".into(), SolverNode { id: 3, x: 2.0 * half_span, z: 0.0 }),
    ].into_iter().collect();

    let materials: HashMap<String, SolverMaterial> = [(
        "1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 },
    )].into_iter().collect();

    let sections: HashMap<String, SolverSection> = [(
        "1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None },
    )].into_iter().collect();

    let elements: HashMap<String, SolverElement> = [
        ("1".into(), SolverElement {
            id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        }),
        ("2".into(), SolverElement {
            id: 2, elem_type: "frame".into(), node_i: 2, node_j: 3, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        }),
    ].into_iter().collect();

    let supports: HashMap<String, SolverSupport> = [
        ("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "pinned".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        }),
        ("2".into(), SolverSupport {
            id: 2, node_id: 3, support_type: "pinned".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        }),
    ].into_iter().collect();

    let loads = vec![SolverLoad::Nodal(SolverNodalLoad {
        node_id: 2, fx: 0.0, fz: -1.0, my: 0.0,
    })];

    let solver = SolverInput {
        solver_options: None, masses: Default::default(),
        nodes, materials, sections, elements, supports, loads,
        constraints: vec![], connectors: HashMap::new(),
    };

    let dc_input = DisplacementControlInput {
        solver,
        control_node: 2,
        control_dof: 1, // uz
        target_displacement: -0.35,
        n_steps: 30,
        max_iter: 30,
        tolerance: 1e-6,
    };

    let result = solve_displacement_control(&dc_input).expect("Displacement control should solve toggle arch");
    assert!(result.converged, "Path following past limit point should converge");
    assert!(result.steps.len() >= 10, "Should trace equilibrium path");

    // Verify limit point traversal: load factors should rise to peak and then decrease past snap-through
    let load_factors: Vec<f64> = result.steps.iter().filter(|s| s.converged).map(|s| s.load_factor).collect();
    assert!(load_factors.len() >= 10, "Need converged steps across limit point");

    // Find local maximum limit point (peak of the snap-through curve):
    let mut limit_point_found = false;
    let mut peak_lambda = 0.0;
    for i in 1..load_factors.len() - 1 {
        if load_factors[i] > load_factors[i - 1] && load_factors[i] > load_factors[i + 1] {
            limit_point_found = true;
            peak_lambda = load_factors[i];
            break;
        }
    }

    assert!(limit_point_found, "Must trace past a limit point (local peak) into softening");
    assert!(peak_lambda > 10.0, "Peak load should be non-trivial: {peak_lambda:.2}");

    // Verify negative load increments occur past the limit point:
    let softening_steps = load_factors.windows(2).filter(|w| w[1] < w[0]).count();
    assert!(softening_steps >= 3, "Expected multiple softening steps with dLambda < 0, got {softening_steps}");
}
