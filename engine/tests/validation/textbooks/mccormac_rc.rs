/// Validation: McCormac & Brown "Design of Reinforced Concrete" (10th Ed.) Textbook Problems
///
/// References:
///   - Jack C. McCormac & Russell H. Brown, "Design of Reinforced Concrete", 10th Edition, Wiley.
///   - ACI 318-14 / ACI 318-19 American Concrete Institute Building Code Requirements.
///   - Covers:
///     1. Flexural analysis of uncracked & cracked sections (Ch. 2)
///     2. Ultimate strength flexure and Whitney stress block (Ch. 3)
///     3. Design of rectangular beams & one-way slabs (Ch. 4)
///     4. T-beams and doubly reinforced beams (Ch. 5)
///     5. Serviceability: deflections & cracked inertia (Ch. 6)
///     6. Development length of reinforcement (Ch. 7)
///     7. Shear and diagonal tension in beams & stirrups (Ch. 8)
///     8. Axially loaded columns and P-M interaction (Ch. 9)
///     9. Slender columns and moment magnification (Ch. 10)
///     10. Footings and punching shear (Ch. 12)
///     11. Cantilever retaining walls (Ch. 14)
use crate::common::*;

// Concrete and steel basic properties
const E_STEEL: f64 = 200_000.0; // MPa

// ================================================================
// Ch. 2: Flexural Analysis of Concrete Beams (Cracked & Uncracked)
// ================================================================

/// Example 2.1 (p. 34): Cracking moment Mcr of a plain or uncracked rectangular beam.
/// Beam 12 in x 18 in (300 mm x 450 mm), f'c = 4000 psi (27.6 MPa).
/// Modulus of rupture fr = 7.5 * lambda * sqrt(f'c) = 0.62 * sqrt(27.6) = 3.257 MPa.
/// Ig = b * h^3 / 12 = 300 * 450^3 / 12 = 2.278e9 mm^4.
/// yt = h / 2 = 225 mm.
/// Mcr = fr * Ig / yt = 3.257 * 2.278e9 / 225 = 33.0 kN*m.
#[test]
fn test_mccormac_rc_ex2_1_cracking_moment() {
    let fc_prime: f64 = 27.6; // MPa
    let b: f64 = 300.0; // mm
    let h: f64 = 450.0; // mm
    let fr: f64 = 0.62 * fc_prime.sqrt(); // MPa
    let ig: f64 = b * h.powi(3) / 12.0; // mm^4
    let yt: f64 = h / 2.0;
    let mcr: f64 = (fr * ig / yt) / 1e6; // kN*m

    assert_close(fr, 3.257, 0.02, "Modulus of rupture fr");
    assert_close(mcr, 33.0, 0.02, "Cracking moment Mcr");
}

/// Example 2.2 (p. 39): Transformed cracked section neutral axis depth kd.
/// Beam b = 300 mm, d = 500 mm, As = 3 #9 bars (1500 mm²), n = Es/Ec = 8.0.
/// Equilibrium of transformed area: (b * (kd)^2) / 2 = n * As * (d - kd).
/// Quadratic: 0.5 * b * (kd)^2 + n*As*(kd) - n*As*d = 0.
#[test]
fn test_mccormac_rc_ex2_2_cracked_neutral_axis() {
    let b: f64 = 300.0; // mm
    let d: f64 = 500.0; // mm
    let as_steel: f64 = 1500.0; // mm²
    let n: f64 = 8.0;

    let a_quad: f64 = 0.5 * b;
    let b_quad: f64 = n * as_steel;
    let c_quad: f64 = -n * as_steel * d;
    let kd: f64 = (-b_quad + (b_quad * b_quad - 4.0 * a_quad * c_quad).sqrt()) / (2.0 * a_quad);

    // kd ≈ 163.6 mm, kd/d ≈ 0.327
    assert_close(kd, 163.6, 0.02, "Cracked neutral axis depth kd");
}

/// Example 2.3 (p. 42): Cracked moment of inertia Icr.
/// Icr = b * (kd)^3 / 3 + n * As * (d - kd)^2.
#[test]
fn test_mccormac_rc_ex2_3_cracked_moment_of_inertia() {
    let b: f64 = 300.0;
    let d: f64 = 500.0;
    let as_steel: f64 = 1500.0;
    let n: f64 = 8.0;
    let kd: f64 = 163.6;

    let icr: f64 = (b * kd.powi(3) / 3.0) + n * as_steel * (d - kd).powi(2);
    // icr ≈ 1.80e9 mm^4
    assert_close(icr / 1e9, 1.80, 0.03, "Cracked moment of inertia Icr");
}

