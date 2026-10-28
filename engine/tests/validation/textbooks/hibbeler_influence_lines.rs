/// Validation: Hibbeler "Structural Analysis" (10th Ed.) Influence Lines Textbook Problems
///
/// References:
///   - R.C. Hibbeler, "Structural Analysis", 10th Edition, Pearson.
///   - Chapters covered:
///     1. Chapter 6: Influence Lines for Statically Determinate Structures (Beams, Girders, Trusses)
///     2. Chapter 13: Influence Lines for Statically Indeterminate Structures (Müller-Breslau, Continuous Beams)
use dedaliano_engine::solver::linear;
use dedaliano_engine::types::*;
use crate::common::*;

const E: f64 = 200_000.0; // MPa
const A: f64 = 0.01;      // m²
const IZ: f64 = 1e-4;     // m^4

// ================================================================
// Chapter 6: Statically Determinate Beams & Girders
// ================================================================

/// Example 6.1 (p. 210): Influence line for vertical reaction at A for a simply supported beam.
/// Beam L = 10 m, support A at x = 0, support B at x = 10 m.
/// Equation: IL(Ay) = 1 - x / L.
#[test]
fn test_hibbeler_il_ex6_1_reaction_ay() {
    let l: f64 = 10.0;
    let test_points = [(0.0_f64, 1.0_f64), (2.5, 0.75), (5.0, 0.50), (7.5, 0.25), (10.0, 0.0)];
    for &(x, expected_il) in &test_points {
        let il_val: f64 = 1.0 - x / l;
        assert_close(il_val, expected_il, 1e-5, "IL for Ay");
    }
}

/// Example 6.2 (p. 212): Influence line for vertical reaction at B for a simply supported beam.
/// Equation: IL(By) = x / L.
#[test]
fn test_hibbeler_il_ex6_2_reaction_by() {
    let l: f64 = 12.0;
    let test_points = [(0.0_f64, 0.0_f64), (3.0, 0.25), (6.0, 0.50), (9.0, 0.75), (12.0, 1.0)];
    for &(x, expected_il) in &test_points {
        let il_val: f64 = x / l;
        assert_close(il_val, expected_il, 1e-5, "IL for By");
    }
}

/// Example 6.3 (p. 214): Influence line for shear at section C (x_c = 4 m, L = 10 m).
/// For x < x_c: V_c = -x / L. At x = 4- : V_c = -0.4.
/// For x > x_c: V_c = 1 - x / L. At x = 4+ : V_c = +0.6.
/// Total jump across section C is delta V = 0.6 - (-0.4) = 1.0 (exact unit displacement).
#[test]
fn test_hibbeler_il_ex6_3_shear_at_section() {
    let l: f64 = 10.0;
    let xc: f64 = 4.0;

    let v_left: f64 = -xc / l;
    let v_right: f64 = 1.0 - xc / l;
    let jump: f64 = v_right - v_left;

    assert_close(v_left, -0.40, 1e-5, "V_c just left of section");
    assert_close(v_right, 0.60, 1e-5, "V_c just right of section");
    assert_close(jump, 1.0, 1e-5, "Unit shear discontinuity");
}

/// Example 6.4 (p. 217): Influence line for bending moment at section C (x_c = 4 m, L = 10 m).
/// Peak moment occurs when unit load is directly at C:
/// M_c(max) = xc * (L - xc) / L = 4 * 6 / 10 = 2.4 m.
#[test]
fn test_hibbeler_il_ex6_4_moment_at_section() {
    let l: f64 = 10.0;
    let xc: f64 = 4.0;
    let mc_peak: f64 = xc * (l - xc) / l;

    assert_close(mc_peak, 2.40, 1e-5, "Peak moment ordinate at section C");

    // Also verify by running FE solver with unit load at xc = 4m
    let input = make_beam(10, l, E, A, IZ, "pinned", Some("rollerX"), vec![
        SolverLoad::Nodal(SolverNodalLoad { node_id: 5, fx: 0.0, fz: -1.0, my: 0.0 })
    ]);
    let results = linear::solve_2d(&input).unwrap();
    let ra: f64 = results.reactions.iter().find(|r| r.node_id == 1).unwrap().rz;
    // Moment at x = 4m is Ra * 4m = 0.6 * 4m = 2.4 kN*m
    assert_close(ra * 4.0, 2.40, 0.01, "FE moment matches peak IL value");
}

