//! Property-based and exhaustive tests for design checks (issue #378).
//!
//! Acceptance criteria:
//! 1. More reinforcement or a larger section never lowers capacity.
//! 2. A larger load never lowers the utilization ratio.
//! 3. phi stays strictly within its design code limits.
//! 4. Property tested across all check families:
//!    - ACI 318 RC (`rc_check`)
//!    - CIRSOC 201 RC (`cirsoc201_check`)
//!    - AISC 360 Steel (`steel_check`)
//!    - CIRSOC 301 Steel (`cirsoc301_check`)
//!    - Eurocode 2 RC (`ec2_check`)
//!    - Eurocode 3 Steel (`ec3_check`)
//!    - NDS Timber (`timber_check`)
//!    - AISI S100 CFS (`cfs_check`)

use dedaliano_engine::postprocess::cfs_check as cfs;
use dedaliano_engine::postprocess::cirsoc201_check as cirsoc201;
use dedaliano_engine::postprocess::cirsoc301_check as cirsoc301;
use dedaliano_engine::postprocess::ec2_check as ec2;
use dedaliano_engine::postprocess::ec3_check as ec3;
use dedaliano_engine::postprocess::rc_check as rc;
use dedaliano_engine::postprocess::steel_check as steel;
use dedaliano_engine::postprocess::timber_check as timber;
use proptest::prelude::*;

// ======================================================================
// 1. phi Code Limits and Monotonicity
// ======================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// ACI 318-19 phi factor for flexure must strictly satisfy 0.65 <= phi <= 0.90
    /// across all tensile strains eps_t in [-0.05, 0.10], fy in [200, 800] MPa,
    /// Es in [150, 250] GPa.
    #[test]
    fn prop_aci318_phi_within_code_limits(
        eps_t in -0.05f64..0.10f64,
        fy in 200e6f64..800e6f64,
        es in 150e9f64..250e9f64,
    ) {
        let phi = rc::compute_phi::<f64>(eps_t, fy, es);
        prop_assert!(
            (0.65..=0.90).contains(&phi),
            "ACI 318-19 phi={phi} out of code bounds [0.65, 0.90] for eps_t={eps_t}, fy={fy}, es={es}"
        );
    }

    /// ACI 318-19 phi factor must be monotonically non-decreasing with tensile strain eps_t.
    #[test]
    fn prop_aci318_phi_monotone_with_strain(
        eps1 in -0.02f64..0.05f64,
        delta in 0.0f64..0.05f64,
        fy in 200e6f64..600e6f64,
        es in 180e9f64..220e9f64,
    ) {
        let eps2 = eps1 + delta;
        let phi1 = rc::compute_phi::<f64>(eps1, fy, es);
        let phi2 = rc::compute_phi::<f64>(eps2, fy, es);
        prop_assert!(
            phi2 >= phi1 - 1e-12,
            "Non-monotonic ACI 318 phi: phi(eps1={eps1})={phi1} > phi(eps2={eps2})={phi2}"
        );
    }

    /// ACI 318-19 beta1 concrete stress block factor must strictly satisfy 0.65 <= beta1 <= 0.85
    /// and be monotonically non-increasing with concrete compressive strength fc.
    #[test]
    fn prop_aci318_beta1_bounds_and_monotonicity(
        fc1 in 15e6f64..90e6f64,
        delta_fc in 0.0f64..50e6f64,
    ) {
        let fc2 = fc1 + delta_fc;
        let b1 = rc::compute_beta1::<f64>(fc1);
        let b2 = rc::compute_beta1::<f64>(fc2);
        prop_assert!((0.65..=0.85).contains(&b1), "beta1={b1} out of [0.65, 0.85]");
        prop_assert!((0.65..=0.85).contains(&b2), "beta1={b2} out of [0.65, 0.85]");
        prop_assert!(b2 <= b1 + 1e-12, "beta1 increased with fc: beta1({fc1})={b1} < beta1({fc2})={b2}");
    }

    /// CIRSOC 201-2005 & 2025 phi flexure factor must strictly satisfy 0.65 <= phi <= 0.90.
    #[test]
    fn prop_cirsoc201_phi_within_code_limits(
        eps_t in -0.05f64..0.10f64,
        fy in 200e6f64..600e6f64,
        es in 150e9f64..250e9f64,
        edition_idx in 0usize..2,
    ) {
        let edition = if edition_idx == 0 {
            cirsoc201::Cirsoc201Edition::E2005
        } else {
            cirsoc201::Cirsoc201Edition::E2025
        };
        let phi = cirsoc201::phi_flexure(edition, eps_t, fy, es);
        prop_assert!(
            (0.65..=0.90).contains(&phi),
            "CIRSOC 201 phi={phi} out of code bounds [0.65, 0.90] for edition={edition:?}, eps_t={eps_t}"
        );
    }

    /// CIRSOC 201 phi factor must be monotonically non-decreasing with tensile strain eps_t.
    #[test]
    fn prop_cirsoc201_phi_monotone_with_strain(
        eps1 in -0.02f64..0.05f64,
        delta in 0.0f64..0.05f64,
        fy in 200e6f64..600e6f64,
        es in 180e9f64..220e9f64,
        edition_idx in 0usize..2,
    ) {
        let edition = if edition_idx == 0 {
            cirsoc201::Cirsoc201Edition::E2005
        } else {
            cirsoc201::Cirsoc201Edition::E2025
        };
        let eps2 = eps1 + delta;
        let phi1 = cirsoc201::phi_flexure(edition, eps1, fy, es);
        let phi2 = cirsoc201::phi_flexure(edition, eps2, fy, es);
        prop_assert!(
            phi2 >= phi1 - 1e-12,
            "Non-monotonic CIRSOC 201 phi: phi(eps1={eps1})={phi1} > phi(eps2={eps2})={phi2}"
        );
    }
}