// ================================================================
// Ch. 3: Ultimate Strength Flexure & Whitney Stress Block
// ================================================================

/// Example 3.1 (p. 68): Depth of Whitney stress block 'a' and nominal strength Mn.
/// Beam b = 300 mm, d = 500 mm, As = 1800 mm², fy = 420 MPa, f'c = 28 MPa.
/// a = (As * fy) / (0.85 * f'c * b) = (1800 * 420) / (0.85 * 28 * 300) = 105.88 mm.
/// Mn = As * fy * (d - a / 2) = 1800 * 420 * (500 - 52.94) = 337.98 kN*m.
#[test]
fn test_mccormac_rc_ex3_1_whitney_stress_block() {
    let b: f64 = 300.0;
    let d: f64 = 500.0;
    let as_steel: f64 = 1800.0;
    let fy: f64 = 420.0;
    let fc: f64 = 28.0;

    let a: f64 = (as_steel * fy) / (0.85 * fc * b);
    let mn: f64 = (as_steel * fy * (d - a / 2.0)) / 1e6; // kN*m

    assert_close(a, 105.88, 0.02, "Whitney stress block depth a");
    assert_close(mn, 338.0, 0.02, "Nominal flexural strength Mn");
}

/// Example 3.2 (p. 73): Check tension-controlled strain limit epsilon_t >= 0.005.
/// For f'c = 28 MPa, beta1 = 0.85. c = a / beta1 = 105.88 / 0.85 = 124.57 mm.
/// epsilon_t = 0.003 * (d - c) / c = 0.003 * (500 - 124.57) / 124.57 = 0.00904.
/// Because epsilon_t >= 0.005, section is tension-controlled: phi = 0.90.
/// Design strength phi * Mn = 0.90 * 338.0 = 304.2 kN*m.
#[test]
fn test_mccormac_rc_ex3_2_phi_factor_tension_controlled() {
    let d: f64 = 500.0;
    let a: f64 = 105.88;
    let beta1: f64 = 0.85;
    let c: f64 = a / beta1;
    let eps_t: f64 = 0.003 * (d - c) / c;
    let phi: f64 = if eps_t >= 0.005 {
        0.90
    } else if eps_t <= 0.002 {
        0.65
    } else {
        0.65 + 0.25 * (eps_t - 0.002) / 0.003
    };

    assert_close(c, 124.57, 0.02, "Neutral axis depth c");
    assert!(eps_t > 0.005, "Tension controlled");
    assert_close(phi, 0.90, 0.001, "Strength reduction factor phi");
}

/// Example 3.4 (p. 80): Balanced reinforcement ratio rho_b.
/// rho_b = (0.85 * beta1 * f'c / fy) * [600 / (600 + fy)].
/// For f'c = 28 MPa, beta1 = 0.85, fy = 420 MPa:
/// rho_b = (0.85 * 0.85 * 28 / 420) * (600 / 1020) = 0.04817 * 0.5882 = 0.0283.
#[test]
fn test_mccormac_rc_ex3_4_balanced_steel_ratio() {
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;
    let beta1: f64 = 0.85;
    let rho_b: f64 = (0.85 * beta1 * fc / fy) * (600.0 / (600.0 + fy));

    assert_close(rho_b, 0.0283, 0.02, "Balanced reinforcement ratio rho_b");
}

// ================================================================
// Ch. 4: Design of Rectangular Beams & Minimum Steel
// ================================================================

/// Example 4.1 (p. 96): ACI minimum flexural steel As,min.
/// As,min = max(0.25 * sqrt(f'c) / fy * b * d, 1.4 / fy * b * d).
/// For f'c = 28 MPa, fy = 420 MPa, b = 300 mm, d = 500 mm:
/// Term 1: 0.25 * sqrt(28) / 420 * 300 * 500 = 0.25 * 5.2915 / 420 * 150000 = 472.5 mm².
/// Term 2: 1.4 / 420 * 300 * 500 = 500.0 mm².
/// As,min = 500.0 mm².
#[test]
fn test_mccormac_rc_ex4_1_as_min() {
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;
    let b: f64 = 300.0;
    let d: f64 = 500.0;

    let term1: f64 = (0.25 * fc.sqrt() / fy) * b * d;
    let term2: f64 = (1.4 / fy) * b * d;
    let as_min: f64 = term1.max(term2);

    assert_close(as_min, 500.0, 0.01, "Minimum flexural reinforcement As,min");
}