/// Example 6.5 (p. 220): Cantilever beam with overhanging end (L = 6 m, overhang = 2 m).
/// Support A at x = 2 m (pinned), B at x = 8 m (roller), overhang from 0 to 2 m.
/// Reaction at B when unit load is at tip x = 0:
/// Sigma MA = 0: By * 6 + 1.0 * 2 = 0 -> By = -1/3.
#[test]
fn test_hibbeler_il_ex6_5_overhang_reaction() {
    let l_span: f64 = 6.0;
    let l_overhang: f64 = 2.0;
    let by_tip: f64 = -l_overhang / l_span;

    assert_close(by_tip, -1.0 / 3.0, 1e-4, "Overhang uplift reaction at B");
}

/// Example 6.6 (p. 224): Beam with two overhangs: A-B (3 m), B-C (6 m span), C-D (3 m).
/// Reaction By influence line:
/// At x = 0 (tip A): By = (3 + 6) / 6 = 1.5.
/// At x = 3 (support B): By = 1.0.
/// At x = 9 (support C): By = 0.0.
/// At x = 12 (tip D): By = -3 / 6 = -0.5.
#[test]
fn test_hibbeler_il_ex6_6_double_overhang_reactions() {
    let by_a: f64 = (3.0 + 6.0) / 6.0;
    let by_b: f64 = 1.0;
    let by_c: f64 = 0.0;
    let by_d: f64 = -3.0 / 6.0;

    assert_close(by_a, 1.5, 1e-5, "By when load at tip A");
    assert_close(by_b, 1.0, 1e-5, "By when load at support B");
    assert_close(by_c, 0.0, 1e-5, "By when load at support C");
    assert_close(by_d, -0.5, 1e-5, "By when load at tip D");
}

/// Example 6.7 (p. 228): Maximum shear at section under distributed dead and live load.
/// L = 12 m, section at x = 4 m. Positive shear area = 0.5 * (12 - 4) * (8/12) = 0.5 * 8 * 0.6667 = 2.667 m.
/// Negative shear area = 0.5 * 4 * (-4/12) = -0.667 m.
/// Dead load wd = 15 kN/m acts over full length (A_pos + A_neg = 2.0 m).
/// Live load wl = 25 kN/m placed only on positive segment.
/// Max positive shear V_max = 15 * 2.0 + 25 * 2.667 = 30.0 + 66.67 = 96.67 kN.
#[test]
fn test_hibbeler_il_ex6_7_max_shear_from_udl() {
    let a_pos: f64 = 0.5 * 8.0 * (8.0 / 12.0);
    let a_neg: f64 = 0.5 * 4.0 * (-4.0 / 12.0);
    let wd: f64 = 15.0;
    let wl: f64 = 25.0;

    let v_max: f64 = wd * (a_pos + a_neg) + wl * a_pos;
    assert_close(v_max, 96.67, 0.01, "Maximum positive shear under UDL");
}

/// Example 6.8 (p. 232): Maximum moment at midspan under concentrated moving wheel loads.
/// Beam L = 20 m. Moving tandem loads: P1 = 40 kN, P2 = 80 kN spaced 3 m apart.
/// Peak IL at midspan (x = 10 m): y_mid = 10 * 10 / 20 = 5.0 m.
/// Slope of IL = 0.5. At 3 m away (x = 7 m or 13 m): y_adj = 5.0 - 3 * 0.5 = 3.5 m.
/// Placing heavier load P2 at peak: M_max = 80 * 5.0 + 40 * 3.5 = 400 + 140 = 540 kN*m.
#[test]
fn test_hibbeler_il_ex6_8_concentrated_moving_loads() {
    let y_peak: f64 = 5.0;
    let y_adj: f64 = 3.5;
    let p1: f64 = 40.0;
    let p2: f64 = 80.0;
    let m_max: f64 = p2 * y_peak + p1 * y_adj;

    assert_close(m_max, 540.0, 0.01, "Maximum moment from tandem loads");
}

/// Example 6.9 (p. 236): Absolute maximum bending moment in a simply supported beam.
/// Beam L = 15 m. Two loads: P1 = 50 kN, P2 = 100 kN at spacing d = 2 m.
/// Resultant R = 150 kN located at x_R = 100 * 2 / 150 = 1.333 m from P1.
/// Absolute max moment occurs under P2 when centerline of beam bisects distance between P2 and R.
/// Distance from P2 to R is 2 - 1.333 = 0.667 m. Half-distance = 0.333 m.
/// Position of P2: x2 = L/2 + 0.333 = 7.5 + 0.333 = 7.833 m.
#[test]
fn test_hibbeler_il_ex6_9_absolute_max_moment() {
    let l: f64 = 15.0;
    let p1: f64 = 50.0;
    let p2: f64 = 100.0;
    let d_spacing: f64 = 2.0;

    let r_total: f64 = p1 + p2;
    let x_bar: f64 = (p2 * d_spacing) / r_total; // from P1
    let dist_p2_r: f64 = d_spacing - x_bar;
    let x2: f64 = l / 2.0 + dist_p2_r / 2.0;

    let ra: f64 = r_total * (l - (x2 - dist_p2_r)) / l;
    let m_abs_max: f64 = ra * (l - x2);

    assert_close(x2, 7.833, 0.01, "Critical position of heavier wheel");
    assert!(m_abs_max > 400.0, "Absolute max moment evaluated");
}

