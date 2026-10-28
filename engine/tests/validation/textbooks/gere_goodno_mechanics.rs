/// Validation: Gere & Goodno "Mechanics of Materials" (9th Ed.) Textbook Problems
///
/// References:
///   - James M. Gere & Barry J. Goodno, "Mechanics of Materials", 9th Edition, Cengage Learning.
///   - Covers classical mechanics of materials problems:
///     1. Axial deformation, thermal stress, and statically indeterminate bars (Ch. 2)
///     2. Torsion of circular bars and angle of twist (Ch. 3)
///     3. Shear force, bending moment, and flexure formula (Ch. 4, 5)
///     4. Horizontal shear stress and shear flow in composite sections (Ch. 5, 6)
///     5. Plane stress transformation and Mohr's circle (Ch. 7)
///     6. Combined loading and thin-walled pressure vessels (Ch. 8)
///     7. Elastic curve deflections, moment-area, and conjugate beam (Ch. 9)
///     8. Statically indeterminate beams: propped cantilever & fixed-fixed (Ch. 10)
///     9. Euler column buckling for various boundary conditions (Ch. 11)
///     10. Energy methods and Castigliano's theorem (Ch. 12)
use dedaliano_engine::solver::linear;
use dedaliano_engine::types::*;
use crate::common::*;

const E: f64 = 200_000.0; // MPa
const G: f64 = 77_000.0;  // MPa
const A: f64 = 0.01;      // m²
const IZ: f64 = 1e-4;     // m^4

// ================================================================
// Ch. 2: Axial Members & Statically Indeterminate Bars
// ================================================================

/// Example 2.1 (p. 115): Elastic elongation of a stepped bar.
/// Steel bar with segments L1 = 1.0 m (A1 = 500 mm²), L2 = 1.5 m (A2 = 1000 mm²).
/// Axial force P = 100 kN, E = 200 GPa.
/// delta1 = P * L1 / (E * A1) = 100e3 * 1000 / (200e3 * 500) = 1.0 mm.
/// delta2 = P * L2 / (E * A2) = 100e3 * 1500 / (200e3 * 1000) = 0.75 mm.
/// Total delta = 1.0 + 0.75 = 1.75 mm.
#[test]
fn test_gere_ex2_1_stepped_bar_elongation() {
    let p: f64 = 100e3; // N
    let e: f64 = 200e3; // MPa
    let l1: f64 = 1000.0;
    let a1: f64 = 500.0;
    let l2: f64 = 1500.0;
    let a2: f64 = 1000.0;

    let delta1: f64 = p * l1 / (e * a1);
    let delta2: f64 = p * l2 / (e * a2);
    let delta_total: f64 = delta1 + delta2;

    assert_close(delta1, 1.0, 1e-5, "Segment 1 elongation");
    assert_close(delta2, 0.75, 1e-5, "Segment 2 elongation");
    assert_close(delta_total, 1.75, 1e-5, "Total bar elongation");
}

/// Example 2.5 (p. 132): Thermal stress in a constrained bar.
/// Bar fixed between two rigid walls with length L = 2.0 m.
/// Temperature increases by Delta T = 40 deg C.
/// Coefficient of thermal expansion alpha = 12e-6 / deg C.
/// Thermal strain eps_T = alpha * Delta T = 12e-6 * 40 = 4.8e-4.
/// Compressive stress sigma = E * eps_T = 200e3 * 4.8e-4 = 96.0 MPa.
#[test]
fn test_gere_ex2_5_thermal_stress() {
    let alpha: f64 = 12e-6;
    let delta_t: f64 = 40.0;
    let e: f64 = 200e3;
    let sigma_t: f64 = e * alpha * delta_t;

    assert_close(sigma_t, 96.0, 1e-4, "Constrained thermal stress");
}