/// Example 4.3 (p. 104): One-way slab minimum shrinkage and temperature steel.
/// Grade 420 steel: As,temp = 0.0018 * b * h.
/// Slab thickness h = 150 mm, strip width b = 1000 mm.
/// As,temp = 0.0018 * 1000 * 150 = 270 mm²/m.
#[test]
fn test_mccormac_rc_ex4_3_slab_temperature_steel() {
    let h: f64 = 150.0;
    let b: f64 = 1000.0;
    let as_temp: f64 = 0.0018 * b * h;
    assert_close(as_temp, 270.0, 0.01, "Temperature reinforcement As,temp");
}

// ================================================================
// Ch. 5: T-Beams and Doubly Reinforced Beams
// ================================================================

/// Example 5.1 (p. 125): Effective flange width beff for an interior T-beam per ACI 318.
/// Beam span L = 6000 mm, web width bw = 300 mm, slab thickness hf = 100 mm, clear spacing sw = 2000 mm.
/// beff = min(L / 4, bw + 16 * hf, bw + sw)
/// L/4 = 1500 mm, bw + 16*hf = 300 + 1600 = 1900 mm, bw + sw = 2300 mm.
/// beff = 1500 mm.
#[test]
fn test_mccormac_rc_ex5_1_tbeam_effective_flange() {
    let span: f64 = 6000.0;
    let bw: f64 = 300.0;
    let hf: f64 = 100.0;
    let sw: f64 = 2000.0;

    let limit1: f64 = span / 4.0;
    let limit2: f64 = bw + 16.0 * hf;
    let limit3: f64 = bw + sw;
    let beff: f64 = limit1.min(limit2).min(limit3);

    assert_close(beff, 1500.0, 0.01, "T-beam effective flange width beff");
}

/// Example 5.2 (p. 130): True T-beam nominal moment strength Mn.
/// beff = 1200 mm, bw = 300 mm, hf = 100 mm, d = 550 mm, As = 4000 mm², f'c = 28 MPa, fy = 420 MPa.
/// Flange capacity: C_f = 0.85 * f'c * (beff - bw) * hf = 0.85 * 28 * 900 * 100 = 2142 kN.
/// Steel force: T = As * fy = 4000 * 420 = 1680 kN.
/// Because T < C_f, stress block falls within flange (a < hf), behaves as rectangular beam of width beff!
/// a = T / (0.85 * f'c * beff) = 1680000 / (0.85 * 28 * 1200) = 58.82 mm (< 100 mm).
/// Mn = T * (d - a/2) = 1680 * (550 - 29.41) = 874.59 kN*m.
#[test]
fn test_mccormac_rc_ex5_2_tbeam_flange_behavior() {
    let beff: f64 = 1200.0;
    let hf: f64 = 100.0;
    let d: f64 = 550.0;
    let as_steel: f64 = 4000.0;
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;

    let _t: f64 = as_steel * fy / 1e3; // kN
    let a: f64 = (as_steel * fy) / (0.85 * fc * beff);
    assert!(a < hf, "Stress block lies within flange");

    let mn: f64 = (as_steel * fy * (d - a / 2.0)) / 1e6; // kN*m
    assert_close(a, 58.82, 0.02, "Equivalent block depth a");
    assert_close(mn, 874.6, 0.02, "T-beam nominal moment Mn");
}

/// Example 5.5 (p. 142): Doubly reinforced beam compression steel yield check.
/// b = 300 mm, d = 500 mm, d' = 60 mm, As = 3000 mm², As' = 1000 mm², fy = 420 MPa, f'c = 28 MPa.
/// Assume compression steel yields (fs' = fy):
/// a = (As - As') * fy / (0.85 * f'c * b) = (2000 * 420) / (0.85 * 28 * 300) = 117.65 mm.
/// c = a / 0.85 = 138.41 mm.
/// Strain in compression steel: eps_s' = 0.003 * (c - d') / c = 0.003 * 78.41 / 138.41 = 0.00170.
/// Yield strain eps_y = 420 / 200000 = 0.00210.
/// Here eps_s' < eps_y, compression steel does NOT yield! fs' = Es * eps_s' = 340 MPa.
#[test]
fn test_mccormac_rc_ex5_5_doubly_reinforced_steel_yield() {
    let d_prime: f64 = 60.0;
    let c: f64 = 138.41;
    let eps_s_prime: f64 = 0.003 * (c - d_prime) / c;
    let eps_y: f64 = 420.0 / E_STEEL;
    assert!(eps_s_prime < eps_y, "Compression steel does not yield");
    let fs_prime: f64 = E_STEEL * eps_s_prime;
    assert_close(fs_prime, 340.0, 0.03, "Compression steel stress fs'");
}

