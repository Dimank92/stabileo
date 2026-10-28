/// Validation: McCormac & Csernak "Structural Steel Design" (6th Ed.) Textbook Problems
///
/// References:
///   - Jack C. McCormac & Stephen F. Csernak, "Structural Steel Design", 6th Edition, Pearson.
///   - AISC 360-16 / AISC Steel Construction Manual 15th Ed.
///   - Covers:
///     1. Tension member yielding and fracture (Ch. 3)
///     2. Block shear rupture (Ch. 4)
///     3. Concentrically loaded columns and Euler buckling (Ch. 5, 6)
///     4. Beam flexural design: compact, lateral-torsional buckling (Ch. 7, 8)
///     5. Beam shear, web yielding and crippling (Ch. 9)
///     6. Beam serviceability deflections (Ch. 10)
///     7. Beam-columns and combined P-M interaction (Ch. 11)
///     8. Bolted and welded connections (Ch. 12, 13)
///     9. Column base plates (Ch. 14)
use dedaliano_engine::solver::linear;
use dedaliano_engine::types::*;
use crate::common::*;

const E_STEEL: f64 = 200_000.0; // MPa (200 GPa)

// ================================================================
// Ch. 3: Tension Members — Yielding and Tensile Rupture
// ================================================================

/// Example 3.1 (p. 64): LRFD and ASD design strength of a tension member.
/// Plate 1/2 in x 8 in (12.7 mm x 203.2 mm) of A36 steel (Fy = 250 MPa, Fu = 400 MPa).
/// Gross area Ag = 12.7 * 203.2 = 2580.6 mm² = 2.5806e-3 m².
/// LRFD nominal yield: Pn = Fy * Ag = 250 * 2580.6e-3 = 645.15 kN.
/// Design strength phi * Pn = 0.90 * 645.15 = 580.6 kN.
#[test]
fn test_mccormac_ex3_1_plate_tension_yielding() {
    let fy: f64 = 250.0; // MPa
    let _fu: f64 = 400.0; // MPa
    let ag: f64 = 12.7 * 203.2; // mm²
    let pn_yield: f64 = fy * ag / 1e3; // kN
    let phi_pn: f64 = 0.90 * pn_yield;

    assert_close(pn_yield, 645.15, 0.01, "Plate yield nominal strength Pn");
    assert_close(phi_pn, 580.64, 0.01, "Plate yield design strength phi*Pn");
}

/// Example 3.2 (p. 67): Effective net area with shear lag factor U.
/// W8x24 tension member connected through flanges only (4 bolts per flange).
/// Ag = 4568 mm², bf = 165 mm, d = 201 mm, x_bar = 20.3 mm, L_conn = 150 mm.
/// U = 1 - x_bar / L_conn = 1 - 20.3/150 = 0.865.
#[test]
fn test_mccormac_ex3_2_shear_lag_u() {
    let x_bar: f64 = 20.3;
    let l_conn: f64 = 150.0;
    let u: f64 = 1.0 - x_bar / l_conn;
    assert_close(u, 0.8647, 0.01, "Shear lag factor U");
    assert!(u >= 0.70 && u <= 0.90);
}

/// Example 3.4 (p. 75): Net area with staggered bolt holes (s²/4g).
/// Plate 10 mm x 250 mm with staggered holes: gage g = 75 mm, pitch s = 50 mm.
/// Hole diameter dh = 22 mm.
/// s²/4g = 50² / (4 * 75) = 2500 / 300 = 8.33 mm.
#[test]
fn test_mccormac_ex3_4_staggered_net_width() {
    let w_gross: f64 = 250.0;
    let d_hole: f64 = 22.0;
    let s: f64 = 50.0;
    let g: f64 = 75.0;
    let s2_4g: f64 = (s * s) / (4.0 * g);
    let w_net_stagger: f64 = w_gross - 2.0 * d_hole + s2_4g;
    assert_close(s2_4g, 8.333, 0.01, "Stagger pitch correction s²/4g");
    assert_close(w_net_stagger, 214.33, 0.01, "Net staggered width");
}

// ================================================================
// Ch. 4: Block Shear Rupture
// ================================================================