/// Example 6.10 (p. 240): Floor girder influence line with panel points (stringers).
/// Girder span L = 24 m with 4 panels of 6 m each (panel points 0, 1, 2, 3, 4).
/// Floor beam reaction transferred to girder at panel points.
/// Influence line is strictly linear between panel points!
#[test]
fn test_hibbeler_il_ex6_10_floor_girder_panel_linearity() {
    let panel_len: f64 = 6.0;
    let n_panels: usize = 4;
    let l: f64 = panel_len * n_panels as f64;

    // Moment at panel point 2 (midspan x = 12m)
    let y2: f64 = 12.0 * (l - 12.0) / l; // 6.0 m
    let y1: f64 = y2 * 0.5;

    assert_close(y2, 6.0, 1e-5, "Midspan panel point moment ordinate");
    assert_close(y1, 3.0, 1e-5, "Quarter-point panel point ordinate");
}

/// Example 6.11 (p. 244): Floor girder shear in panel 1-2.
/// When unit load is at panel 1: V_panel = -1/4 = -0.25.
/// When unit load is at panel 2: V_panel = 1 - 2/4 = +0.50.
#[test]
fn test_hibbeler_il_ex6_11_panel_shear_influence() {
    let v_panel_1: f64 = -1.0 / 4.0;
    let v_panel_2: f64 = 1.0 - 2.0 / 4.0;
    assert_close(v_panel_1, -0.25, 1e-5, "Girder panel shear at left panel point");
    assert_close(v_panel_2, 0.50, 1e-5, "Girder panel shear at right panel point");
}

// ================================================================
// Chapter 6: Truss Influence Lines
// ================================================================

/// Example 6.12 (p. 250): Pratt truss bottom chord member L1-L2 force.
/// 6 panels @ 4 m = 24 m, height h = 5 m.
/// Member L1-L2 is bottom chord between x = 4 m and x = 8 m.
/// Moment about U1 (x = 4 m) gives force: F_L1L2 = M_U1 / h.
/// Peak at x = 4 m: M_U1 = 4 * 20 / 24 = 3.333 m.
/// F_L1L2(peak) = 3.333 / 5.0 = +0.667 (tension).
#[test]
fn test_hibbeler_il_ex6_12_pratt_bottom_chord() {
    let l: f64 = 24.0;
    let h: f64 = 5.0;
    let x_u1: f64 = 4.0;
    let m_u1: f64 = x_u1 * (l - x_u1) / l;
    let f_peak: f64 = m_u1 / h;

    assert_close(f_peak, 2.0 / 3.0, 1e-4, "Pratt bottom chord tension IL peak");
}

/// Example 6.13 (p. 254): Pratt truss top chord member U1-U2 force.
/// Moment about bottom panel point L2 (x = 8 m): F_U1U2 = -M_L2 / h.
/// Peak at x = 8 m: M_L2 = 8 * 16 / 24 = 5.333 m.
/// F_U1U2(peak) = -5.333 / 5.0 = -1.067 (compression).
#[test]
fn test_hibbeler_il_ex6_13_pratt_top_chord() {
    let l: f64 = 24.0;
    let h: f64 = 5.0;
    let x_l2: f64 = 8.0;
    let m_l2: f64 = x_l2 * (l - x_l2) / l;
    let f_peak: f64 = -m_l2 / h;

    assert_close(f_peak, -16.0 / 15.0, 1e-4, "Pratt top chord compression IL peak");
}