// ================================================================
// Ch. 6: Serviceability — Branson Effective Inertia & Deflections
// ================================================================

/// Example 6.1 (p. 165): Branson effective moment of inertia Ie per ACI 318.
/// Ig = 3.6e9 mm^4, Icr = 1.2e9 mm^4, Mcr = 35 kN*m, applied service moment Ma = 50 kN*m.
/// Ie = (Mcr / Ma)^3 * Ig + [1 - (Mcr / Ma)^3] * Icr <= Ig.
/// (Mcr / Ma)^3 = (35 / 50)^3 = 0.70^3 = 0.343.
/// Ie = 0.343 * 3.6e9 + (1 - 0.343) * 1.2e9 = 1.235e9 + 0.788e9 = 2.023e9 mm^4.
#[test]
fn test_mccormac_rc_ex6_1_branson_effective_inertia() {
    let ig: f64 = 3.6e9;
    let icr: f64 = 1.2e9;
    let mcr: f64 = 35.0;
    let ma: f64 = 50.0;

    let ratio: f64 = mcr / ma;
    let ratio3: f64 = ratio.powi(3);
    let ie: f64 = (ratio3 * ig + (1.0 - ratio3) * icr).min(ig);

    assert_close(ie / 1e9, 2.023, 0.02, "Branson effective inertia Ie");
}

/// Example 6.3 (p. 172): Long-term deflection multiplier lambda_Delta.
/// lambda_Delta = xi / (1 + 50 * rho').
/// For duration > 5 years, xi = 2.0. With no compression steel (rho' = 0), lambda_Delta = 2.0.
/// With compression steel rho' = 0.01: lambda_Delta = 2.0 / (1 + 50 * 0.01) = 2.0 / 1.5 = 1.333.
#[test]
fn test_mccormac_rc_ex6_3_long_term_deflection_multiplier() {
    let xi: f64 = 2.0; // > 5 years
    let rho_prime: f64 = 0.01;
    let lambda_delta: f64 = xi / (1.0 + 50.0 * rho_prime);
    assert_close(lambda_delta, 1.333, 0.02, "Long-term deflection factor");
}

// ================================================================
// Ch. 7: Reinforcement Development Length
// ================================================================

/// Example 7.1 (p. 192): Tension development length ld for uncoated straight #8 (25 mm) bar.
/// ld = (fy / (1.1 * lambda * sqrt(f'c))) * (psi_t * psi_e * psi_s / ((cb + Ktr)/db)) * db.
/// Simplified ACI: ld / db = (fy * psi_t * psi_e) / (2.1 * lambda * sqrt(f'c)) for bottom bars.
/// For f'c = 28 MPa, fy = 420 MPa, db = 25.4 mm, normal weight (lambda = 1.0), psi_t = 1.0, psi_e = 1.0:
/// ld / db = 420 / (2.1 * 1.0 * sqrt(28)) = 420 / (2.1 * 5.2915) = 420 / 11.112 = 37.8.
/// ld = 37.8 * 25.4 = 960 mm.
#[test]
fn test_mccormac_rc_ex7_1_straight_bar_development_length() {
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;
    let db: f64 = 25.4;
    let ratio: f64 = fy / (2.1 * fc.sqrt());
    let ld: f64 = ratio * db;

    assert_close(ratio, 37.8, 0.02, "Development length ld/db ratio");
    assert_close(ld, 960.0, 0.02, "Tension development length ld");
}

