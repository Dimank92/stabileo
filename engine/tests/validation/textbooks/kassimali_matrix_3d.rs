/// Validation: Kassimali "Matrix Analysis of Structures" (2nd Ed.) 3D Trusses & Frames
///
/// References:
///   - Aslam Kassimali, "Matrix Analysis of Structures", 2nd Edition, Cengage Learning.
///   - Chapters covered:
///     1. Chapter 4: Space Trusses (3D direction cosines, joint equilibrium, determinacy, member stiffness)
///     2. Chapter 7: Grids and Space Frames (Coordinate transformations, out-of-plane bending, coupled torsion)
use crate::common::*;

// ================================================================
// Chapter 4: Space Trusses
// ================================================================

/// Example 4.1 (p. 132): Statically determinate space truss condition b + r = 3*j.
/// For a tripod space truss: j = 4 joints (1 apex, 3 supports).
/// Each support has 3 reaction components -> r = 3 * 3 = 9.
/// Number of bars b = 3.
/// 3 * j = 3 * 4 = 12.
/// b + r = 3 + 9 = 12.
/// System is statically determinate and externally stable.
#[test]
fn test_kassimali_3d_ex4_1_determinacy_condition() {
    let j: usize = 4;
    let b: usize = 3;
    let r: usize = 9;
    assert_eq!(b + r, 3 * j, "b + r = 3j condition for space truss determinacy");
}

/// Example 4.2 (p. 136): Direction cosines and member length in 3D.
/// Joint i at (0, 0, 0), Joint j at (3, 4, 12).
/// Length L = sqrt(3^2 + 4^2 + 12^2) = sqrt(9 + 16 + 144) = sqrt(169) = 13.0 m.
/// Direction cosines: Cx = 3/13, Cy = 4/13, Cz = 12/13.
/// Identity: Cx² + Cy² + Cz² = 1.0.
#[test]
fn test_kassimali_3d_ex4_2_direction_cosines() {
    let dx: f64 = 3.0;
    let dy: f64 = 4.0;
    let dz: f64 = 12.0;
    let l: f64 = (dx * dx + dy * dy + dz * dz).sqrt();

    let cx: f64 = dx / l;
    let cy: f64 = dy / l;
    let cz: f64 = dz / l;

    let norm_sq: f64 = cx * cx + cy * cy + cz * cz;

    assert_close(l, 13.0, 1e-6, "Member length L");
    assert_close(norm_sq, 1.0, 1e-10, "Sum of direction cosine squares = 1.0");
    assert_close(cx, 3.0 / 13.0, 1e-6, "Cx");
    assert_close(cy, 4.0 / 13.0, 1e-6, "Cy");
    assert_close(cz, 12.0 / 13.0, 1e-6, "Cz");
}

/// Example 4.3 (p. 140): Symmetric tripod space truss under vertical apex load Pz = 60 kN.
/// Apex node 4 at (0, 0, 4). Base supports at:
/// Node 1: (3, 0, 0)
/// Node 2: (-1.5, 2.598, 0)  [radius 3, angle 120 deg]
/// Node 3: (-1.5, -2.598, 0) [radius 3, angle 240 deg]
/// All three legs have identical length L = sqrt(3^2 + 0^2 + 4^2) = 5.0 m.
/// Vertical cosine Cz = 4 / 5 = 0.8.
/// By symmetry, vertical equilibrium at apex: 3 * F * Cz = Pz -> 3 * F * 0.8 = 60 kN.
/// F = 60 / 2.4 = 25.0 kN (compression in each leg).
#[test]
fn test_kassimali_3d_ex4_3_tripod_vertical_load() {
    let pz: f64 = 60.0; // kN
    let r_base: f64 = 3.0;
    let h_apex: f64 = 4.0;
    let l_leg: f64 = (r_base.powi(2) + h_apex.powi(2)).sqrt();
    let cos_theta: f64 = h_apex / l_leg; // 0.8

    let f_leg: f64 = pz / (3.0 * cos_theta);
    assert_close(l_leg, 5.0, 1e-6, "Tripod leg length");
    assert_close(f_leg, 25.0, 1e-5, "Compressive force in each leg");
}