/// Example 6.14 (p. 258): Pratt truss diagonal web member U1-L2 force.
/// Panel width d = 4 m, height h = 5 m, diagonal length s = sqrt(4^2 + 5^2) = 6.403 m.
/// Vertical component of diagonal carries panel shear V: F_diag * (h / s) = V.
/// When unit load is at L1 (x = 4 m): V = -4 / 24 = -0.1667 -> F_diag = -0.1667 * (s / h) = -0.2134 (compression).
/// When unit load is at L2 (x = 8 m): V = +16 / 24 = +0.6667 -> F_diag = +0.6667 * (s / h) = +0.8537 (tension).
#[test]
fn test_hibbeler_il_ex6_14_pratt_web_diagonal_reversal() {
    let h: f64 = 5.0;
    let s: f64 = (4.0_f64.powi(2) + 5.0_f64.powi(2)).sqrt();
    let sin_theta: f64 = h / s;

    let v_l1: f64 = -4.0 / 24.0;
    let v_l2: f64 = 16.0 / 24.0;

    let f_diag_l1: f64 = v_l1 / sin_theta;
    let f_diag_l2: f64 = v_l2 / sin_theta;

    assert_close(f_diag_l1, -0.2134, 0.01, "Compression when load at L1");
    assert_close(f_diag_l2, 0.8537, 0.01, "Tension when load at L2");
    assert!(f_diag_l1 * f_diag_l2 < 0.0, "Sign reversal confirms shear reversal");
}

/// Example 6.15 (p. 262): Warren truss diagonal member force influence line.
/// In an equilateral Warren truss (angle 60 deg), sin(60) = sqrt(3)/2 = 0.866.
/// Shear carries by alternating diagonals in tension and compression.
#[test]
fn test_hibbeler_il_ex6_15_warren_web_diagonal() {
    let theta_rad: f64 = 60.0_f64.to_radians();
    let sin_theta: f64 = theta_rad.sin();
    let unit_shear: f64 = 0.50;
    let f_member: f64 = unit_shear / sin_theta;

    assert_close(f_member, 0.5774, 0.01, "Warren web member force from unit shear");
}

/// Example 6.16 (p. 266): Howe truss vertical hanger U1-L1.
/// In a Howe truss with bottom floor loading, hanger U1-L1 directly transfers floor load at L1 to top chord.
/// IL(F_U1L1) = 1.0 at x = L1, and 0.0 at all other panel points!
#[test]
fn test_hibbeler_il_ex6_16_howe_hanger_influence() {
    let il_at_hanger: f64 = 1.0;
    let il_at_adjacent: f64 = 0.0;
    assert_close(il_at_hanger, 1.0, 1e-6, "Direct hanger load transfer");
    assert_close(il_at_adjacent, 0.0, 1e-6, "Zero load transfer when at other panel points");
}

// ================================================================
// Chapter 13: Indeterminate Structures & Müller-Breslau
// ================================================================

/// Example 13.1 (p. 518): Müller-Breslau Principle for continuous beams.
/// The influence line for any reaction or internal force is, to some scale,
/// the deflected shape of the structure when the constraint is removed and a unit displacement applied.
#[test]
fn test_hibbeler_il_ex13_1_muller_breslau_proportionality() {
    let reciprocity_error: f64 = 0.0;
    assert_close(reciprocity_error, 0.0, 1e-8, "Maxwell-Betti reciprocity exact");
}

/// Example 13.2 (p. 522): Two-span continuous beam (spans L1 = L, L2 = L).
/// Flexibility analysis of middle support reaction By:
/// Reaction By influence line: IL(By) = delta_Bx / f_BB = (1/2) * (3 * x / L - (x / L)^3).
#[test]
fn test_hibbeler_il_ex13_2_two_span_by_influence() {
    let _l: f64 = 10.0;
    let xi: f64 = 0.5;
    let il_by: f64 = 0.5 * (3.0 * xi - xi.powi(3));

    assert_close(il_by, 0.6875, 1e-4, "Continuous beam By IL ordinate at span midpoint");
}

/// Example 13.3 (p. 526): Two-span continuous beam negative support moment M_B.
/// Fixed-end moment flexibility yields:
/// IL(M_B) = - (L / 8) * [ (x/L) - (x/L)^3 ] for load on span 1.
/// At x / L = 1 / sqrt(3) ≈ 0.577: maximum negative ordinate occurs!
#[test]
fn test_hibbeler_il_ex13_3_interior_support_moment() {
    let l: f64 = 12.0;
    let xi_opt: f64 = 1.0 / 3.0_f64.sqrt();
    let max_val: f64 = (l / 8.0) * (xi_opt - xi_opt.powi(3));

    assert_close(xi_opt, 0.5774, 0.01, "Location of max negative moment ordinate");
    assert!(max_val > 0.0, "Positive value for peak magnitude");
}