/// Example 7.4 (p. 204): Standard 90-degree hook development length ldh.
/// ldh = (0.24 * fy * psi_e * psi_c * psi_r / (lambda * sqrt(f'c))) * db >= max(8 * db, 150 mm).
/// For fy = 420 MPa, f'c = 28 MPa, db = 20 mm, all psi factors = 1.0:
/// ldh = (0.24 * 420 / sqrt(28)) * 20 = (100.8 / 5.2915) * 20 = 19.05 * 20 = 381 mm.
#[test]
fn test_mccormac_rc_ex7_4_hook_development_length() {
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;
    let db: f64 = 20.0;
    let ldh_calc: f64 = (0.24 * fy / fc.sqrt()) * db;
    let ldh: f64 = ldh_calc.max(8.0 * db).max(150.0);

    assert_close(ldh, 381.0, 0.02, "Hook development length ldh");
}

// ================================================================
// Ch. 8: Shear and Diagonal Tension
// ================================================================

/// Example 8.1 (p. 218): Nominal concrete shear strength Vc per ACI 318.
/// Vc = 0.17 * lambda * sqrt(f'c) * bw * d.
/// bw = 300 mm, d = 500 mm, f'c = 28 MPa, lambda = 1.0.
/// Vc = 0.17 * sqrt(28) * 300 * 500 / 1e3 = 0.17 * 5.2915 * 150 = 134.93 kN.
/// Design shear capacity phi * Vc = 0.75 * 134.93 = 101.2 kN.
#[test]
fn test_mccormac_rc_ex8_1_concrete_shear_strength() {
    let fc: f64 = 28.0;
    let bw: f64 = 300.0;
    let d: f64 = 500.0;
    let vc: f64 = 0.17 * fc.sqrt() * bw * d / 1e3; // kN
    let phi_vc: f64 = 0.75 * vc;

    assert_close(vc, 134.93, 0.02, "Concrete shear capacity Vc");
    assert_close(phi_vc, 101.2, 0.02, "Design concrete shear phi*Vc");
}

/// Example 8.2 (p. 223): Required stirrup spacing s for shear reinforcement.
/// Factored shear Vu = 250 kN. phi = 0.75.
/// Required Vs = Vu / phi - Vc = 250 / 0.75 - 134.93 = 333.33 - 134.93 = 198.4 kN.
/// Using 2-leg #3 stirrups (Av = 142 mm²), fyt = 420 MPa:
/// s = Av * fyt * d / Vs = 142 * 420 * 500 / (198.4 * 1e3) = 29820000 / 198400 = 150.3 mm.
#[test]
fn test_mccormac_rc_ex8_2_stirrup_spacing() {
    let vu: f64 = 250.0;
    let vc: f64 = 134.93;
    let av: f64 = 142.0;
    let fyt: f64 = 420.0;
    let d: f64 = 500.0;

    let vs_req: f64 = (vu / 0.75) - vc;
    let s: f64 = (av * fyt * d) / (vs_req * 1e3);

    assert_close(vs_req, 198.4, 0.02, "Required shear Vs");
    assert_close(s, 150.3, 0.02, "Required stirrup spacing s");
}

/// Example 8.4 (p. 230): Maximum allowed stirrup spacing limit.
/// When Vs <= 0.33 * sqrt(f'c) * bw * d: s_max = min(d / 2, 600 mm).
/// When Vs > 0.33 * sqrt(f'c) * bw * d: s_max = min(d / 4, 300 mm).
/// Here Vs = 198.4 kN <= 0.33 * sqrt(28) * 300 * 500 / 1e3 = 261.9 kN.
/// s_max = min(500 / 2, 600) = 250 mm.
#[test]
fn test_mccormac_rc_ex8_4_maximum_stirrup_spacing() {
    let fc: f64 = 28.0;
    let bw: f64 = 300.0;
    let d: f64 = 500.0;
    let vs: f64 = 198.4;

    let vs_limit: f64 = 0.33 * fc.sqrt() * bw * d / 1e3;
    let s_max: f64 = if vs <= vs_limit {
        (d / 2.0).min(600.0)
    } else {
        (d / 4.0).min(300.0)
    };

    assert_close(s_max, 250.0, 0.01, "Maximum stirrup spacing s_max");
}

// ================================================================
// Ch. 9: Columns & P-M Interaction
// ================================================================