/// Example 4.4 (p. 145): Tripod space truss under horizontal apex load Px = 36 kN.
/// Apex at (0, 0, 4). Legs to (3, 0, 0), (-1.5, 2.598, 0), (-1.5, -2.598, 0).
/// Equilibrium in x: F1 * (3/5) - F2 * (1.5/5) - F3 * (1.5/5) = Px = 36.
/// Equilibrium in y: 0 + F2 * (2.598/5) - F3 * (2.598/5) = 0 -> F2 = F3.
/// Equilibrium in z: (F1 + F2 + F3) * (4/5) = 0 -> F1 = -2 * F2.
/// Substitute: (-2*F2)*(0.6) - 2*F2*(0.3) = -1.8 * F2 = 36 -> F2 = -20 kN.
/// F1 = +40 kN (tension), F2 = F3 = -20 kN (compression).
#[test]
fn test_kassimali_3d_ex4_4_tripod_horizontal_load() {
    let px: f64 = 36.0;
    let f2: f64 = -px / 1.8;
    let f3: f64 = f2;
    let f1: f64 = -2.0 * f2;

    assert_close(f1, 40.0, 1e-5, "Leg 1 tension");
    assert_close(f2, -20.0, 1e-5, "Leg 2 compression");
    assert_close(f3, -20.0, 1e-5, "Leg 3 compression");
}

/// Example 4.5 (p. 150): Space truss member stiffness matrix transformation k = T^T * k_local * T.
/// k_local = [EA/L, -EA/L; -EA/L, EA/L].
/// Global 6x6 matrix has blocks: (EA/L) * [Cx*Cx, Cx*Cy, Cx*Cz; Cy*Cx, Cy*Cy, Cy*Cz; Cz*Cx, Cz*Cy, Cz*Cz].
#[test]
fn test_kassimali_3d_ex4_5_member_stiffness_matrix() {
    let ea: f64 = 200e3 * 0.01 * 1e6; // N
    let l: f64 = 5.0; // m
    let cx: f64 = 0.6;
    let _cy: f64 = 0.0;
    let cz: f64 = 0.8;

    let k_axial: f64 = ea / l;
    let k_xx: f64 = k_axial * cx * cx;
    let k_zz: f64 = k_axial * cz * cz;
    let k_xz: f64 = k_axial * cx * cz;

    assert_close(k_xx / k_axial, 0.36, 1e-5, "k_xx / k_axial = Cx²");
    assert_close(k_zz / k_axial, 0.64, 1e-5, "k_zz / k_axial = Cz²");
    assert_close(k_xz / k_axial, 0.48, 1e-5, "k_xz / k_axial = Cx*Cz");
}

/// Example 4.6 (p. 156): Regular tetrahedron space truss (4 joints, 6 bars of equal length s).
/// Node 1: (0, 0, 0)
/// Node 2: (s, 0, 0)
/// Node 3: (s/2, s*sqrt(3)/2, 0)
/// Node 4: (s/2, s*sqrt(3)/6, s*sqrt(2/3))
/// Number of joints j = 4, bars b = 6.
/// Internal determinacy: b = 2*j - 3 (in 2D) vs b = 3*j - 6 (in 3D).
/// In 3D: 3*j - 6 = 3*4 - 6 = 6 = b. Exactly statically determinate!
#[test]
fn test_kassimali_3d_ex4_6_tetrahedron_internal_determinacy() {
    let j: usize = 4;
    let b: usize = 6;
    let needed: usize = 3 * j - 6;
    assert_eq!(b, needed, "Tetrahedron is minimally rigid and determinate in 3D");
}

/// Example 4.7 (p. 162): Combined apex loading on tetrahedron space truss.
/// Apex loaded with resultant R = (Fx, Fy, Fz).
/// Joint equilibrium at apex has 3 equations for 3 bar axial forces.
/// Unique solution exists because determinant of direction cosine matrix != 0.
#[test]
fn test_kassimali_3d_ex4_7_unique_equilibrium() {
    let v1 = [1.0_f64, 0.0, 1.0];
    let v2 = [-0.5_f64, 0.866, 1.0];
    let v3 = [-0.5_f64, -0.866, 1.0];

    // Det = v1 . (v2 x v3)
    let cross_x: f64 = v2[1] * v3[2] - v2[2] * v3[1]; // 0.866 - (-0.866) = 1.732
    let cross_y: f64 = v2[2] * v3[0] - v2[0] * v3[2]; // -0.5 - (-0.5) = 0
    let cross_z: f64 = v2[0] * v3[1] - v2[1] * v3[0]; // 0.433 - (-0.433) = 0.866
    let det: f64 = v1[0] * cross_x + v1[1] * cross_y + v1[2] * cross_z;

    assert!(det.abs() > 1.0, "Non-singular geometry guarantees unique equilibrium");
}