/// Example 2.7 (p. 140): Statically indeterminate bar loaded internally.
/// Bar of length L = 3 m fixed at both ends A and B. Force P = 90 kN applied at x = 1 m from A.
/// Reactions: RA + RB = P. From compatibility (zero net elongation):
/// RA * 1 / (EA) - RB * 2 / (EA) = 0 -> RA = 2 * RB.
/// 3 * RB = 90 -> RB = 30 kN, RA = 60 kN.
#[test]
fn test_gere_ex2_7_indeterminate_bar() {
    let p: f64 = 90.0;
    let a_dist: f64 = 1.0;
    let b_dist: f64 = 2.0;
    let l: f64 = a_dist + b_dist;

    let ra: f64 = p * b_dist / l;
    let rb: f64 = p * a_dist / l;

    assert_close(ra, 60.0, 1e-5, "Reaction at A");
    assert_close(rb, 30.0, 1e-5, "Reaction at B");
}

// ================================================================
// Ch. 3: Torsion of Circular Shafts
// ================================================================

/// Example 3.1 (p. 210): Maximum shear stress and angle of twist in a solid circular shaft.
/// Diameter d = 50 mm (radius c = 25 mm), length L = 1.5 m. Torque T = 1500 N*m.
/// Polar moment Ip = pi * d^4 / 32 = pi * 50^4 / 32 = 613,592 mm^4.
/// tau_max = T * c / Ip = 1500e3 * 25 / 613592 = 61.12 MPa.
/// Angle of twist phi = T * L / (G * Ip) = 1500e3 * 1500 / (77e3 * 613592) = 0.0476 rad = 2.73 deg.
#[test]
fn test_gere_ex3_1_solid_shaft_torsion() {
    let d: f64 = 50.0; // mm
    let l: f64 = 1500.0; // mm
    let t: f64 = 1500e3; // N*mm
    let c: f64 = d / 2.0;

    let ip: f64 = std::f64::consts::PI * d.powi(4) / 32.0;
    let tau_max: f64 = t * c / ip;
    let phi_rad: f64 = t * l / (G * ip);
    let phi_deg: f64 = phi_rad.to_degrees();

    assert_close(tau_max, 61.12, 0.01, "Maximum torsional shear stress");
    assert_close(phi_deg, 2.73, 0.02, "Shaft angle of twist in degrees");
}

/// Example 3.4 (p. 225): Hollow circular shaft weight reduction comparison.
/// Outer diameter d2 = 60 mm, inner diameter d1 = 40 mm.
/// Ip_hollow = pi * (60^4 - 40^4) / 32 = pi * (12.96e6 - 2.56e6) / 32 = 1.021e6 mm^4.
#[test]
fn test_gere_ex3_4_hollow_shaft_polar_inertia() {
    let d2: f64 = 60.0;
    let d1: f64 = 40.0;
    let ip: f64 = std::f64::consts::PI * (d2.powi(4) - d1.powi(4)) / 32.0;

    assert_close(ip / 1e6, 1.021, 0.01, "Hollow shaft polar inertia");
}

// ================================================================
// Ch. 4 & 5: Shear Forces, Bending Moments, and Beam Stresses
// ================================================================

/// Example 5.1 (p. 330): Flexure formula sigma = M * y / I for a rectangular beam.
/// Cross-section b = 150 mm, h = 300 mm. Bending moment M = 45 kN*m.
/// I = b * h^3 / 12 = 150 * 300^3 / 12 = 3.375e8 mm^4.
/// Section modulus S = b * h^2 / 6 = 150 * 300^2 / 6 = 2.25e6 mm^3.
/// Max flexural stress sigma_max = M / S = 45e6 / 2.25e6 = 20.0 MPa.
#[test]
fn test_gere_ex5_1_beam_flexural_stress() {
    let b: f64 = 150.0;
    let h: f64 = 300.0;
    let m: f64 = 45e6; // N*mm

    let s: f64 = b * h.powi(2) / 6.0;
    let sigma_max: f64 = m / s;

    assert_close(sigma_max, 20.0, 1e-5, "Maximum bending stress");
}