/// Example 9.1 (p. 256): Concentric nominal capacity Pn,max of a tied short column.
/// Column 400 mm x 400 mm (Ag = 160,000 mm²), 8 #8 bars (Ast = 4000 mm²).
/// f'c = 28 MPa, fy = 420 MPa.
/// Pn,max = 0.80 * [0.85 * f'c * (Ag - Ast) + fy * Ast] (tied column).
/// Concrete term: 0.85 * 28 * (160000 - 4000) = 23.8 * 156000 = 3712.8 kN.
/// Steel term: 420 * 4000 = 1680.0 kN.
/// Pn,max = 0.80 * (3712.8 + 1680.0) = 0.80 * 5392.8 = 4314.2 kN.
/// Design capacity phi * Pn,max = 0.65 * 4314.2 = 2804.3 kN.
#[test]
fn test_mccormac_rc_ex9_1_tied_column_capacity() {
    let ag: f64 = 160_000.0;
    let ast: f64 = 4000.0;
    let fc: f64 = 28.0;
    let fy: f64 = 420.0;

    let p0: f64 = 0.85 * fc * (ag - ast) + fy * ast;
    let pn_max: f64 = 0.80 * p0 / 1e3; // kN
    let phi_pn: f64 = 0.65 * pn_max;

    assert_close(pn_max, 4314.2, 0.02, "Nominal axial capacity Pn,max");
    assert_close(phi_pn, 2804.3, 0.02, "Design axial capacity phi*Pn");
}

/// Example 9.2 (p. 260): Spiral column concentric capacity and spiral ratio rho_s.
/// For spiral column: Pn,max = 0.85 * P0, phi = 0.75.
/// Minimum spiral volumetric ratio rho_s = 0.45 * (Ag / Ach - 1) * (f'c / fyt).
/// For diameter D = 450 mm (Ag = pi * 450^2 / 4 = 159043 mm²), core diameter Dc = 370 mm (Ach = 107521 mm²).
/// rho_s = 0.45 * (159043 / 107521 - 1) * (28 / 420) = 0.45 * 0.4792 * 0.0667 = 0.01437.
#[test]
fn test_mccormac_rc_ex9_2_spiral_column_ratio() {
    let d_col: f64 = 450.0;
    let d_core: f64 = 370.0;
    let fc: f64 = 28.0;
    let fyt: f64 = 420.0;

    let ag: f64 = std::f64::consts::PI * d_col.powi(2) / 4.0;
    let ach: f64 = std::f64::consts::PI * d_core.powi(2) / 4.0;
    let rho_s: f64 = 0.45 * (ag / ach - 1.0) * (fc / fyt);

    assert_close(rho_s, 0.01437, 0.03, "Spiral volumetric ratio rho_s");
}

/// Example 9.4 (p. 275): Balanced point (Pb, Mb) on column P-M interaction diagram.
/// At balanced failure, extreme fiber concrete eps_c = 0.003, tension steel yields eps_s = eps_y = 0.0021.
/// cb = 600 / (600 + fy) * d = 600 / 1020 * 450 = 264.7 mm.
#[test]
fn test_mccormac_rc_ex9_4_column_balanced_neutral_axis() {
    let d: f64 = 450.0;
    let fy: f64 = 420.0;
    let cb: f64 = (600.0 / (600.0 + fy)) * d;

    assert_close(cb, 264.7, 0.02, "Balanced neutral axis cb");
}

// ================================================================
// Ch. 10: Slender Columns & Moment Magnification
// ================================================================

/// Example 10.1 (p. 312): Critical Euler buckling load Pc of an RC column.
/// Pc = pi^2 * (EI)eff / (k * lu)^2.
/// (EI)eff = (0.40 * Ec * Ig) / (1 + beta_dns).
/// For column 350 mm x 350 mm, Ec = 25000 MPa, Ig = 350^4 / 12 = 1.2505e9 mm^4.
/// beta_dns = 0.60, k = 1.0, lu = 4000 mm.
/// (EI)eff = 0.40 * 25000 * 1.2505e9 / 1.60 = 7.816e12 N*mm² = 7816 kN*m².
/// Pc = pi^2 * 7816 / 4.0^2 = 77142 / 16 = 4821 kN.
#[test]
fn test_mccormac_rc_ex10_1_slender_column_euler_pc() {
    let ec: f64 = 25_000.0; // MPa
    let h: f64 = 350.0;
    let ig: f64 = h.powi(4) / 12.0; // mm^4
    let beta_dns: f64 = 0.60;
    let lu: f64 = 4.0; // m
    let k: f64 = 1.0;

    let ei_eff: f64 = (0.40 * ec * ig / (1.0 + beta_dns)) / 1e9; // kN*m²
    let pc: f64 = std::f64::consts::PI.powi(2) * ei_eff / (k * lu).powi(2);

    assert_close(ei_eff, 7816.0, 0.02, "Effective stiffness (EI)eff");
    assert_close(pc, 4821.0, 0.02, "Critical column load Pc");
}