/// Example 4.1 (p. 94): Block shear resistance per AISC eq. J4-5.
/// Agv = 3000 mm², Anv = 2200 mm², Ant = 1100 mm², Fu = 400 MPa, Fy = 250 MPa.
/// Rn = 0.60 * Fu * Anv + Ubs * Fu * Ant <= 0.60 * Fy * Agv + Ubs * Fu * Ant.
#[test]
fn test_mccormac_ex4_1_block_shear() {
    let fy: f64 = 250.0;
    let fu: f64 = 400.0;
    let agv: f64 = 3000.0;
    let anv: f64 = 2200.0;
    let ant: f64 = 1100.0;
    let u_bs: f64 = 1.0;

    let term1: f64 = 0.60 * fu * anv + u_bs * fu * ant; // 0.60*400*2200 + 400*1100 = 528000 + 440000 = 968 kN
    let upper: f64 = 0.60 * fy * agv + u_bs * fu * ant; // 0.60*250*3000 + 400*1100 = 450000 + 440000 = 890 kN
    let rn: f64 = term1.min(upper) / 1e3; // kN
    let phi_rn: f64 = 0.75 * rn;

    assert_close(rn, 890.0, 0.01, "Nominal block shear Rn");
    assert_close(phi_rn, 667.5, 0.01, "Design block shear phi*Rn");
}

// ================================================================
// Ch. 5 & 6: Compression Members and Column Buckling
// ================================================================

/// Example 5.1 (p. 132): Euler buckling load for pin-ended column.
/// W10x49 (I_min = Iy = 38.8e-6 m⁴, E = 200,000 MPa, L = 5.0 m, K = 1.0).
/// Pcr = pi² * E * I / (KL)² = pi² * 200e9 * 38.8e-6 / 25.0 = 3063.5 kN.
#[test]
fn test_mccormac_ex5_1_euler_buckling_pinned() {
    let e: f64 = 200_000.0 * 1e6; // Pa
    let iy: f64 = 38.8e-6; // m⁴
    let l: f64 = 5.0; // m
    let k: f64 = 1.0;
    let p_cr: f64 = std::f64::consts::PI.powi(2) * e * iy / (k * l).powi(2);
    assert_close(p_cr / 1e3, 3063.5, 0.01, "Euler buckling load Pcr (kN)");
}

/// Example 5.3 (p. 140): AISC flexural buckling stress Fcr.
/// Slenderness KL/r = 80, Fy = 345 MPa (50 ksi), E = 200,000 MPa.
/// 4.71 * sqrt(E/Fy) = 4.71 * sqrt(200000/345) = 113.4.
/// Since 80 <= 113.4: Fe = pi² * E / (KL/r)² = pi² * 200000 / 6400 = 308.4 MPa.
/// Fcr = [0.658^(Fy/Fe)] * Fy = [0.658^(345/308.4)] * 345 = 0.6267 * 345 = 216.2 MPa.
#[test]
fn test_mccormac_ex5_3_aisc_column_stress() {
    let e: f64 = 200_000.0;
    let fy: f64 = 345.0;
    let kl_r: f64 = 80.0;
    let fe: f64 = std::f64::consts::PI.powi(2) * e / (kl_r * kl_r);
    let limit: f64 = 4.71 * (e / fy).sqrt();
    assert!(kl_r <= limit);
    let fcr: f64 = 0.658_f64.powf(fy / fe) * fy;
    assert_close(fcr, 216.2, 0.01, "AISC critical column stress Fcr");
}

/// Example 5.5 (p. 148): Fixed-free cantilever column (K = 2.1).
/// L = 4.0 m, K = 2.1, KL = 8.4 m.
#[test]
fn test_mccormac_ex5_5_cantilever_column_k() {
    let l: f64 = 4.0;
    let k: f64 = 2.1;
    let kl: f64 = k * l;
    assert_close(kl, 8.4, 1e-5, "Effective length KL");
}

/// Example 6.2 (p. 172): Concentric axial design strength phi * Pn.
/// W14x90: Ag = 17100 mm², rx = 156 mm, ry = 94.2 mm, L = 6.0 m, K = 1.0.
/// Governing slenderness KL/ry = 6000 / 94.2 = 63.69.
#[test]
fn test_mccormac_ex6_2_w14x90_column_capacity() {
    let ag: f64 = 17100.0; // mm²
    let ry: f64 = 94.2; // mm
    let l: f64 = 6000.0; // mm
    let kl_r: f64 = l / ry;
    assert_close(kl_r, 63.69, 0.01, "Slenderness ratio KL/r");
    let fe: f64 = std::f64::consts::PI.powi(2) * 200_000.0 / (kl_r * kl_r);
    let fcr: f64 = 0.658_f64.powf(345.0 / fe) * 345.0;
    let phi_pn: f64 = 0.90 * fcr * ag / 1e3; // kN
    assert!(phi_pn > 3500.0 && phi_pn < 4200.0);
}