/// Example 5.6 (p. 352): Maximum horizontal transverse shear stress in a rectangular beam.
/// tau_max = 1.5 * V / A at neutral axis y = 0.
/// Shear force V = 60 kN, b = 120 mm, h = 240 mm (A = 28,800 mm²).
/// tau_max = 1.5 * 60,000 / 28,800 = 3.125 MPa.
#[test]
fn test_gere_ex5_6_rectangular_beam_shear_stress() {
    let v: f64 = 60e3; // N
    let b: f64 = 120.0;
    let h: f64 = 240.0;
    let area: f64 = b * h;
    let tau_max: f64 = 1.5 * v / area;

    assert_close(tau_max, 3.125, 1e-4, "Maximum shear stress at neutral axis");
}

/// Example 5.8 (p. 360): Shear flow q = V * Q / I in a built-up wooden beam joined by nails.
/// Flange plate 50 mm x 200 mm, web 50 mm x 200 mm. Total I = 1.8e8 mm^4.
/// Q of top flange about neutral axis: Area = 50 * 200 = 10,000 mm², y_bar = 125 mm.
/// Q = 10000 * 125 = 1.25e6 mm^3.
/// For shear V = 18 kN: q = V * Q / I = 18000 * 1.25e6 / 1.8e8 = 125 N/mm = 125 kN/m.
#[test]
fn test_gere_ex5_8_built_up_beam_shear_flow() {
    let v: f64 = 18e3; // N
    let q_stat: f64 = 1.25e6; // mm^3
    let i_total: f64 = 1.8e8; // mm^4
    let q_flow: f64 = v * q_stat / i_total; // N/mm

    assert_close(q_flow, 125.0, 1e-4, "Horizontal shear flow q");
}

// ================================================================
// Ch. 6: Composite Beams (Transformed Section Method)
// ================================================================

/// Example 6.1 (p. 420): Transformed section of wood beam reinforced with steel plate.
/// Wood: Ew = 10 GPa, bw = 150 mm, hw = 250 mm.
/// Steel bottom plate: Es = 200 GPa, bs = 150 mm, ts = 10 mm. Modular ratio n = Es / Ew = 20.
/// Transformed steel width: b_st = n * bs = 20 * 150 = 3000 mm.
#[test]
fn test_gere_ex6_1_composite_beam_modular_ratio() {
    let ew: f64 = 10.0; // GPa
    let es: f64 = 200.0; // GPa
    let n: f64 = es / ew;
    let bs: f64 = 150.0;
    let b_transformed: f64 = n * bs;

    assert_close(n, 20.0, 1e-5, "Modular ratio n");
    assert_close(b_transformed, 3000.0, 1e-5, "Transformed steel plate width");
}

// ================================================================
// Ch. 7: Plane Stress Transformation & Mohr's Circle
// ================================================================

/// Example 7.1 (p. 480): Principal stresses and max in-plane shear stress.
/// Stress state: sigma_x = 80 MPa, sigma_y = -40 MPa, tau_xy = 35 MPa.
/// Center: sigma_avg = (80 + (-40)) / 2 = 20 MPa.
/// Radius: R = sqrt( ((80 - (-40))/2)^2 + 35^2 ) = sqrt( 60^2 + 35^2 ) = sqrt( 3600 + 1225 ) = sqrt(4825) = 69.46 MPa.
/// Principal stresses: sigma1 = 20 + 69.46 = 89.46 MPa, sigma2 = 20 - 69.46 = -49.46 MPa.
/// Max in-plane shear: tau_max = R = 69.46 MPa.
#[test]
fn test_gere_ex7_1_mohr_circle_principal_stresses() {
    let sx: f64 = 80.0;
    let sy: f64 = -40.0;
    let txy: f64 = 35.0;

    let s_avg: f64 = (sx + sy) / 2.0;
    let r: f64 = (((sx - sy) / 2.0).powi(2) + txy.powi(2)).sqrt();
    let s1: f64 = s_avg + r;
    let s2: f64 = s_avg - r;

    assert_close(s_avg, 20.0, 1e-5, "Center of Mohr circle");
    assert_close(r, 69.46, 0.01, "Radius / max in-plane shear tau_max");
    assert_close(s1, 89.46, 0.01, "Principal stress sigma1");
    assert_close(s2, -49.46, 0.01, "Principal stress sigma2");
}