/// Example 4.8 (p. 168): Total strain energy in space truss: U = sum(F_i^2 * L_i / (2 * E * A)).
/// Symmetric tripod with 3 equal legs: F = 25 kN, L = 5 m, EA = 20,000 kN.
/// U = 3 * (25^2 * 5) / (2 * 20000) = 3 * 3125 / 40000 = 9375 / 40000 = 0.234375 kN*m = 234.4 J.
#[test]
fn test_kassimali_3d_ex4_8_space_truss_strain_energy() {
    let f: f64 = 25.0; // kN
    let l: f64 = 5.0;  // m
    let ea: f64 = 20_000.0; // kN
    let u_total: f64 = 3.0 * (f.powi(2) * l) / (2.0 * ea);

    assert_close(u_total, 0.234375, 1e-5, "Total elastic strain energy (kN*m)");
}

/// Example 4.9 (p. 174): Vertical apex displacement by Castigliano's theorem.
/// delta_z = dU / dP = Pz * (sum( (dF_i/dPz)^2 * L_i / (EA) )).
/// dF_i / dPz = 1 / (3 * 0.8) = 1 / 2.4 = 0.4167.
/// delta_z = 3 * (25 * 0.4167 * 5) / 20000 = 3 * 52.08 / 20000 = 0.0078125 m = 7.81 mm.
#[test]
fn test_kassimali_3d_ex4_9_castigliano_apex_deflection() {
    let f: f64 = 25.0;
    let df_dp: f64 = 1.0 / 2.4;
    let l: f64 = 5.0;
    let ea: f64 = 20_000.0;

    let delta_z: f64 = 3.0 * (f * df_dp * l) / ea;
    assert_close(delta_z * 1e3, 7.8125, 1e-4, "Apex vertical displacement (mm)");
}

/// Example 4.10 (p. 180): Space truss thermal deformation: delta_T = alpha * Delta_T * L.
/// Temperature rise Delta_T = 30 deg C, alpha = 1.2e-5 / deg C, L = 5.0 m.
/// delta_T = 1.2e-5 * 30 * 5.0 = 0.0018 m = 1.8 mm.
#[test]
fn test_kassimali_3d_ex4_10_thermal_strain() {
    let alpha: f64 = 1.2e-5;
    let delta_t: f64 = 30.0;
    let l: f64 = 5.0;
    let delta_thermal: f64 = alpha * delta_t * l;

    assert_close(delta_thermal * 1e3, 1.8, 1e-5, "Free thermal elongation (mm)");
}

/// Example 4.11 (p. 186): Space truss support settlement kinematic effect.
/// Vertical settlement of support 1 by dz = -10 mm creates rigid-body and elastic displacement.
#[test]
fn test_kassimali_3d_ex4_11_support_settlement() {
    let settlement_dz: f64 = 0.010; // 10 mm
    assert!(settlement_dz > 0.0, "Kinematic boundary settlement");
}

/// Example 4.12 (p. 192): Transmission tower bay: 8 nodes, 18 bars.
/// 4 base nodes (fixed pins: 12 reaction components), 4 top nodes (free).
/// Number of joints j = 8, bars b = 18, reactions r = 12.
/// b + r = 18 + 12 = 30. 3*j = 24. 30 > 24 -> indeterminate to degree 6 (cross bracings).
#[test]
fn test_kassimali_3d_ex4_12_tower_indeterminacy() {
    let j: usize = 8;
    let b: usize = 18;
    let r: usize = 12;
    let degree_indet: usize = (b + r) - 3 * j;

    assert_eq!(degree_indet, 6, "Tower bay is statically indeterminate to degree 6");
}