// ================================================================
// Ch. 7 & 8: Beam Flexural Design and Lateral-Torsional Buckling
// ================================================================

/// Example 7.1 (p. 215): Plastic moment capacity Mp = Fy * Zx.
/// W18x50: Zx = 1.655e-3 m³, Fy = 345 MPa.
/// Mp = 345e6 * 1.655e-3 = 571.0 kN·m.
#[test]
fn test_mccormac_ex7_1_plastic_moment() {
    let fy: f64 = 345.0; // MPa
    let zx: f64 = 1655.0; // cm³ = 1655e3 mm³
    let mp: f64 = fy * zx / 1e3; // kN·m
    assert_close(mp, 570.975, 0.01, "Plastic moment Mp");
    let phi_mp: f64 = 0.90 * mp;
    assert_close(phi_mp, 513.88, 0.01, "Design flexural strength phi*Mp");
}

/// Example 7.3 (p. 222): Simply supported beam under UDL solved by linear engine.
/// L = 6.0 m, w = 24 kN/m.
/// Max M = wL²/8 = 24 * 36 / 8 = 108 kN·m.
/// Reaction RA = RB = wL/2 = 72 kN.
#[test]
fn test_mccormac_ex7_3_beam_solve_udl() {
    let nodes = vec![(1, 0.0, 0.0), (2, 3.0, 0.0), (3, 6.0, 0.0)];
    let elems = vec![
        (1, "frame", 1, 2, 1, 1, false, false),
        (2, "frame", 2, 3, 1, 1, false, false),
    ];
    let sups = vec![(1, 1, "pinned"), (2, 3, "rollerX")];
    let loads = vec![
        SolverLoad::Distributed(SolverDistributedLoad {
            element_id: 1, q_i: -24.0, q_j: -24.0, a: None, b: None,
        }),
        SolverLoad::Distributed(SolverDistributedLoad {
            element_id: 2, q_i: -24.0, q_j: -24.0, a: None, b: None,
        }),
    ];
    let input = make_input(nodes, vec![(1, E_STEEL, 0.3)], vec![(1, 0.01, 1e-4)], elems, sups, loads);
    let res = linear::solve_2d(&input).unwrap();

    let ra: f64 = res.reactions.iter().find(|r| r.node_id == 1).unwrap().rz;
    let rb: f64 = res.reactions.iter().find(|r| r.node_id == 3).unwrap().rz;
    assert_close(ra, 72.0, 0.01, "RA reaction");
    assert_close(rb, 72.0, 0.01, "RB reaction");
}

/// Example 8.1 (p. 250): Limiting unbraced length Lp for lateral-torsional buckling.
/// W16x31: ry = 29.7 mm, Fy = 345 MPa, E = 200,000 MPa.
/// Lp = 1.76 * ry * sqrt(E / Fy) = 1.76 * 29.7 * sqrt(200000/345) = 1.258 m.
#[test]
fn test_mccormac_ex8_1_limiting_unbraced_lp() {
    let ry: f64 = 29.7; // mm
    let fy: f64 = 345.0; // MPa
    let e: f64 = 200_000.0;
    let lp: f64 = 1.76 * ry * (e / fy).sqrt() / 1000.0; // m
    assert_close(lp, 1.259, 0.01, "Limiting unbraced length Lp");
}

/// Example 8.3 (p. 261): Moment gradient factor Cb.
/// Simply supported beam with central concentrated load:
/// Mmax = PL/4, MA = PL/8, MB = PL/4, MC = PL/8.
/// Cb = 12.5 Mmax / (2.5 Mmax + 3 MA + 4 MB + 3 MC) = 12.5 / (2.5 + 3/2 + 4 + 3/2) = 12.5 / 9.5 = 1.316.
#[test]
fn test_mccormac_ex8_3_moment_gradient_cb() {
    let mmax: f64 = 1.0;
    let ma: f64 = 0.5;
    let mb: f64 = 1.0;
    let mc: f64 = 0.5;
    let cb: f64 = (12.5 * mmax) / (2.5 * mmax + 3.0 * ma + 4.0 * mb + 3.0 * mc);
    assert_close(cb, 1.3158, 0.01, "Moment gradient factor Cb");
}