/// Example 7.2 (p. 488): Principal angles theta_p.
/// tan(2 * theta_p) = 2 * tau_xy / (sigma_x - sigma_y) = 2 * 35 / (80 - (-40)) = 70 / 120 = 0.5833.
/// 2 * theta_p = 30.26 deg -> theta_p1 = 15.13 deg.
#[test]
fn test_gere_ex7_2_principal_orientation() {
    let txy: f64 = 35.0;
    let sx: f64 = 80.0;
    let sy: f64 = -40.0;

    let tan_2tp: f64 = (2.0 * txy) / (sx - sy);
    let tp1_deg: f64 = 0.5 * tan_2tp.atan().to_degrees();

    assert_close(tan_2tp, 70.0 / 120.0, 1e-4, "tan(2 theta_p)");
    assert_close(tp1_deg, 15.13, 0.02, "Principal plane angle");
}

// ================================================================
// Ch. 8: Thin-Walled Pressure Vessels & Combined Loads
// ================================================================

/// Example 8.1 (p. 545): Cylindrical thin-walled pressure vessel stresses.
/// Internal gauge pressure p = 1.2 MPa, inside radius r = 300 mm, wall thickness t = 6 mm.
/// Circumferential (hoop) stress: sigma_h = p * r / t = 1.2 * 300 / 6 = 60.0 MPa.
/// Longitudinal (axial) stress: sigma_l = p * r / (2 * t) = 1.2 * 300 / 12 = 30.0 MPa.
#[test]
fn test_gere_ex8_1_cylindrical_pressure_vessel() {
    let p: f64 = 1.2; // MPa
    let r: f64 = 300.0; // mm
    let t: f64 = 6.0; // mm

    let sigma_hoop: f64 = p * r / t;
    let sigma_long: f64 = p * r / (2.0 * t);

    assert_close(sigma_hoop, 60.0, 1e-5, "Hoop stress sigma_h");
    assert_close(sigma_long, 30.0, 1e-5, "Longitudinal stress sigma_l");
}

/// Example 8.2 (p. 550): Spherical thin-walled pressure vessel stress.
/// sigma = p * r / (2 * t). For same p, r, t: sigma = 30.0 MPa in all membrane directions.
#[test]
fn test_gere_ex8_2_spherical_pressure_vessel() {
    let p: f64 = 1.2;
    let r: f64 = 300.0;
    let t: f64 = 6.0;
    let sigma_sphere: f64 = p * r / (2.0 * t);
    assert_close(sigma_sphere, 30.0, 1e-5, "Spherical membrane stress");
}

// ================================================================
// Ch. 9: Deflections of Determinate Beams
// ================================================================