// ======================================================================
// 2. Capacity Monotonicity: Reinforcement & Section Geometry
// ======================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// ACI 318 RC flexural capacity: adding tension steel As never lowers nominal moment Mn.
    #[test]
    fn prop_aci318_flexure_monotone_with_tension_steel(
        b in 0.20f64..0.50f64,
        h in 0.35f64..0.80f64,
        fc in 20e6f64..50e6f64,
        fy in 300e6f64..500e6f64,
        as1_frac in 0.002f64..0.015f64,
        delta_as_frac in 0.0005f64..0.010f64,
    ) {
        let d = h - 0.05;
        let bd = b * d;
        let as1 = as1_frac * bd;
        let as2 = (as1_frac + delta_as_frac) * bd;

        let mut m1 = rc::RCMemberData {
            element_id: 1,
            fc,
            fy,
            es: Some(200e9),
            b,
            h,
            d,
            d_prime: None,
            as_tension: as1,
            as_compression: None,
            section_type: rc::RCSectionType::Rectangular,
            bf: None,
            hf: None,
            av: None,
            s_stirrup: None,
            lambda: None,
        };
        let (phi_mn1, f1, _phi1) = rc::flexural_capacity::<f64>(&m1);

        m1.as_tension = as2;
        let (phi_mn2, f2, _phi2) = rc::flexural_capacity::<f64>(&m1);

        prop_assert!(
            f2.mn >= f1.mn - 1e-6,
            "Nominal flexural capacity decreased with more tension steel: Mn(As1={as1:.6})={:?} > Mn(As2={as2:.6})={:?}",
            f1.mn, f2.mn
        );
        prop_assert!(
            phi_mn2 >= phi_mn1 - 1e-6,
            "Factored flexural capacity decreased with more tension steel: phiMn(As1)={phi_mn1} > phiMn(As2)={phi_mn2}"
        );
    }

    /// ACI 318 RC flexural capacity: increasing section width b or depth d never lowers Mn.
    #[test]
    fn prop_aci318_flexure_monotone_with_section_size(
        b1 in 0.20f64..0.40f64,
        delta_b in 0.0f64..0.20f64,
        h1 in 0.35f64..0.60f64,
        delta_h in 0.0f64..0.30f64,
        fc in 20e6f64..45e6f64,
        fy in 400e6f64..450e6f64,
    ) {
        let d1 = h1 - 0.05;
        let as_tens = 0.004 * b1 * d1;

        let m1 = rc::RCMemberData {
            element_id: 1,
            fc,
            fy,
            es: Some(200e9),
            b: b1,
            h: h1,
            d: d1,
            d_prime: None,
            as_tension: as_tens,
            as_compression: None,
            section_type: rc::RCSectionType::Rectangular,
            bf: None,
            hf: None,
            av: None,
            s_stirrup: None,
            lambda: None,
        };
        let (phi_mn1, f1, _) = rc::flexural_capacity::<f64>(&m1);

        let b2 = b1 + delta_b;
        let h2 = h1 + delta_h;
        let d2 = h2 - 0.05;
        let m2 = rc::RCMemberData {
            b: b2,
            h: h2,
            d: d2,
            ..m1
        };
        let (phi_mn2, f2, _) = rc::flexural_capacity::<f64>(&m2);

        prop_assert!(
            f2.mn >= f1.mn - 1e-6,
            "Nominal flexural capacity decreased with larger section: Mn(b1, h1)={:?} > Mn(b2, h2)={:?}",
            f1.mn, f2.mn
        );
        prop_assert!(
            phi_mn2 >= phi_mn1 - 1e-6,
            "Factored flexural capacity decreased with larger section: phiMn1={phi_mn1} > phiMn2={phi_mn2}"
        );
    }

    /// ACI 318 RC shear capacity: adding stirrups Av or reducing spacing s never lowers phi*Vn.
    #[test]
    fn prop_aci318_shear_monotone_with_stirrups(
        b in 0.20f64..0.50f64,
        h in 0.35f64..0.80f64,
        fc in 20e6f64..50e6f64,
        fy in 400e6f64..450e6f64,
        av1 in 1e-4f64..5e-4f64,
        delta_av in 0.0f64..5e-4f64,
        s1 in 0.15f64..0.30f64,
        s_red in 0.01f64..0.08f64,
    ) {
        let d = h - 0.05;
        let as_tens = 0.005 * b * d;
        let av2 = av1 + delta_av;
        let s2 = s1 - s_red;

        let m1 = rc::RCMemberData {
            element_id: 1,
            fc,
            fy,
            es: Some(200e9),
            b,
            h,
            d,
            d_prime: None,
            as_tension: as_tens,
            as_compression: None,
            section_type: rc::RCSectionType::Rectangular,
            bf: None,
            hf: None,
            av: Some(av1),
            s_stirrup: Some(s1),
            lambda: Some(1.0),
        };
        let phi_vn1: f64 = rc::shear_capacity(&m1);

        let m2 = rc::RCMemberData {
            av: Some(av2),
            s_stirrup: Some(s2),
            ..m1
        };
        let phi_vn2: f64 = rc::shear_capacity(&m2);

        prop_assert!(
            phi_vn2 >= phi_vn1 - 1e-6,
            "Shear capacity decreased with more stirrups: phiVn(Av1, s1)={phi_vn1} > phiVn(Av2, s2)={phi_vn2}"
        );
    }

    /// ACI 318 RC axial capacity: increasing gross section Ag or reinforcement never lowers capacity.
    #[test]
    fn prop_aci318_axial_monotone_with_section_and_steel(
        b1 in 0.20f64..0.40f64,
        delta_b in 0.0f64..0.20f64,
        h1 in 0.20f64..0.40f64,
        delta_h in 0.0f64..0.20f64,
        fc in 20e6f64..50e6f64,
        fy in 400e6f64..450e6f64,
        ast1 in 0.0004f64..0.0020f64,
        delta_ast in 0.0f64..0.0020f64,
    ) {
        let ast2 = ast1 + delta_ast;
        let m1 = rc::RCMemberData {
            element_id: 1,
            fc,
            fy,
            es: Some(200e9),
            b: b1,
            h: h1,
            d: h1 - 0.04,
            d_prime: Some(0.04),
            as_tension: ast1 * 0.5,
            as_compression: Some(ast1 * 0.5),
            section_type: rc::RCSectionType::Rectangular,
            bf: None,
            hf: None,
            av: None,
            s_stirrup: None,
            lambda: None,
        };
        let (pn_max1, pnt1): (f64, f64) = rc::axial_capacity(&m1);

        let m2 = rc::RCMemberData {
            b: b1 + delta_b,
            h: h1 + delta_h,
            as_tension: ast2 * 0.5,
            as_compression: Some(ast2 * 0.5),
            ..m1
        };
        let (pn_max2, pnt2): (f64, f64) = rc::axial_capacity(&m2);

        prop_assert!(
            pn_max2 >= pn_max1 - 1e-6,
            "Compressive axial capacity decreased: pn_max1={pn_max1} > pn_max2={pn_max2}"
        );
        prop_assert!(
            pnt2 >= pnt1 - 1e-6,
            "Tensile axial capacity decreased: pnt1={pnt1} > pnt2={pnt2}"
        );
    }

    /// AISC 360 Steel capacity: increasing section size geometrically never lowers tension, compression, flexure or shear capacity.
    #[test]
    fn prop_steel_capacity_monotone_with_section_scale(
        scale in 1.001f64..2.5f64,
        fy in 250e6f64..450e6f64,
    ) {
        let base = steel::SteelMemberData {
            element_id: 1,
            fy,
            fu: Some(1.2 * fy),
            ag: 0.005,
            an: Some(0.005),
            u_factor: Some(1.0),
            aw: Some(0.002),
            cv1: Some(1.0),
            lby: 3.0,
            lbz: 3.0,
            ky: Some(1.0),
            kz: Some(1.0),
            iy: 5e-5,
            iz: 2e-5,
            ry: 0.10,
            rz: 0.05,
            zy: 0.001,
            zz: 0.0004,
            sy: 0.0009,
            sz: 0.00035,
            j: 1e-7,
            cw: Some(1e-8),
            lb: Some(3.0),
            cb: Some(1.0),
            e: 200e9,
            g: Some(77e9),
            lcz: None,
            depth: Some(0.30),
            flange_slenderness: None,
            compression_elements: Some(vec![]),
            load_position: None,
        };
        let t1: f64 = steel::tension_capacity(&base);
        let c1: f64 = steel::compression_capacity(&base);
        let my1: f64 = steel::flexural_capacity_y(&base);
        let v1: f64 = steel::shear_capacity(&base);

        let s2 = scale * scale;
        let s3 = s2 * scale;
        let s4 = s2 * s2;
        let s6 = s3 * s3;

        let scaled = steel::SteelMemberData {
            ag: base.ag * s2,
            an: Some(base.ag * s2),
            aw: Some(0.002 * s2),
            iy: base.iy * s4,
            iz: base.iz * s4,
            ry: base.ry * scale,
            rz: base.rz * scale,
            zy: base.zy * s3,
            zz: base.zz * s3,
            sy: base.sy * s3,
            sz: base.sz * s3,
            j: base.j * s4,
            cw: Some(1e-8 * s6),
            depth: Some(0.30 * scale),
            ..base
        };
        let t2: f64 = steel::tension_capacity(&scaled);
        let c2: f64 = steel::compression_capacity(&scaled);
        let my2: f64 = steel::flexural_capacity_y(&scaled);
        let v2: f64 = steel::shear_capacity(&scaled);

        prop_assert!(t2 >= t1 - 1e-6, "Steel tension capacity decreased: t1={t1} > t2={t2}");
        prop_assert!(c2 >= c1 - 1e-6, "Steel compression capacity decreased: c1={c1} > c2={c2}");
        prop_assert!(my2 >= my1 - 1e-6, "Steel flexure-Y capacity decreased: my1={my1} > my2={my2}");
        prop_assert!(v2 >= v1 - 1e-6, "Steel shear capacity decreased: v1={v1} > v2={v2}");
    }

    /// AISC 360 Steel plastic flexural capacity (braced beam) never lowers when Zy increases.
    #[test]
    fn prop_steel_braced_flexure_monotone_with_zy(
        zy1 in 0.0002f64..0.0020f64,
        delta_zy in 0.0f64..0.0010f64,
        fy in 250e6f64..450e6f64,
    ) {
        let m1 = steel::SteelMemberData {
            element_id: 1,
            fy,
            fu: Some(1.2 * fy),
            ag: 0.005,
            an: Some(0.005),
            u_factor: Some(1.0),
            aw: Some(0.002),
            cv1: Some(1.0),
            lby: 0.0,
            lbz: 0.0,
            ky: Some(1.0),
            kz: Some(1.0),
            iy: 5e-5,
            iz: 2e-5,
            ry: 0.10,
            rz: 0.05,
            zy: zy1,
            zz: zy1 * 0.5,
            sy: zy1 * 0.9,
            sz: zy1 * 0.45,
            j: 1e-7,
            cw: Some(1e-8),
            lb: Some(0.0),
            cb: Some(1.0),
            e: 200e9,
            g: Some(77e9),
            lcz: None,
            depth: Some(0.30),
            flange_slenderness: None,
            compression_elements: Some(vec![]),
            load_position: None,
        };
        let my1: f64 = steel::flexural_capacity_y(&m1);

        let m2 = steel::SteelMemberData {
            zy: zy1 + delta_zy,
            sy: (zy1 + delta_zy) * 0.9,
            ..m1
        };
        let my2: f64 = steel::flexural_capacity_y(&m2);

        prop_assert!(my2 >= my1 - 1e-6, "Braced flexure capacity decreased: my1={my1} > my2={my2}");
    }
}