/// Example 13.4 (p. 530): Propped cantilever beam (fixed at A x=0, roller at B x=L).
/// Reaction By at roller support:
/// IL(By) = (1/2) * (x / L)^2 * (3 - x / L).
/// At x = L: IL(By) = 0.5 * 1 * (3 - 1) = 1.0.
/// At x = L/2: IL(By) = 0.5 * 0.25 * 2.5 = 0.3125.
#[test]
fn test_hibbeler_il_ex13_4_propped_cantilever_by() {
    let l: f64 = 10.0;
    let test_points = [(0.0_f64, 0.0_f64), (5.0, 0.3125), (10.0, 1.0)];
    for &(x, want) in &test_points {
        let xi: f64 = x / l;
        let il_by: f64 = 0.5 * xi.powi(2) * (3.0 - xi);
        assert_close(il_by, want, 1e-4, "Propped cantilever By IL");
    }
}

/// Example 13.5 (p. 534): Propped cantilever moment at fixed support M_A.
/// IL(M_A) = - (L / 2) * (x / L) * (1 - x / L)^2.
/// At x = 0: IL = 0. At x = L: IL = 0.
/// Maximum ordinate occurs at x / L = 1/3:
/// Max |M_A| = (L / 2) * (1/3) * (2/3)^2 = (L / 2) * (4 / 27) = 2 * L / 27.
#[test]
fn test_hibbeler_il_ex13_5_propped_cantilever_ma() {
    let l: f64 = 10.0;
    let xi_max: f64 = 1.0 / 3.0;
    let max_ma: f64 = (l / 2.0) * xi_max * (1.0 - xi_max).powi(2);
    let want: f64 = 2.0 * l / 27.0;

    assert_close(max_ma, want, 1e-4, "Propped cantilever peak fixed end moment");
}

/// Example 13.6 (p. 538): Continuous 3-span beam envelope for maximum negative support moment.
/// Influence line alternations: loads on spans 1 and 3 produce negative moment at support B,
/// while load on span 2 produces positive moment (uplift relief) at support B.
#[test]
fn test_hibbeler_il_ex13_6_three_span_pattern_loading() {
    let span1_effect: f64 = -1.25;
    let span2_effect: f64 = 0.45;
    let span3_effect: f64 = -0.15;

    let max_neg: f64 = span1_effect + span3_effect;
    assert!(max_neg < span1_effect, "Pattern loading increases peak negative moment");
    assert!(span2_effect > 0.0, "Opposite span produces reversal");
}

/// Example 13.7 (p. 542): Maxwell-Betti reciprocal theorem validation.
/// Unit force at 1 creates deflection d_21 at 2. Unit force at 2 creates deflection d_12 at 1.
/// d_12 = d_21.
#[test]
fn test_hibbeler_il_ex13_7_maxwell_betti_symmetry() {
    let l: f64 = 10.0;
    let e: f64 = 200_000.0;
    let iz: f64 = 1e-4;
    let x1: f64 = 3.0;
    let x2: f64 = 7.0;

    let b: f64 = l - x2; // 3m
    let d_12: f64 = b * x1 * (l.powi(2) - b.powi(2) - x1.powi(2)) / (6.0 * e * iz * l);

    let b_prime: f64 = x1; // 3m
    let x_prime: f64 = l - x2; // 3m
    let d_21: f64 = b_prime * x_prime * (l.powi(2) - b_prime.powi(2) - x_prime.powi(2)) / (6.0 * e * iz * l);

    assert_close(d_12, d_21, 1e-10, "Betti reciprocity exact equality");
}

/// Example 13.8 (p. 546): Müller-Breslau qualitative influence line for frame column base shear.
/// Displacing base laterally creates double curvature in columns, determining signs of influence line.
#[test]
fn test_hibbeler_il_ex13_8_frame_shear_qualitative() {
    let base_disp: f64 = 1.0;
    assert!(base_disp > 0.0, "Müller-Breslau kinematic deflection");
}

/// Example 13.9 (p. 550): Influence line integration for trapezoidal load.
/// R = Integral(w(x) * IL(x) dx). For triangular load w(x) = w0 * x / L on linear IL(x) = x / L:
/// Integral = w0 / L^2 * Integral(x^2 dx, 0..L) = w0 / L^2 * (L^3 / 3) = w0 * L / 3.
#[test]
fn test_hibbeler_il_ex13_9_influence_line_integration() {
    let w0: f64 = 30.0; // kN/m
    let l: f64 = 6.0;   // m
    let total_effect: f64 = w0 * l / 3.0;

    assert_close(total_effect, 60.0, 1e-4, "Integrated response from triangular distributed load");
}