/// Example 9.1 (p. 610): Cantilever beam with end concentrated load P.
/// Tip deflection delta = P * L^3 / (3 * E * I). Tip slope theta = P * L^2 / (2 * E * I).
/// P = 10 kN, L = 4 m, E = 200 GPa, I = 1e-4 m^4.
/// EI = 200e6 kPa * 1e-4 m^4 = 20,000 kN*m².
/// delta = 10 * 64 / (3 * 20000) = 640 / 60000 = 0.01067 m = 10.67 mm.
#[test]
fn test_gere_ex9_1_cantilever_point_load_deflection() {
    let p: f64 = 10.0;
    let l: f64 = 4.0;
    let ei: f64 = (E * 1e3) * IZ; // 20,000 kN*m²
    let delta_exact: f64 = (p * l.powi(3)) / (3.0 * ei);

    let input = make_beam(8, l, E, A, IZ, "fixed", None, vec![
        SolverLoad::Nodal(SolverNodalLoad { node_id: 9, fx: 0.0, fz: -p, my: 0.0 })
    ]);
    let results = linear::solve_2d(&input).unwrap();
    let tip_disp: f64 = results.displacements.iter().find(|d| d.node_id == 9).unwrap().uz.abs();

    assert_close(tip_disp, delta_exact, 0.01, "Cantilever FE tip deflection matches exact");
}

/// Example 9.2 (p. 614): Cantilever beam with uniform load q.
/// Tip deflection delta = q * L^4 / (8 * E * I). Tip slope theta = q * L^3 / (6 * E * I).
/// q = 5 kN/m, L = 4 m, EI = 20,000 kN*m².
/// delta = 5 * 256 / (8 * 20000) = 1280 / 160000 = 0.0080 m = 8.0 mm.
#[test]
fn test_gere_ex9_2_cantilever_udl_deflection() {
    let q: f64 = 5.0;
    let l: f64 = 4.0;
    let ei: f64 = (E * 1e3) * IZ;
    let delta_exact: f64 = (q * l.powi(4)) / (8.0 * ei);

    assert_close(delta_exact * 1e3, 8.0, 1e-5, "Cantilever UDL exact deflection (mm)");
}

/// Example 9.4 (p. 622): Simply supported beam with center point load P.
/// Midspan deflection delta_c = P * L^3 / (48 * E * I).
/// P = 24 kN, L = 6 m, EI = 20,000 kN*m².
/// delta_c = 24 * 216 / (48 * 20000) = 5184 / 960000 = 0.0054 m = 5.4 mm.
#[test]
fn test_gere_ex9_4_ss_beam_center_load() {
    let p: f64 = 24.0;
    let l: f64 = 6.0;
    let ei: f64 = (E * 1e3) * IZ;
    let delta_exact: f64 = (p * l.powi(3)) / (48.0 * ei);

    let input = make_beam(6, l, E, A, IZ, "pinned", Some("rollerX"), vec![
        SolverLoad::Nodal(SolverNodalLoad { node_id: 4, fx: 0.0, fz: -p, my: 0.0 })
    ]);
    let results = linear::solve_2d(&input).unwrap();
    let mid_disp: f64 = results.displacements.iter().find(|d| d.node_id == 4).unwrap().uz.abs();

    assert_close(mid_disp, delta_exact, 0.01, "SS beam center load deflection");
}

/// Example 9.5 (p. 626): Simply supported beam with full UDL q.
/// Midspan deflection delta_c = 5 * q * L^4 / (384 * E * I).
/// q = 12 kN/m, L = 8 m, EI = 20,000 kN*m².
/// delta_c = 5 * 12 * 4096 / (384 * 20000) = 245760 / 7680000 = 0.0320 m = 32.0 mm.
#[test]
fn test_gere_ex9_5_ss_beam_udl_deflection() {
    let q: f64 = 12.0;
    let l: f64 = 8.0;
    let ei: f64 = (E * 1e3) * IZ;
    let delta_exact: f64 = (5.0 * q * l.powi(4)) / (384.0 * ei);

    assert_close(delta_exact * 1e3, 32.0, 1e-4, "SS beam UDL deflection (mm)");
}

// ================================================================
// Ch. 10: Statically Indeterminate Beams
// ================================================================

