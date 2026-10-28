//! Verification of adjoint sensitivity analysis against central finite difference (FD).

use std::collections::HashMap;

use dedaliano_engine::solver::adjoint::{
    solve_buckling_adjoint_sensitivities, solve_modal_adjoint_sensitivities,
    solve_static_adjoint_sensitivities,
};
use dedaliano_engine::types::*;

fn simple_3d_frame() -> SolverInput3D {
    let json = r#"{
      "nodes": {
        "1": {"id":1,"x":0,"y":0,"z":0},
        "2": {"id":2,"x":0,"y":0,"z":3},
        "3": {"id":3,"x":4,"y":0,"z":3}
      },
      "materials": {
        "1": {"id":1,"e":210000,"nu":0.3}
      },
      "sections": {
        "1": {"id":1,"a":0.01,"iy":0.0002,"iz":0.0001,"j":0.00005}
      },
      "elements": {
        "1": {"id":1,"type":"frame","nodeI":1,"nodeJ":2,"materialId":1,"sectionId":1},
        "2": {"id":2,"type":"frame","nodeI":2,"nodeJ":3,"materialId":1,"sectionId":1}
      },
      "supports": {
        "1": {"nodeId":1,"rx":true,"ry":true,"rz":true,"rrx":true,"rry":true,"rrz":true}
      },
      "loads": [
        {"type":"nodal","data":{"nodeId":3,"fx":10,"fy":5,"fz":-20,"mx":0,"my":0,"mz":0}}
      ]
    }"#;
    serde_json::from_str(json).unwrap()
}

fn simple_plate_model() -> SolverInput3D {
    let json = r#"{
      "nodes": {
        "1": {"id":1,"x":0,"y":0,"z":0},
        "2": {"id":2,"x":2,"y":0,"z":0},
        "3": {"id":3,"x":2,"y":2,"z":0},
        "4": {"id":4,"x":0,"y":2,"z":0}
      },
      "materials": {
        "1": {"id":1,"e":200000,"nu":0.3}
      },
      "plates": {
        "1": {"id":1,"nodes":[1,2,3],"materialId":1,"thickness":0.05},
        "2": {"id":2,"nodes":[1,3,4],"materialId":1,"thickness":0.05}
      },
      "supports": {
        "1": {"nodeId":1,"rx":true,"ry":true,"rz":true,"rrx":true,"rry":true,"rrz":true},
        "2": {"nodeId":4,"rx":true,"ry":true,"rz":true,"rrx":true,"rry":true,"rrz":true}
      },
      "loads": [
        {"type":"nodal","data":{"nodeId":3,"fx":0,"fy":0,"fz":-15,"mx":0,"my":0,"mz":0}}
      ]
    }"#;
    serde_json::from_str(json).unwrap()
}

#[test]
fn static_displacement_adjoint_matches_finite_difference() {
    let input = simple_3d_frame();
    let target = AdjointResponseTarget::Displacement {
        node_id: 3,
        dof: 2, // uz
    };

    let result = solve_static_adjoint_sensitivities(&input, &target).expect("adjoint solve failed");
    assert!(result.verified_by_finite_difference);
    assert!(!result.gradients.is_empty());

    for grad in &result.gradients {
        let fd = grad.finite_difference_check.expect("missing FD check");
        let diff = (grad.gradient - fd).abs();
        let scale = grad.gradient.abs().max(1e-6);
        assert!(
            diff / scale < 1e-4,
            "grad {} != fd {} (diff: {:.2e}) for var {}",
            grad.gradient,
            fd,
            diff,
            grad.variable_type
        );
    }
}

#[test]
fn static_drift_adjoint_matches_finite_difference() {
    let input = simple_3d_frame();
    let target = AdjointResponseTarget::Drift {
        node_i: 3,
        node_j: 2,
        dof: 0, // ux drift
    };

    let result = solve_static_adjoint_sensitivities(&input, &target).expect("adjoint solve failed");
    assert!(result.verified_by_finite_difference);
    assert!(!result.gradients.is_empty());
}

#[test]
fn plate_thickness_adjoint_matches_finite_difference() {
    let input = simple_plate_model();
    let target = AdjointResponseTarget::Displacement {
        node_id: 3,
        dof: 2, // uz
    };

    let result = solve_static_adjoint_sensitivities(&input, &target).expect("adjoint solve failed");
    assert!(result.verified_by_finite_difference);

    let thick_grad = result
        .gradients
        .iter()
        .find(|g| g.variable_type == "shell_thickness")
        .expect("missing shell_thickness gradient");
    let fd = thick_grad.finite_difference_check.unwrap();
    assert!((thick_grad.gradient - fd).abs() / thick_grad.gradient.abs().max(1e-6) < 1e-4);
}

#[test]
fn modal_frequency_adjoint_matches_finite_difference() {
    let input = simple_3d_frame();
    let target = AdjointResponseTarget::ModalFrequency { mode: 0 };

    let result = solve_modal_adjoint_sensitivities(&input, &target).expect("modal adjoint solve failed");
    assert!(result.verified_by_finite_difference);

    let mat_grad = result
        .gradients
        .iter()
        .find(|g| g.variable_type == "material_e")
        .expect("missing material_e gradient");
    let fd = mat_grad.finite_difference_check.unwrap();
    assert!((mat_grad.gradient - fd).abs() / mat_grad.gradient.abs().max(1e-6) < 1e-4);
}

#[test]
fn buckling_factor_adjoint_matches_finite_difference() {
    let input = simple_3d_frame();
    let target = AdjointResponseTarget::BucklingFactor { mode: 0 };

    let result = solve_buckling_adjoint_sensitivities(&input, &target).expect("buckling adjoint solve failed");
    assert!(result.verified_by_finite_difference);

    let mat_grad = result
        .gradients
        .iter()
        .find(|g| g.variable_type == "material_e")
        .expect("missing material_e gradient");
    let fd = mat_grad.finite_difference_check.unwrap();
    assert!((mat_grad.gradient - fd).abs() / mat_grad.gradient.abs().max(1e-6) < 1e-4);
}