// ======================================================================
// 3. Load Monotonicity on Utilization Ratios
// ======================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// ACI 318 RC member check: scaling factored loads by alpha >= 1.0 never lowers the unity ratio.
    #[test]
    fn prop_aci318_load_monotonicity(
        mu in 1e3f64..200e3f64,
        vu in 1e3f64..100e3f64,
        alpha in 1.001f64..2.5f64,
    ) {
        let member = rc::RCMemberData {
            element_id: 1,
            fc: 30e6,
            fy: 420e6,
            es: Some(200e9),
            b: 0.30,
            h: 0.50,
            d: 0.45,
            d_prime: None,
            as_tension: 0.0015,
            as_compression: None,
            section_type: rc::RCSectionType::Rectangular,
            bf: None,
            hf: None,
            av: Some(1.57e-4),
            s_stirrup: Some(0.15),
            lambda: Some(1.0),
        };

        let f1 = rc::RCDesignForces {
            element_id: 1,
            mu,
            vu: Some(vu),
            nu: Some(0.0),
        };
        let input1 = rc::RCCheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &rc::check_rc_members(&input1)[0];

        let f2 = rc::RCDesignForces {
            element_id: 1,
            mu: alpha * mu,
            vu: Some(alpha * vu),
            nu: Some(0.0),
        };
        let input2 = rc::RCCheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &rc::check_rc_members(&input2)[0];

        prop_assert!(
            res2.flexure_ratio >= res1.flexure_ratio - 1e-12,
            "Flexure ratio decreased under scaled load: r1={} > r2={}",
            res1.flexure_ratio, res2.flexure_ratio
        );
        prop_assert!(
            res2.shear_ratio >= res1.shear_ratio - 1e-12,
            "Shear ratio decreased under scaled load: r1={} > r2={}",
            res1.shear_ratio, res2.shear_ratio
        );
        prop_assert!(
            res2.unity_ratio >= res1.unity_ratio - 1e-12,
            "Overall unity ratio decreased under scaled load: r1={} > r2={}",
            res1.unity_ratio, res2.unity_ratio
        );
    }

    /// CIRSOC 201 member check: scaling factored loads by alpha >= 1.0 never lowers the unity ratio.
    #[test]
    fn prop_cirsoc201_load_monotonicity(
        mu in 1e3f64..150e3f64,
        vu in 1e3f64..80e3f64,
        alpha in 1.001f64..2.5f64,
        edition_idx in 0usize..2,
    ) {
        let edition = if edition_idx == 0 {
            cirsoc201::Cirsoc201Edition::E2005
        } else {
            cirsoc201::Cirsoc201Edition::E2025
        };
        let member = cirsoc201::Cirsoc201MemberData {
            element_id: 1,
            edition,
            fc: 25e6,
            fy: 420e6,
            es: Some(200e9),
            b: 0.25,
            bf: None,
            hf: None,
            h: 0.45,
            d: 0.40,
            as_tension: 0.0012,
            as_compression: None,
            d_prime: None,
            av: Some(1.57e-4),
            s_stirrup: Some(0.15),
            fyt: Some(420e6),
            lambda: Some(1.0),
            ag: None,
        };

        let f1 = cirsoc201::Cirsoc201DesignForces {
            element_id: 1,
            mu: Some(mu),
            vu: Some(vu),
            axial_compression: Some(0.0),
        };
        let input1 = cirsoc201::Cirsoc201CheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &cirsoc201::check_cirsoc201_members(&input1)[0];

        let f2 = cirsoc201::Cirsoc201DesignForces {
            element_id: 1,
            mu: Some(alpha * mu),
            vu: Some(alpha * vu),
            axial_compression: Some(0.0),
        };
        let input2 = cirsoc201::Cirsoc201CheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &cirsoc201::check_cirsoc201_members(&input2)[0];

        prop_assert!(
            res2.unity_ratio >= res1.unity_ratio - 1e-12,
            "CIRSOC 201 unity ratio decreased under scaled load: r1={} > r2={}",
            res1.unity_ratio, res2.unity_ratio
        );
    }

    /// AISC 360 Steel member check: scaling individual load components by alpha >= 1.0 never lowers unity ratio.
    #[test]
    fn prop_steel_load_monotonicity(
        n in 10e3f64..100e3f64,
        my in 5e3f64..50e3f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = steel::SteelMemberData {
            element_id: 1,
            fy: 345e6,
            fu: Some(450e6),
            ag: 0.005,
            an: Some(0.005),
            u_factor: Some(1.0),
            aw: Some(0.002),
            cv1: Some(1.0),
            lby: 3.0,
            lbz: 3.0,
            ky: Some(1.0),
            kz: Some(1.0),
            iy: 5e-5,
            iz: 2e-5,
            ry: 0.10,
            rz: 0.05,
            zy: 0.001,
            zz: 0.0004,
            sy: 0.0009,
            sz: 0.00035,
            j: 1e-7,
            cw: Some(1e-8),
            lb: Some(3.0),
            cb: Some(1.0),
            e: 200e9,
            g: Some(77e9),
            lcz: None,
            depth: Some(0.30),
            flange_slenderness: None,
            compression_elements: Some(vec![]),
            load_position: None,
        };

        let f1 = steel::ElementDesignForces {
            element_id: 1,
            n: -n, // compression
            vy: Some(0.0),
            my,
            mz: Some(0.0),
        };
        let input1 = steel::SteelCheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &steel::check_steel_members(&input1)[0];

        let f2 = steel::ElementDesignForces {
            element_id: 1,
            n: -alpha * n,
            vy: Some(0.0),
            my: alpha * my,
            mz: Some(0.0),
        };
        let input2 = steel::SteelCheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &steel::check_steel_members(&input2)[0];

        prop_assert!(
            res2.compression_ratio >= res1.compression_ratio - 1e-12,
            "Compression ratio decreased: r1={} > r2={}",
            res1.compression_ratio, res2.compression_ratio
        );
        prop_assert!(
            res2.flexure_y_ratio >= res1.flexure_y_ratio - 1e-12,
            "Flexure-Y ratio decreased: r1={} > r2={}",
            res1.flexure_y_ratio, res2.flexure_y_ratio
        );
        prop_assert!(
            res2.unity_ratio >= res1.unity_ratio - 1e-12,
            "Steel unity ratio decreased: r1={} > r2={}",
            res1.unity_ratio, res2.unity_ratio
        );
    }

    /// Eurocode 2 RC member check: scaling loads by alpha >= 1.0 never lowers the ratios.
    #[test]
    fn prop_ec2_load_monotonicity(
        med in 5e3f64..80e3f64,
        ved in 5e3f64..40e3f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = ec2::Ec2MemberData {
            element_id: 1,
            fck: 30e6,
            fyk: 500e6,
            b: 0.30,
            h: 0.50,
            d: 0.45,
            as_tension: 0.0012,
            as_compression: None,
            d_prime: None,
            es: Some(200e9),
            gamma_c: Some(1.5),
            gamma_s: Some(1.15),
            alpha_cc: Some(1.0),
            asw: Some(1.57e-4),
            s_stirrup: Some(0.15),
            theta_shear: Some(0.785), // 45 deg
            bw: None,
            z: None,
        };

        let f1 = ec2::Ec2DesignForces {
            element_id: 1,
            m_ed: Some(med),
            v_ed: Some(ved),
            n_ed: Some(0.0),
        };
        let input1 = ec2::Ec2CheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &ec2::check_ec2_members(&input1)[0];

        let f2 = ec2::Ec2DesignForces {
            element_id: 1,
            m_ed: Some(alpha * med),
            v_ed: Some(alpha * ved),
            n_ed: Some(0.0),
        };
        let input2 = ec2::Ec2CheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &ec2::check_ec2_members(&input2)[0];

        prop_assert!(
            res2.flexure_ratio >= res1.flexure_ratio - 1e-12,
            "EC2 flexure ratio decreased: r1={} > r2={}",
            res1.flexure_ratio, res2.flexure_ratio
        );
        prop_assert!(
            res2.shear_ratio >= res1.shear_ratio - 1e-12,
            "EC2 shear ratio decreased: r1={} > r2={}",
            res1.shear_ratio, res2.shear_ratio
        );
    }

    /// Eurocode 3 Steel member check: scaling loads by alpha >= 1.0 never lowers flexure ratio.
    #[test]
    fn prop_ec3_load_monotonicity(
        my_ed in 5e3f64..60e3f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = ec3::Ec3MemberData {
            element_id: 1,
            fy: 275e6,
            e: Some(210e9),
            a: 0.006,
            a_eff: None,
            wel_y: 0.0008,
            wpl_y: 0.00095,
            weff_y: None,
            wel_z: 0.0003,
            wpl_z: 0.00038,
            weff_z: None,
            e_n_y: None,
            e_n_z: None,
            iy: 4e-5,
            iz: 1.5e-5,
            it: 1e-7,
            iw: 1e-8,
            lcr_y: 3.5,
            lcr_z: 3.5,
            lb: 3.5,
            section_class: ec3::SectionClass::Class1,
            buckling_curve_y: ec3::BucklingCurve::A,
            buckling_curve_z: ec3::BucklingCurve::B,
            buckling_curve_lt: ec3::BucklingCurve::B,
            gamma_m0: Some(1.0),
            gamma_m1: Some(1.0),
            c1: Some(1.0),
            av: Some(0.0025),
            interaction: None,
            geometry: None,
            ltb: None,
        };

        let f1 = ec3::Ec3DesignForces {
            element_id: 1,
            n_ed: Some(0.0),
            my_ed: Some(my_ed),
            mz_ed: Some(0.0),
            v_ed: Some(0.0),
        };
        let input1 = ec3::Ec3CheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &ec3::check_ec3_members(&input1)[0];

        let f2 = ec3::Ec3DesignForces {
            element_id: 1,
            n_ed: Some(0.0),
            my_ed: Some(alpha * my_ed),
            mz_ed: Some(0.0),
            v_ed: Some(0.0),
        };
        let input2 = ec3::Ec3CheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &ec3::check_ec3_members(&input2)[0];

        prop_assert!(
            res2.flexure_ratio_y >= res1.flexure_ratio_y - 1e-12,
            "EC3 flexure-Y ratio decreased: r1={} > r2={}",
            res1.flexure_ratio_y, res2.flexure_ratio_y
        );
    }

    /// NDS Timber member check: scaling loads by alpha >= 1.0 never lowers the unity ratio.
    #[test]
    fn prop_timber_load_monotonicity(
        m in 500.0f64..5000.0f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = timber::TimberMemberData {
            element_id: 1,
            fb: 10e6,
            ft: 6e6,
            fc: 9e6,
            fv: 1.2e6,
            e: 11e9,
            e_min: Some(4e9),
            b: 0.10,
            d: 0.20,
            le: 3.0,
            le_b: None,
            lu: Some(3.0),
            le_bending: None,
            cd: Some(1.0),
            cm: None,
            ct: None,
            cf_bending: None,
            cf_tension: None,
            cf_compression: None,
            cfu: None,
            ci: None,
            cr: None,
        };

        let f1 = timber::TimberDesignForces {
            element_id: 1,
            m,
            n: Some(0.0),
            v: Some(0.0),
        };
        let input1 = timber::TimberCheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &timber::check_timber_members(&input1)[0];

        let f2 = timber::TimberDesignForces {
            element_id: 1,
            m: alpha * m,
            n: Some(0.0),
            v: Some(0.0),
        };
        let input2 = timber::TimberCheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &timber::check_timber_members(&input2)[0];

        prop_assert!(
            res2.unity_ratio >= res1.unity_ratio - 1e-12,
            "Timber unity ratio decreased: r1={} > r2={}",
            res1.unity_ratio, res2.unity_ratio
        );
    }

    /// AISI S100 CFS member check: scaling loads by alpha >= 1.0 never lowers flexure ratio.
    #[test]
    fn prop_cfs_load_monotonicity(
        mx in 500.0f64..5000.0f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = cfs::CfsMemberData {
            element_id: 1,
            fy: 230e6,
            e: 200e9,
            ag: 0.0015,
            ae: 0.0012,
            ix: 3e-6,
            se_x: 3e-5,
            sf_x: 3.5e-5,
            iy: 1e-6,
            se_y: 1.5e-5,
            rx: 0.045,
            ry: 0.025,
            j: 1e-9,
            cw: 1e-10,
            lb: 2.0,
            lc: 2.0,
            k: Some(1.0),
            cb: Some(1.0),
            h: Some(0.15),
            t: Some(0.002),
            xo: Some(0.0),
            kx_lx: Some(2.0),
            kt_lt: Some(2.0),
            fcrd: None,
            fcrd_flex: None,
            aw: Some(0.0003),
        };

        let f1 = cfs::CfsDesignForces {
            element_id: 1,
            axial: Some(0.0),
            mx: Some(mx),
            my: Some(0.0),
            shear: Some(0.0),
        };
        let input1 = cfs::CfsCheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
        };
        let res1 = &cfs::check_cfs_members(&input1)[0];

        let f2 = cfs::CfsDesignForces {
            element_id: 1,
            axial: Some(0.0),
            mx: Some(alpha * mx),
            my: Some(0.0),
            shear: Some(0.0),
        };
        let input2 = cfs::CfsCheckInput {
            members: vec![member],
            forces: vec![f2],
        };
        let res2 = &cfs::check_cfs_members(&input2)[0];

        prop_assert!(
            res2.flexure_ratio_x >= res1.flexure_ratio_x - 1e-12,
            "CFS flexure ratio decreased: r1={} > r2={}",
            res1.flexure_ratio_x, res2.flexure_ratio_x
        );
    }

    /// CIRSOC 201 flexural capacity: adding tension steel As never lowers nominal moment Mn.
    #[test]
    fn prop_cirsoc201_flexure_monotone_with_tension_steel(
        b in 0.20f64..0.45f64,
        h in 0.35f64..0.75f64,
        fc in 20e6f64..45e6f64,
        fy in 400e6f64..450e6f64,
        as1_frac in 0.002f64..0.015f64,
        delta_as_frac in 0.0005f64..0.008f64,
        edition_idx in 0usize..2,
    ) {
        let edition = if edition_idx == 0 {
            cirsoc201::Cirsoc201Edition::E2005
        } else {
            cirsoc201::Cirsoc201Edition::E2025
        };
        let d = h - 0.05;
        let bd = b * d;
        let as1 = as1_frac * bd;
        let as2 = (as1_frac + delta_as_frac) * bd;

        let m1 = cirsoc201::Cirsoc201MemberData {
            element_id: 1,
            edition,
            fc,
            fy,
            es: Some(200e9),
            b,
            bf: None,
            hf: None,
            h,
            d,
            as_tension: as1,
            as_compression: None,
            d_prime: None,
            av: None,
            s_stirrup: None,
            fyt: None,
            lambda: Some(1.0),
            ag: None,
        };
        let f1 = cirsoc201::nominal_flexure(&m1).unwrap();

        let m2 = cirsoc201::Cirsoc201MemberData {
            as_tension: as2,
            ..m1
        };
        let f2 = cirsoc201::nominal_flexure(&m2).unwrap();

        prop_assert!(
            f2.mn >= f1.mn - 1e-6,
            "CIRSOC 201 nominal moment decreased with more steel: Mn(As1)={} > Mn(As2)={}",
            f1.mn, f2.mn
        );
    }

    /// CIRSOC 301 Steel member check: scaling loads by alpha >= 1.0 never lowers unity ratio.
    #[test]
    fn prop_cirsoc301_load_monotonicity(
        mu in 5e3f64..60e3f64,
        alpha in 1.001f64..2.0f64,
    ) {
        let member = cirsoc301::Cirsoc301MemberData {
            element_id: 1,
            fy: 250e6,
            fu: 400e6,
            e: 200e9,
            a: 0.005,
            i_strong: 5e-5,
            i_weak: 2e-5,
            h: 0.30,
            b: 0.15,
            tw: 0.008,
            tf: 0.012,
            zx: Some(0.001),
            zy: Some(0.0004),
            sx: Some(0.0009),
            sy: Some(0.00035),
            c: None,
            l: 3.0,
            lb: 3.0,
            kx: Some(1.0),
            ky: Some(1.0),
            j: Some(1e-7),
            cw: Some(1e-8),
            cb: Some(1.0),
            built_up: Some(false),
            shape: Some("I".to_string()),
            hw: None,
            load_position: None,
            g: Some(77e9),
            compression_elements: None,
            kz: Some(1.0),
            lz: Some(3.0),
            shear_centre_offset: None,
            tension_connection: None,
            angle: None,
        };

        let f1 = cirsoc301::Cirsoc301DesignForces {
            element_id: 1,
            nu: 0.0,
            mu_strong: mu,
            mu_weak: 0.0,
            vu: 0.0,
        };
        let input1 = cirsoc301::Cirsoc301CheckInput {
            members: vec![member.clone()],
            forces: vec![f1],
            steps: false,
        };
        let res1 = &cirsoc301::check_cirsoc301_members(&input1)[0];

        let f2 = cirsoc301::Cirsoc301DesignForces {
            element_id: 1,
            nu: 0.0,
            mu_strong: alpha * mu,
            mu_weak: 0.0,
            vu: 0.0,
        };
        let input2 = cirsoc301::Cirsoc301CheckInput {
            members: vec![member],
            forces: vec![f2],
            steps: false,
        };
        let res2 = &cirsoc301::check_cirsoc301_members(&input2)[0];

        prop_assert!(
            res2.unity_ratio >= res1.unity_ratio - 1e-12,
            "CIRSOC 301 unity ratio decreased: r1={} > r2={}",
            res1.unity_ratio, res2.unity_ratio
        );
    }
}