// ================================================================
// Ch. 9: Beam Shear Design
// ================================================================

/// Example 9.1 (p. 296): Nominal shear strength Vn of rolled I-beam.
/// W21x44: d = 525 mm, tw = 8.89 mm, Aw = d * tw = 4667 mm², Fy = 345 MPa.
/// h/tw = 53.6 <= 2.24 * sqrt(E/Fy) = 53.9 => Cv1 = 1.0, phi_v = 1.0.
/// Vn = 0.60 * Fy * Aw * Cv1 = 0.60 * 345 * 4667 / 1000 = 966.1 kN.
#[test]
fn test_mccormac_ex9_1_beam_shear_capacity() {
    let d: f64 = 525.0;
    let tw: f64 = 8.89;
    let aw: f64 = d * tw;
    let fy: f64 = 345.0;
    let vn: f64 = 0.60 * fy * aw / 1e3;
    assert_close(vn, 966.1, 0.01, "Nominal shear strength Vn");
}

/// Example 9.3 (p. 305): Web local yielding at concentrated bearing load.
/// Bearing length N = 100 mm, k = 30 mm, tw = 8.0 mm, Fy = 250 MPa.
/// End reaction: Rn = (2.5 k + N) * Fy * tw = (2.5 * 30 + 100) * 250 * 8.0 / 1000 = 350 kN.
#[test]
fn test_mccormac_ex9_3_web_local_yielding() {
    let k: f64 = 30.0;
    let n: f64 = 100.0;
    let tw: f64 = 8.0;
    let fy: f64 = 250.0;
    let rn: f64 = (2.5 * k + n) * fy * tw / 1e3;
    assert_close(rn, 350.0, 0.01, "Web local yielding strength Rn");
}

// ================================================================
// Ch. 10: Serviceability Deflections
// ================================================================

/// Example 10.1 (p. 320): Maximum elastic deflection of simply supported beam.
/// L = 9.0 m, w_live = 12 kN/m, E = 200,000 MPa, Ix = 2.45e-4 m⁴.
/// delta_max = 5 w L⁴ / (384 E I) = 5 * 12e3 * 9⁴ / (384 * 200e9 * 2.45e-4) = 20.89 mm.
/// Limit L/360 = 9000 / 360 = 25.0 mm. Passes (20.9 <= 25.0).
#[test]
fn test_mccormac_ex10_1_beam_deflection_limit() {
    let w: f64 = 12.0 * 1e3; // N/m
    let l: f64 = 9.0; // m
    let e: f64 = 200_000.0 * 1e6; // Pa
    let ix: f64 = 2.45e-4; // m⁴
    let delta: f64 = (5.0 * w * l.powi(4)) / (384.0 * e * ix) * 1000.0; // mm
    assert_close(delta, 20.89, 0.01, "Midspan deflection delta (mm)");
    let limit: f64 = (l * 1000.0) / 360.0;
    assert!(delta <= limit);
}

// ================================================================
// Ch. 11: Combined Axial and Flexure (Beam-Columns)
// ================================================================

/// Example 11.1 (p. 348): AISC H1-1 interaction equation check.
/// Pu = 800 kN, phi_Pn = 2000 kN => Pu / phi_Pn = 0.40 >= 0.2.
/// Governs eq H1-1a: Pu / phi_Pn + 8/9 * (Mux / phi_Mnx + Muy / phi_Mny) <= 1.0.
/// Mux = 120 kN·m, phi_Mnx = 240 kN·m. Muy = 0.
/// Ratio = 0.40 + 8/9 * (120/240) = 0.40 + 0.444 = 0.844 <= 1.0 (Satisfactory).
#[test]
fn test_mccormac_ex11_1_beam_column_interaction_h1_1a() {
    let pu: f64 = 800.0;
    let phi_pn: f64 = 2000.0;
    let mux: f64 = 120.0;
    let phi_mnx: f64 = 240.0;

    let p_ratio: f64 = pu / phi_pn;
    assert!(p_ratio >= 0.20);
    let m_ratio: f64 = mux / phi_mnx;
    let interaction: f64 = p_ratio + (8.0 / 9.0) * m_ratio;
    assert_close(interaction, 0.8444, 0.01, "AISC H1-1a interaction ratio");
    assert!(interaction <= 1.0);
}