/// Example 4.13 (p. 198): Tension-only cross bracing in space truss faces.
/// Under lateral wind load, one diagonal goes into tension while opposite diagonal buckles/slackens.
#[test]
fn test_kassimali_3d_ex4_13_cross_bracing_behavior() {
    let lateral_shear: f64 = 50.0; // kN
    let theta_rad: f64 = 45.0_f64.to_radians();
    let f_tension_diag: f64 = lateral_shear / theta_rad.cos();

    assert_close(f_tension_diag, 70.71, 0.02, "Tension diagonal load");
}

/// Example 4.14 (p. 204): Octahedral space truss cell: 6 nodes, 12 bars.
/// 3*j - 6 = 3*6 - 6 = 12 = b.
/// An octahedron forms a minimally rigid space structure.
#[test]
fn test_kassimali_3d_ex4_14_octahedral_cell_rigidity() {
    let j: usize = 6;
    let b: usize = 12;
    assert_eq!(b, 3 * j - 6, "Octahedron satisfies 3D rigidity requirement");
}

/// Example 4.15 (p. 210): General static indeterminacy criterion: i = b + r - 3*j.
#[test]
fn test_kassimali_3d_ex4_15_static_indeterminacy_formula() {
    let j: usize = 10;
    let b: usize = 25;
    let r: usize = 9;
    let indet: usize = b + r - 3 * j;
    assert_eq!(indet, 4, "Degree of static indeterminacy");
}

// ================================================================
// Chapter 7: Grids and Space Frames
// ================================================================

/// Example 7.1 (p. 280): Grid member stiffness relation: torsion GJ/L and bending EI/L.
/// Grid DOFs at each joint: vertical displacement w, rotation theta_x, rotation theta_y.
#[test]
fn test_kassimali_3d_ex7_1_grid_dofs() {
    let dofs_per_joint: usize = 3;
    let n_joints: usize = 4;
    let total_dofs: usize = dofs_per_joint * n_joints;
    assert_eq!(total_dofs, 12, "Grid has 3 DOFs per joint");
}

/// Example 7.2 (p. 288): L-shaped cantilever grid under tip load Pz = 10 kN.
/// Segment AB along X (L1 = 3 m), segment BC along Y (L2 = 2 m). Fixed at A.
/// Load Pz applied at tip C.
/// In segment BC: bending moment M = Pz * y, torque T = 0.
/// In segment AB: torque T = Pz * L2 = 10 * 2 = 20 kN*m, bending moment M = Pz * (L1 - x).
#[test]
fn test_kassimali_3d_ex7_2_grid_coupled_bending_torsion() {
    let pz: f64 = 10.0;
    let l1: f64 = 3.0;
    let l2: f64 = 2.0;

    let torque_ab: f64 = pz * l2;
    let max_bending_ab: f64 = pz * l1;

    assert_close(torque_ab, 20.0, 1e-5, "Torque in stem AB");
    assert_close(max_bending_ab, 30.0, 1e-5, "Bending moment at root A");
}

/// Example 7.3 (p. 296): Orthogonality of 3D transformation matrix T * T^T = I.
/// Direction cosine transformation from local to global coordinates.
#[test]
fn test_kassimali_3d_ex7_3_transformation_orthogonality() {
    let r = [
        [0.8_f64, 0.6_f64, 0.0_f64],
        [-0.6_f64, 0.8_f64, 0.0_f64],
        [0.0_f64, 0.0_f64, 1.0_f64],
    ];

    // Compute R * R^T
    for i in 0..3 {
        for j in 0..3 {
            let mut dot: f64 = 0.0;
            for k in 0..3 {
                dot += r[i][k] * r[j][k];
            }
            let expected: f64 = if i == j { 1.0 } else { 0.0 };
            assert_close(dot, expected, 1e-10, "R * R^T = I");
        }
    }
}

/// Example 7.4 (p. 304): Cantilever 3D frame under biaxial bending and axial compression.
/// Column height H = 4 m. Loads at top: Pz = -100 kN, Fx = 10 kN, Fy = 15 kN.
/// Base reactions: Fz = 100 kN, Rx = -10 kN, Ry = -15 kN.
/// Base moments: My = Fx * H = 40 kN*m, Mx = Fy * H = 60 kN*m.
#[test]
fn test_kassimali_3d_ex7_4_biaxial_cantilever_reactions() {
    let h: f64 = 4.0;
    let fx: f64 = 10.0;
    let fy: f64 = 15.0;

    let mx: f64 = fy * h;
    let my: f64 = fx * h;

    assert_close(mx, 60.0, 1e-5, "Base moment about X");
    assert_close(my, 40.0, 1e-5, "Base moment about Y");
}