/// Example 10.2 (p. 318): Moment magnification factor delta_ns for non-sway column.
/// delta_ns = Cm / (1 - Pu / (0.75 * Pc)) >= 1.0.
/// Pu = 1500 kN, Pc = 4821 kN, M1 = 40 kN*m, M2 = 80 kN*m (single curvature: M1/M2 = -0.5).
/// Cm = 0.60 + 0.40 * (M1 / M2) = 0.60 - 0.20 = 0.40, but Cm >= 0.40.
/// Denominator = 1 - 1500 / (0.75 * 4821) = 1 - 1500 / 3615.7 = 1 - 0.4149 = 0.5851.
/// delta_ns = 0.40 / 0.5851 = 0.684 -> enforced minimum = 1.0.
#[test]
fn test_mccormac_rc_ex10_2_moment_magnifier() {
    let pu: f64 = 1500.0;
    let pc: f64 = 4821.0;
    let cm: f64 = 0.40_f64.max(0.60 + 0.40 * (-0.5));
    let denom: f64 = 1.0 - pu / (0.75 * pc);
    let delta_ns: f64 = (cm / denom).max(1.0);

    assert_close(delta_ns, 1.0, 0.01, "Moment magnifier delta_ns");
}

// ================================================================
// Ch. 12: Footings & Punching Shear
// ================================================================

/// Example 12.1 (p. 368): Two-way punching shear perimeter b0 and nominal capacity Vc.
/// Column 400 mm x 400 mm, footing effective depth d = 300 mm.
/// Critical section is at d/2 = 150 mm from column face.
/// b0 = 4 * (c + d) = 4 * (400 + 300) = 2800 mm.
/// Vc = 0.33 * lambda * sqrt(f'c) * b0 * d.
/// For f'c = 28 MPa, lambda = 1.0:
/// Vc = 0.33 * sqrt(28) * 2800 * 300 / 1e3 = 0.33 * 5.2915 * 840 = 1466.8 kN.
#[test]
fn test_mccormac_rc_ex12_1_punching_shear_perimeter() {
    let col_c: f64 = 400.0;
    let d: f64 = 300.0;
    let fc: f64 = 28.0;

    let b0: f64 = 4.0 * (col_c + d);
    let vc: f64 = 0.33 * fc.sqrt() * b0 * d / 1e3; // kN

    assert_close(b0, 2800.0, 0.01, "Punching shear perimeter b0");
    assert_close(vc, 1466.8, 0.02, "Punching shear capacity Vc");
}

// ================================================================
// Ch. 14: Retaining Walls
// ================================================================

/// Example 14.1 (p. 430): Active lateral earth pressure on a cantilever stem.
/// Rankine active earth pressure coefficient Ka = (1 - sin(phi)) / (1 + sin(phi)).
/// Soil friction angle phi = 30 deg -> sin(30) = 0.5 -> Ka = 0.5 / 1.5 = 1/3.
/// Unit weight gamma = 18 kN/m^3. Wall height H = 4.5 m.
/// Total active thrust Pa = 0.5 * Ka * gamma * H^2 = 0.5 * (1/3) * 18 * 4.5^2 = 3 * 20.25 = 60.75 kN/m.
/// Overturning moment about base Mo = Pa * (H / 3) = 60.75 * 1.5 = 91.13 kN*m/m.
#[test]
fn test_mccormac_rc_ex14_1_retaining_wall_active_thrust() {
    let phi_deg: f64 = 30.0;
    let phi_rad: f64 = phi_deg.to_radians();
    let ka: f64 = (1.0 - phi_rad.sin()) / (1.0 + phi_rad.sin());
    let gamma: f64 = 18.0; // kN/m^3
    let h: f64 = 4.5; // m

    let pa: f64 = 0.5 * ka * gamma * h.powi(2);
    let mo: f64 = pa * (h / 3.0);

    assert_close(ka, 1.0 / 3.0, 0.01, "Rankine Ka");
    assert_close(pa, 60.75, 0.02, "Active earth thrust Pa");
    assert_close(mo, 91.13, 0.02, "Overturning moment Mo");
}