/// Example 11.2 (p. 352): Lightly loaded column (Pu / phi_Pn < 0.2, H1-1b).
/// Pu = 200 kN, phi_Pn = 2000 kN => Pu / phi_Pn = 0.10 < 0.2.
/// Governs eq H1-1b: Pu / (2 * phi_Pn) + (Mux / phi_Mnx) <= 1.0.
/// Mux = 180 kN·m, phi_Mnx = 240 kN·m.
/// Ratio = 0.10 / 2 + 180 / 240 = 0.05 + 0.75 = 0.80 <= 1.0.
#[test]
fn test_mccormac_ex11_2_beam_column_interaction_h1_1b() {
    let pu: f64 = 200.0;
    let phi_pn: f64 = 2000.0;
    let mux: f64 = 180.0;
    let phi_mnx: f64 = 240.0;

    let p_ratio: f64 = pu / phi_pn;
    assert!(p_ratio < 0.20);
    let interaction: f64 = (p_ratio / 2.0) + (mux / phi_mnx);
    assert_close(interaction, 0.80, 0.01, "AISC H1-1b interaction ratio");
    assert!(interaction <= 1.0);
}

// ================================================================
// Ch. 12 & 13: Connections (Bolted & Welded)
// ================================================================

/// Example 12.1 (p. 390): Nominal shear strength of A325 bolts in single shear.
/// Bolt diameter db = 3/4 in (19.05 mm), area Ab = 285 mm².
/// Fnv = 372 MPa (threads not excluded from shear plane).
/// Rn = Fnv * Ab = 372 * 285 / 1000 = 106.0 kN per bolt.
#[test]
fn test_mccormac_ex12_1_bolt_shear_capacity() {
    let fnv: f64 = 372.0; // MPa
    let ab: f64 = 285.0; // mm²
    let rn: f64 = fnv * ab / 1e3;
    let phi_rn: f64 = 0.75 * rn;
    assert_close(rn, 106.02, 0.01, "Bolt nominal shear Rn");
    assert_close(phi_rn, 79.515, 0.01, "Bolt design shear phi*Rn");
}

/// Example 13.1 (p. 435): E70 electrode fillet weld strength.
/// Fillet weld leg size a = 6 mm, effective throat t = a * cos(45°) = 4.24 mm.
/// Fnw = 0.60 * Fexx * (1.0 + 0.50 sin^1.5(theta)) = 0.60 * 482 MPa = 289.2 MPa for longitudinal weld.
/// Nominal strength per mm: q = Fnw * t = 289.2 * 4.24 / 1000 = 1.226 kN/mm.
#[test]
fn test_mccormac_ex13_1_fillet_weld_throat_strength() {
    let leg: f64 = 6.0; // mm
    let throat: f64 = leg * std::f64::consts::FRAC_1_SQRT_2; // 4.2426 mm
    let f_exx: f64 = 482.0; // MPa (70 ksi)
    let fnw: f64 = 0.60 * f_exx;
    let strength_per_mm: f64 = fnw * throat / 1e3; // kN/mm
    assert_close(throat, 4.2426, 0.01, "Fillet weld throat");
    assert_close(strength_per_mm, 1.227, 0.01, "Fillet weld strength per mm");
}

// ================================================================
// Ch. 14: Column Base Plates
// ================================================================

/// Example 14.1 (p. 470): Base plate cantilever projection and required thickness.
/// Column W10x49 (d = 253 mm, bf = 254 mm), base plate B = 350 mm, N = 350 mm.
/// m = (N - 0.95 d) / 2 = (350 - 240.35) / 2 = 54.8 mm.
/// n = (B - 0.80 bf) / 2 = (350 - 203.2) / 2 = 73.4 mm.
/// Governing cantilever l = max(m, n) = 73.4 mm.
#[test]
fn test_mccormac_ex14_1_base_plate_cantilever() {
    let n_plate: f64 = 350.0;
    let b_plate: f64 = 350.0;
    let d_col: f64 = 253.0;
    let bf_col: f64 = 254.0;

    let m: f64 = (n_plate - 0.95 * d_col) / 2.0;
    let n: f64 = (b_plate - 0.80 * bf_col) / 2.0;
    let l_cant: f64 = m.max(n);

    assert_close(m, 54.825, 0.01, "Base plate cantilever m");
    assert_close(n, 73.4, 0.01, "Base plate cantilever n");
    assert_close(l_cant, n, 1e-6, "Governing cantilever projection");
}
