//! Tests for user model checking on solve (Issue #498).
//!
//! Roadmap requirement:
//! "After each solve, in the background:
//!  - global equilibrium recomputed from the model's loads (not from K);
//!  - a rotated and renumbered twin of the model solved and compared (the metamorphic check);
//!  - an independent re-assembly of K and f (#403) compared entry by entry.
//!  Any disagreement: the result is not shown as verified, and the reason is."

mod common;

use common::*;
use dedaliano_engine::certify::user_model_check::{check_user_model_2d, check_user_model_3d, UserModelCheckReport};
use dedaliano_engine::certify::{certify_linear_2d_full, certify_linear_3d_full, CertificateLevel};
use dedaliano_engine::solver::linear::{solve_2d, solve_3d};
use dedaliano_engine::types::*;

#[test]
fn test_user_model_check_2d_clean_portal_frame() {
    let input = make_portal_frame(3.0, 4.0, 200_000.0, 0.01, 1e-4, 15.0, -25.0);
    let results = solve_2d(&input).expect("solve_2d succeeds");
    let report: UserModelCheckReport = check_user_model_2d(&input, &results);

    assert!(report.passed, "clean portal frame must pass all checks: {:?}", report.failure_reason);
    assert!(report.equilibrium.passed, "equilibrium check must pass");
    assert!(report.metamorphic_twin.passed, "metamorphic twin check must pass");
    assert!(report.dual_assembly.passed, "dual assembly check must pass");
    assert!(report.failure_reason.is_none());
}

#[test]
fn test_user_model_check_2d_equilibrium_detects_imbalance() {
    let input = make_portal_frame(3.0, 4.0, 200_000.0, 0.01, 1e-4, 15.0, -25.0);
    let mut results = solve_2d(&input).expect("solve_2d succeeds");

    // Introduce an artificial 30 kN reaction error (as if reaction assembly dropped a component)
    if let Some(r) = results.reactions.first_mut() {
        r.rz += 30.0;
    }

    let report = check_user_model_2d(&input, &results);
    assert!(!report.passed, "unbalanced model must fail user model check");
    assert!(!report.equilibrium.passed, "equilibrium check must catch imbalance");
    assert!(report.failure_reason.is_some(), "failure reason must be provided");
    let reason = report.failure_reason.unwrap();
    assert!(reason.to_lowercase().contains("equilibrium"), "reason should cite equilibrium: {}", reason);
}

#[test]
fn test_user_model_check_2d_metamorphic_detects_tampered_displacements() {
    let input = make_portal_frame(3.0, 4.0, 200_000.0, 0.01, 1e-4, 15.0, -25.0);
    let mut results = solve_2d(&input).expect("solve_2d succeeds");

    // Perturb displacements so the metamorphic twin comparison fails
    if let Some(d) = results.displacements.get_mut(1) {
        d.ux += 0.08;
    }

    let report = check_user_model_2d(&input, &results);
    assert!(!report.passed, "tampered displacements must fail metamorphic twin check");
    assert!(!report.metamorphic_twin.passed, "metamorphic twin check must fail");
    assert!(report.failure_reason.is_some(), "failure reason must be provided");
    let reason = report.failure_reason.unwrap();
    assert!(reason.to_lowercase().contains("metamorphic") || reason.to_lowercase().contains("disagree"), "reason: {}", reason);
}

#[test]
fn test_user_model_check_2d_integration_with_certify() {
    let input = make_portal_frame(3.0, 4.0, 200_000.0, 0.01, 1e-4, 15.0, -25.0);
    let cert = certify_linear_2d_full(&input).expect("certify succeeds");
    assert!(cert.certificate.is_certified());
    assert_eq!(cert.certificate_level, CertificateLevel::EndToEnd);
    assert!(cert.user_model_check.passed);
}

#[test]
fn test_user_model_check_3d_clean_cantilever() {
    let nodes = vec![
        (1, 0.0, 0.0, 0.0),
        (2, 2.0, 0.0, 0.0),
    ];
    let mats = vec![(1, 210_000.0, 0.3)];
    let secs = vec![(1, 0.01, 1e-4, 1e-4, 1e-5)];
    let elems = vec![(1, "frame", 1, 2, 1, 1)];
    let sups = vec![(1, vec![true, true, true, true, true, true])];
    let loads = vec![
        SolverLoad3D::Nodal(SolverNodalLoad3D {
            node_id: 2,
            fx: 0.0,
            fy: 5.0,
            fz: -10.0,
            mx: 0.0,
            my: 2.0,
            mz: 0.0,
            bw: None,
        }),
    ];
    let input = make_3d_input(nodes, mats, secs, elems, sups, loads);
    let results = solve_3d(&input).expect("solve_3d succeeds");

    let report = check_user_model_3d(&input, &results);
    assert!(report.passed, "clean 3D cantilever must pass: {:?}", report.failure_reason);
    assert!(report.equilibrium.passed, "3D equilibrium passed");
    assert!(report.metamorphic_twin.passed, "3D metamorphic twin passed");
    assert!(report.dual_assembly.passed, "3D dual assembly passed");

    let cert = certify_linear_3d_full(&input).expect("certify 3D succeeds");
    assert!(cert.certificate.is_certified());
    assert!(cert.user_model_check.passed);
}