/// Example 10.1 (p. 680): Propped cantilever beam with UDL q.
/// Fixed at A (x=0), roller at B (x=L).
/// Reaction at roller: RB = 3 * q * L / 8.
/// Reaction at fixed end: RA = 5 * q * L / 8, MA = q * L^2 / 8.
/// For q = 16 kN/m, L = 6 m:
/// RB = 3 * 16 * 6 / 8 = 36 kN.
/// RA = 5 * 16 * 6 / 8 = 60 kN.
/// MA = 16 * 36 / 8 = 72 kN*m.
#[test]
fn test_gere_ex10_1_propped_cantilever_udl() {
    let q: f64 = 16.0;
    let l: f64 = 6.0;

    let rb_exact: f64 = 3.0 * q * l / 8.0;
    let ra_exact: f64 = 5.0 * q * l / 8.0;
    let ma_exact: f64 = q * l.powi(2) / 8.0;

    let n = 6;
    let mut loads = Vec::new();
    for i in 1..=n {
        loads.push(SolverLoad::Distributed(SolverDistributedLoad {
            element_id: i, q_i: -q, q_j: -q, a: None, b: None,
        }));
    }
    let input = make_beam(n, l, E, A, IZ, "fixed", Some("rollerX"), loads);
    let results = linear::solve_2d(&input).unwrap();

    let ra: f64 = results.reactions.iter().find(|r| r.node_id == 1).unwrap().rz;
    let ma: f64 = results.reactions.iter().find(|r| r.node_id == 1).unwrap().my.abs();
    let rb: f64 = results.reactions.iter().find(|r| r.node_id == n + 1).unwrap().rz;

    assert_close(rb, rb_exact, 0.02, "Roller reaction RB = 3/8 qL");
    assert_close(ra, ra_exact, 0.02, "Fixed support reaction RA = 5/8 qL");
    assert_close(ma, ma_exact, 0.02, "Fixed support moment MA = qL²/8");
}

/// Example 10.3 (p. 692): Fixed-fixed beam with center concentrated load P.
/// End moments: MA = MB = P * L / 8.
/// Center deflection: delta_c = P * L^3 / (192 * E * I).
/// P = 40 kN, L = 8 m, EI = 20,000 kN*m².
/// MA = 40 * 8 / 8 = 40 kN*m.
/// delta_c = 40 * 512 / (192 * 20000) = 20480 / 3840000 = 0.005333 m = 5.333 mm.
#[test]
fn test_gere_ex10_3_fixed_fixed_center_load() {
    let p: f64 = 40.0;
    let l: f64 = 8.0;
    let ei: f64 = (E * 1e3) * IZ;

    let m_end: f64 = p * l / 8.0;
    let delta_center: f64 = (p * l.powi(3)) / (192.0 * ei);

    let input = make_beam(8, l, E, A, IZ, "fixed", Some("fixed"), vec![
        SolverLoad::Nodal(SolverNodalLoad { node_id: 5, fx: 0.0, fz: -p, my: 0.0 })
    ]);
    let results = linear::solve_2d(&input).unwrap();
    let ma: f64 = results.reactions.iter().find(|r| r.node_id == 1).unwrap().my.abs();
    let mid_disp: f64 = results.displacements.iter().find(|d| d.node_id == 5).unwrap().uz.abs();

    assert_close(ma, m_end, 0.02, "Fixed-fixed end moment PL/8");
    assert_close(mid_disp, delta_center, 0.02, "Fixed-fixed center deflection PL³/192EI");
}

/// Example 10.4 (p. 696): Fixed-fixed beam with full UDL q.
/// End moments: MA = MB = q * L^2 / 12. Midspan moment: M_mid = q * L^2 / 24.
#[test]
fn test_gere_ex10_4_fixed_fixed_udl() {
    let q: f64 = 24.0;
    let l: f64 = 6.0;
    let m_end: f64 = q * l.powi(2) / 12.0;
    let m_mid: f64 = q * l.powi(2) / 24.0;

    assert_close(m_end, 72.0, 1e-4, "End moment qL²/12");
    assert_close(m_mid, 36.0, 1e-4, "Midspan moment qL²/24");
}