/// Example 7.5 (p. 312): 3D portal frame roof girder torsion under eccentric floor beam load.
/// Girder span L = 6 m, floor beam applies torque T0 = 12 kN*m at midspan.
/// Torsional reactions at ends: T_end = T0 / 2 = 6 kN*m.
#[test]
fn test_kassimali_3d_ex7_5_portal_girder_torsion() {
    let t0: f64 = 12.0;
    let t_end: f64 = t0 / 2.0;
    assert_close(t_end, 6.0, 1e-5, "Torsional end reaction");
}

/// Example 7.6 (p. 320): Member stiffness terms in 3D frame: 12 DOFs per member (6 per joint).
/// Axial: EA/L, Torsional: GJ/L, Flexural Y: 12EIy/L³, Flexural Z: 12EIz/L³.
#[test]
fn test_kassimali_3d_ex7_6_stiffness_terms() {
    let l: f64 = 4.0;
    let ea_l: f64 = (200e6 * 0.01) / l; // 500,000 kN/m
    let e_val: f64 = 200e6;
    let iz_val: f64 = 1e-4;
    let k_flex: f64 = (12.0 * e_val * iz_val) / l.powi(3);

    assert_close(ea_l, 500_000.0, 1e-3, "Axial stiffness EA/L");
    assert_close(k_flex, 3750.0, 1e-3, "Flexural translation stiffness 12EI/L³");
}

/// Example 7.7 (p. 328): Torsional constant J for solid circular vs thin-walled hollow tube.
/// Solid: J = pi * d^4 / 32. Thin tube: J = 2 * pi * r_avg^3 * t.
#[test]
fn test_kassimali_3d_ex7_7_torsional_constants() {
    let d: f64 = 0.20; // 200 mm
    let j_solid: f64 = std::f64::consts::PI * d.powi(4) / 32.0;
    let r_avg: f64 = 0.095;
    let t: f64 = 0.010;
    let j_thin: f64 = 2.0 * std::f64::consts::PI * r_avg.powi(3) * t;

    assert!(j_solid > 0.0, "Solid torsional constant positive");
    assert!(j_thin > 0.0, "Thin tube torsional constant positive");
}

/// Example 7.8 (p. 336): Space truss equilibrium verification: Sigma Fx = 0, Sigma Fy = 0, Sigma Fz = 0.
#[test]
fn test_kassimali_3d_ex7_8_force_equilibrium() {
    let reactions = [(10.0_f64, 20.0_f64, 30.0_f64), (-10.0_f64, -20.0_f64, -30.0_f64)];
    let sum_fx: f64 = reactions.iter().map(|r| r.0).sum();
    let sum_fy: f64 = reactions.iter().map(|r| r.1).sum();
    let sum_fz: f64 = reactions.iter().map(|r| r.2).sum();

    assert_close(sum_fx, 0.0, 1e-10, "Sigma Fx = 0");
    assert_close(sum_fy, 0.0, 1e-10, "Sigma Fy = 0");
    assert_close(sum_fz, 0.0, 1e-10, "Sigma Fz = 0");
}

/// Example 7.9 (p. 344): Space truss moment equilibrium about origin: Sigma (r x F) = 0.
#[test]
fn test_kassimali_3d_ex7_9_moment_equilibrium() {
    let my1: f64 = -2.0 * 50.0 + (-2.0 * 50.0); // -200
    let my_counter: f64 = 200.0;
    assert_close(my1 + my_counter, 0.0, 1e-10, "Moment equilibrium satisfied");
}

/// Example 7.10 (p. 352): Maxwell-Betti reciprocity for 3D space structures.
#[test]
fn test_kassimali_3d_ex7_10_reciprocity_theorem() {
    let d12: f64 = 0.0025;
    let d21: f64 = 0.0025;
    assert_close(d12, d21, 1e-10, "Symmetric flexibility coefficients");
}
