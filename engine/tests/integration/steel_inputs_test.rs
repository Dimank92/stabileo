use dedaliano_engine::postprocess::steel_inputs::*;

#[test]
fn test_moment_gradient_symmetric_parabola() {
    // Parabolic moment diagram on simple beam: M(t) = 4 * Mmax * t * (1 - t)
    // At t = 0.25: M = 4 * 100 * 0.25 * 0.75 = 75
    // At t = 0.50: M = 100
    // At t = 0.75: M = 75
    let stations = vec![
        StationMoment { t: 0.0, m: 0.0 },
        StationMoment { t: 0.25, m: 75.0 },
        StationMoment { t: 0.50, m: 100.0 },
        StationMoment { t: 0.75, m: 75.0 },
        StationMoment { t: 1.0, m: 0.0 },
    ];

    let res = compute_moment_gradient_cb(&stations, 0.0, 1.0, Some("I"), false);
    assert_eq!(res.basis, CbBasis::Computed);
    assert_eq!(res.curvature, "single");
    // Denom = 2.5*100 + 3*75 + 4*100 + 3*75 = 250 + 225 + 400 + 225 = 1100
    // Cb = 12.5*100 / 1100 = 1250 / 1100 ≈ 1.136
    assert!((res.cb - 1.13636).abs() < 1e-3);
}

#[test]
fn test_moment_gradient_cantilever_free_end() {
    let stations = vec![
        StationMoment { t: 0.0, m: 100.0 },
        StationMoment { t: 0.5, m: 25.0 },
        StationMoment { t: 1.0, m: 0.0 },
    ];

    let res = compute_moment_gradient_cb(&stations, 0.0, 1.0, Some("I"), true);
    assert_eq!(res.basis, CbBasis::UnityRequiredCantilever);
    assert_eq!(res.cb, 1.0);
}

#[test]
fn test_moment_gradient_singly_symmetric_double_curvature_out_of_scope() {
    // Channel (singly-symmetric) with double curvature moment
    let stations = vec![
        StationMoment { t: 0.0, m: -50.0 },
        StationMoment { t: 0.5, m: 0.0 },
        StationMoment { t: 1.0, m: 50.0 },
    ];

    let res = compute_moment_gradient_cb(&stations, 0.0, 1.0, Some("U"), false);
    assert_eq!(res.basis, CbBasis::UnityOutOfScope);
    assert_eq!(res.cb, 1.0);
}

#[test]
fn test_section_constants_i_shape() {
    let b = 0.15;
    let h = 0.30;
    let tw = 0.007;
    let tf = 0.011;
    let iy = 6e-6;
    let iz = 8e-5;

    let res = compute_steel_section_constants(
        Some("I"),
        Some(b),
        Some(h),
        Some(tw),
        Some(tf),
        Some(iz),
        Some(iy),
        None,
    );

    assert_eq!(res.j_basis, "thinWalled");
    assert!(res.j > 0.0);
    assert!(res.cw.is_some());
    let cw = res.cw.unwrap();
    // Cw = Iy * (h - tf)^2 / 4
    let h0 = h - tf;
    let expected_cw = iy * h0 * h0 / 4.0;
    assert!((cw - expected_cw).abs() < 1e-12);
    assert!(res.zx.is_some());
    assert!(res.zy.is_some());
}

#[test]
fn test_unbraced_lengths_calculation() {
    // 3 elements in chain, 2m each => total L = 6m
    let chain = vec![1, 2, 3];
    let res = compute_member_unbraced_lengths(2.0, &chain, None, None);
    assert_eq!(res.l, 6.0);
    assert_eq!(res.lb, 6.0);
    assert_eq!(res.source, "chain");

    // Bracing points at 2.0m and 4.0m
    let bracing = vec![2.0, 4.0];
    let res_braced = compute_member_unbraced_lengths(2.0, &chain, None, Some(&bracing));
    assert_eq!(res_braced.l, 6.0);
    assert_eq!(res_braced.lb, 2.0);
    assert_eq!(res_braced.source, "braced_points");
}

#[test]
fn test_torsional_buckling_e4_scope() {
    // Closed tube is out of scope
    let res_rhs = evaluate_torsional_buckling_e4(
        Some("RHS"),
        Some(1e-6),
        Some(1e-8),
        2e11,
        7.7e10,
        1.0,
        3.0,
        0.005,
        0.01,
    );
    assert!(!res_rhs.applicable);
    assert_eq!(res_rhs.scope, "outOfScope");

    // Open I-shape with valid J and Cw
    let res_i = evaluate_torsional_buckling_e4(
        Some("I"),
        Some(1e-7),
        Some(1e-8),
        2e11,
        7.7e10,
        1.0,
        3.0,
        0.005,
        0.01,
    );
    assert!(res_i.applicable);
    assert!(res_i.fez.is_some());
}

#[test]
fn test_flange_local_buckling_f62() {
    let b = 0.20;
    let tf = 0.010;
    let fy = 250e6;
    let e = 200e9;
    let iy = 1e-5;

    let res = evaluate_flange_local_buckling_f62(Some("I"), b, tf, fy, e, iy);
    assert_eq!(res.state, "classified");
    assert!(res.flange_slenderness.is_some());
    // lambda_f = (b/2)/tf = 0.10 / 0.01 = 10.0
    assert_eq!(res.flange_slenderness.unwrap(), 10.0);
}

#[test]
fn test_effective_net_area_d3() {
    let ag = 0.0020; // 20 cm2 = 0.002 m2
    let holes = vec![HoleData {
        diameter_mm: 20.0,
        count_per_line: 2,
        lines: 2,
        thickness_m: 0.010,
    }];
    let x_bar = 0.025;
    let conn_l = 0.150;

    let res = compute_effective_net_area_d3(ag, Some(&holes), None, x_bar, conn_l);
    // Hole deduction: 2 lines * (20 + 2)mm * 10mm = 2 * 0.022 * 0.010 = 0.00044 m2
    // An = 0.0020 - 0.00044 = 0.00156 m2
    assert!((res.an - 0.00156).abs() < 1e-6);
    // U = 1 - 0.025 / 0.150 = 1 - 0.1667 = 0.8333
    assert!((res.u - 0.83333).abs() < 1e-3);
    assert!((res.ae - (res.an * res.u)).abs() < 1e-8);
}

#[test]
fn test_material_family_resolution() {
    let f24 = resolve_material_family("ASTM A36", 250.0);
    assert_eq!(f24.family, "F-24");

    let f36 = resolve_material_family("S355", 355.0);
    assert_eq!(f36.family, "F-36");
}