// ================================================================
// Ch. 11: Columns and Euler Buckling
// ================================================================

/// Example 11.1 (p. 750): Euler buckling load for pinned-pinned column (k = 1.0).
/// Pcr = pi^2 * E * I / L^2.
/// L = 5.0 m, E = 200 GPa, I = 1e-4 m^4 -> EI = 20,000 kN*m².
/// Pcr = pi^2 * 20000 / 25 = 7895.68 kN.
#[test]
fn test_gere_ex11_1_euler_pinned_pinned() {
    let l: f64 = 5.0;
    let ei: f64 = (E * 1e3) * IZ; // 20,000 kN*m²
    let pcr: f64 = std::f64::consts::PI.powi(2) * ei / l.powi(2);

    assert_close(pcr, 7895.68, 0.01, "Euler load Pcr for pinned-pinned column");
}

/// Example 11.2 (p. 756): Euler buckling load for fixed-free column (flagpole, k = 2.0).
/// Pcr = pi^2 * E * I / (4 * L^2). Ratio to pinned-pinned is 0.25.
#[test]
fn test_gere_ex11_2_euler_fixed_free() {
    let l: f64 = 5.0;
    let ei: f64 = (E * 1e3) * IZ;
    let pcr: f64 = std::f64::consts::PI.powi(2) * ei / (4.0 * l.powi(2));
    let ratio: f64 = pcr / (std::f64::consts::PI.powi(2) * ei / l.powi(2));

    assert_close(ratio, 0.25, 1e-5, "Flagpole buckling ratio 1/4");
    assert_close(pcr, 1973.92, 0.01, "Fixed-free buckling capacity");
}

/// Example 11.3 (p. 760): Euler buckling load for fixed-pinned column (k ≈ 0.699).
/// Pcr = 20.19 * E * I / L^2.
#[test]
fn test_gere_ex11_3_euler_fixed_pinned() {
    let l: f64 = 5.0;
    let ei: f64 = (E * 1e3) * IZ;
    let pcr: f64 = 20.19 * ei / l.powi(2);
    let want: f64 = 20.19 * 20000.0 / 25.0;

    assert_close(pcr, want, 1e-4, "Fixed-pinned Euler capacity (20.19 EI/L²)");
}

/// Example 11.4 (p. 764): Euler buckling load for fixed-fixed column (k = 0.5).
/// Pcr = 4 * pi^2 * E * I / L^2. Ratio to pinned-pinned is 4.0.
#[test]
fn test_gere_ex11_4_euler_fixed_fixed() {
    let l: f64 = 5.0;
    let ei: f64 = (E * 1e3) * IZ;
    let pcr: f64 = 4.0 * std::f64::consts::PI.powi(2) * ei / l.powi(2);

    assert_close(pcr, 31582.7, 0.02, "Fixed-fixed Euler capacity (4 pi² EI/L²)");
}

// ================================================================
// Ch. 12: Energy Methods & Castigliano's Theorem
// ================================================================

/// Example 12.1 (p. 820): Strain energy in a cantilever beam under end point load P.
/// U = Integral( M(x)^2 / (2 * E * I) dx, 0..L ). M(x) = P * (L - x).
/// U = P^2 * L^3 / (6 * E * I).
/// Castigliano's theorem: delta = dU / dP = P * L^3 / (3 * E * I) matches elastic deflection!
#[test]
fn test_gere_ex12_1_castigliano_beam_energy() {
    let p: f64 = 15.0;
    let l: f64 = 3.0;
    let ei: f64 = (E * 1e3) * IZ;

    let u: f64 = (p.powi(2) * l.powi(3)) / (6.0 * ei);
    let delta: f64 = (p * l.powi(3)) / (3.0 * ei);

    assert_close(delta, 2.0 * u / p, 1e-6, "Castigliano theorem exact consistency");
}
