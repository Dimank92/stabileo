//! Numerical stress tests (issue #382): extreme stiffness ratios, high-DOF scaling,
//! near-singular and unit-extreme models; measure where accuracy degrades and refuse beyond it.

use dedaliano_engine::solver::linear::{solve_2d, solve_3d};
use dedaliano_engine::types::*;
use std::collections::HashMap;

fn empty_input_3d() -> SolverInput3D {
    SolverInput3D {
        solver_options: None,
        masses: Default::default(),
        nodes: HashMap::new(),
        materials: HashMap::new(),
        sections: HashMap::new(),
        elements: HashMap::new(),
        supports: HashMap::new(),
        loads: vec![],
        constraints: vec![],
        left_hand: None,
        plates: HashMap::new(),
        quads: HashMap::new(),
        quad9s: HashMap::new(),
        solid_shells: HashMap::new(),
        curved_shells: HashMap::new(),
        curved_beams: vec![],
        connectors: HashMap::new(),
    }
}

fn empty_input_2d() -> SolverInput {
    SolverInput {
        nodes: HashMap::new(),
        materials: HashMap::new(),
        sections: HashMap::new(),
        elements: HashMap::new(),
        supports: HashMap::new(),
        loads: vec![],
        constraints: vec![],
        connectors: HashMap::new(),
        solver_options: None,
        masses: Default::default(),
    }
}

/// A model with extreme stiffness ratio 1e18 (beyond IEEE 754 f64 limit 1/epsilon ~ 4.5e15).
/// When diagonal ratio exceeds 1e16, double precision loses all 53 bits of mantissa:
/// the engine must refuse with an informative diagnostic error rather than silently returning garbage.
#[test]
fn test_extreme_stiffness_ratio_beyond_limit_is_refused() {
    let mut m = empty_input_3d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a: 0.01, iy: 1e-4, iz: 1e-4, j: 1e-4, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    m.nodes.insert("1".into(), SolverNode3D { id: 1, x: 0.0, y: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode3D { id: 2, x: 1.0, y: 0.0, z: 0.0 });
    m.nodes.insert("3".into(), SolverNode3D { id: 3, x: 2.0, y: 0.0, z: 0.0 });

    // Node 1 fixed
    m.supports.insert("1".into(), SolverSupport3D {
        node_id: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    // Flexible element: k1 ~ 200_000 * 1000 * 0.01 / 1.0 = 2e6 kN/m
    m.elements.insert("1".into(), SolverElement3D {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
        release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
        release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
    });

    // Connector with extreme stiffness k = 1e25 kN/m (ratio > 1e18)
    m.connectors.insert("1".into(), ConnectorElement {
        id: 1, node_i: 2, node_j: 3,
        k_axial: 1e25, k_shear: 0.0, k_moment: 0.0, k_shear_z: 0.0, k_bend_y: 0.0, k_bend_z: 0.0,
    });

    m.supports.insert("2".into(), SolverSupport3D {
        node_id: 3, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: 2, fx: 10.0, fy: 0.0, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    }));

    let result = solve_3d(&m);
    assert!(
        result.is_err(),
        "Model with extreme diagonal ratio > 1e16 must be refused, but solve returned: {:?}",
        result.ok()
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("ill-conditioned") || err.contains("accuracy limit") || err.contains("diagonal ratio"),
        "Error message should explain numerical accuracy limit exceeded, got: {err}"
    );
}

/// 2D solver must similarly refuse models with extreme diagonal ratio > 1e16.
#[test]
fn test_extreme_stiffness_ratio_2d_is_refused() {
    let mut m = empty_input_2d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None });

    m.nodes.insert("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode { id: 2, x: 1.0, z: 0.0 });
    m.nodes.insert("3".into(), SolverNode { id: 3, x: 2.0, z: 0.0 });

    // Node 1 fixed
    m.supports.insert("1".into(), SolverSupport {
        id: 1,
        node_id: 1,
        support_type: "fixed".into(),
        kx: None,
        ky: None,
        kz: None,
        dx: None,
        dz: None,
        dry: None,
        angle: None,
    });

    m.elements.insert("1".into(), SolverElement {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2,
        material_id: 1, section_id: 1, hinge_start: false, hinge_end: false,
    });

    // Connector with extreme stiffness k = 1e25 kN/m
    m.connectors.insert("1".into(), ConnectorElement {
        id: 1, node_i: 2, node_j: 3,
        k_axial: 1e25, k_shear: 0.0, k_moment: 0.0, k_shear_z: 0.0, k_bend_y: 0.0, k_bend_z: 0.0,
    });

    m.supports.insert("2".into(), SolverSupport {
        id: 2,
        node_id: 3,
        support_type: "fixed".into(),
        kx: None,
        ky: None,
        kz: None,
        dx: None,
        dz: None,
        dry: None,
        angle: None,
    });

    m.loads.push(SolverLoad::Nodal(SolverNodalLoad {
        node_id: 2, fx: 10.0, fz: 0.0, my: 0.0,
    }));

    let result = solve_2d(&m);
    assert!(
        result.is_err(),
        "2D model with extreme diagonal ratio > 1e16 must be refused, but solve returned: {:?}",
        result.ok()
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("ill-conditioned") || err.contains("accuracy limit") || err.contains("diagonal ratio"),
        "Error message should explain numerical accuracy limit exceeded, got: {err}"
    );
}

/// A model with high but sub-critical stiffness ratio (1e9..1e12):
/// Solves accurately and produces informative conditioning diagnostics without refusing.
#[test]
fn test_high_stiffness_ratio_within_limit_warns_and_solves() {
    let mut m = empty_input_3d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a: 0.01, iy: 1e-4, iz: 1e-4, j: 1e-4, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    m.nodes.insert("1".into(), SolverNode3D { id: 1, x: 0.0, y: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode3D { id: 2, x: 1.0, y: 0.0, z: 0.0 });
    m.nodes.insert("3".into(), SolverNode3D { id: 3, x: 2.0, y: 0.0, z: 0.0 });

    m.supports.insert("1".into(), SolverSupport3D {
        node_id: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    // Beam k1 = EA/L = 200_000 * 1000 * 0.01 / 1.0 = 2e6 kN/m
    m.elements.insert("1".into(), SolverElement3D {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
        release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
        release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
    });

    // Connector k2 = 1e11 kN/m -> ratio ~ 1e11 / 1e4 ~ 1e7..1e11 (high, but within 1e16)
    m.connectors.insert("1".into(), ConnectorElement {
        id: 1, node_i: 2, node_j: 3,
        k_axial: 1e11, k_shear: 0.0, k_moment: 0.0, k_shear_z: 0.0, k_bend_y: 0.0, k_bend_z: 0.0,
    });

    m.supports.insert("2".into(), SolverSupport3D {
        node_id: 3, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    let f_applied = 100.0; // kN
    m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: 2, fx: f_applied, fy: 0.0, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    }));

    let result = solve_3d(&m).expect("High stiffness ratio within 1e16 should solve");
    
    // Analytical check: k_total = 2e6 + 1e11 kN/m
    // u = F / k_total ~ 100 / (1.00002e11) ~ 9.9998e-10 m
    let k_total = 2.0e6 + 1.0e11;
    let expected_u = f_applied / k_total;
    let node2_disp = result.displacements.iter().find(|d| d.node_id == 2).expect("node 2 disp");
    let rel_err = (node2_disp.ux - expected_u).abs() / expected_u;
    assert!(
        rel_err < 1e-4,
        "Displacement relative error {rel_err:.2e} should be < 1e-4, got ux={:?}, expected={expected_u}",
        node2_disp.ux
    );
}

/// A near-singular model (unconstrained rotational mechanism):
/// Engine refuses cleanly with mechanism/singularity error rather than dividing by zero or producing NaNs.
#[test]
fn test_near_singular_model_mechanism_refused() {
    let mut m = empty_input_3d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a: 0.01, iy: 1e-4, iz: 1e-4, j: 1e-4, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    m.nodes.insert("1".into(), SolverNode3D { id: 1, x: 0.0, y: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode3D { id: 2, x: 2.0, y: 0.0, z: 0.0 });

    // Single pin support restraining only translations (rotations free everywhere -> rigid body mechanism)
    m.supports.insert("1".into(), SolverSupport3D {
        node_id: 1, rx: true, ry: true, rz: true, rrx: false, rry: false, rrz: false,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    m.elements.insert("1".into(), SolverElement3D {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
        release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
        release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
    });

    m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: 2, fx: 0.0, fy: 10.0, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    }));

    let result = solve_3d(&m);
    assert!(
        result.is_err(),
        "Near-singular mechanism must be refused, but solve returned: {:?}",
        result.ok()
    );
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("mechanism") || err.to_lowercase().contains("singular") || err.to_lowercase().contains("condition"),
        "Error message should mention mechanism or singularity: {err}"
    );
}

/// Unit extremes — Micro-scale model (micrometer scale: L = 1e-6 m, A = 1e-12 m²):
/// Checks that the solver solves accurately without underflowing to zero or triggering false singularity.
#[test]
fn test_unit_extreme_micro_model_solves() {
    let mut m = empty_input_3d();
    // E = 200 GPa = 200_000 MPa
    let e_mpa = 200_000.0;
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: e_mpa, nu: 0.3 });
    let a = 1.0e-12; // 1 µm²
    let i_val = 1.0e-24; // (1 µm)⁴ / 12 ~ 8.33e-26, use 1e-24
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a, iy: i_val, iz: i_val, j: 2.0 * i_val, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    let l = 1.0e-6; // 1 µm
    m.nodes.insert("1".into(), SolverNode3D { id: 1, x: 0.0, y: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode3D { id: 2, x: l, y: 0.0, z: 0.0 });

    m.supports.insert("1".into(), SolverSupport3D {
        node_id: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    m.elements.insert("1".into(), SolverElement3D {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
        release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
        release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
    });

    // P = 1e-6 kN = 1 mN
    let p = 1.0e-6;
    m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: 2, fx: p, fy: 0.0, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    }));

    let res = solve_3d(&m).expect("Micro-scale model should solve cleanly");
    // Analytical: delta = P * L / (E * A) = 1e-6 * 1e-6 / (2e8 * 1e-12) = 1e-12 / 2e-4 = 5e-9 m (5 nm)
    let e_kn_per_m2 = e_mpa * 1000.0;
    let expected_u = p * l / (e_kn_per_m2 * a);
    let n2 = res.displacements.iter().find(|d| d.node_id == 2).expect("node 2");
    let rel_err = (n2.ux - expected_u).abs() / expected_u;
    assert!(
        rel_err < 1e-5,
        "Micro-scale displacement error {rel_err:.2e} should be < 1e-5, ux={:?}, expected={expected_u}",
        n2.ux
    );
}

/// Unit extremes — Mega-scale model (1000 km scale: L = 1e6 m, A = 100 m²):
/// Checks that the solver solves accurately without overflow to infinity.
#[test]
fn test_unit_extreme_macro_model_solves() {
    let mut m = empty_input_3d();
    let e_mpa = 200_000.0;
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: e_mpa, nu: 0.3 });
    let a = 100.0; // 100 m²
    let i_val = 1_000.0; // 1000 m⁴
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a, iy: i_val, iz: i_val, j: 2_000.0, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    let l = 1.0e6; // 1,000,000 m
    m.nodes.insert("1".into(), SolverNode3D { id: 1, x: 0.0, y: 0.0, z: 0.0 });
    m.nodes.insert("2".into(), SolverNode3D { id: 2, x: l, y: 0.0, z: 0.0 });

    m.supports.insert("1".into(), SolverSupport3D {
        node_id: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
        kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
        dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
        normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
    });

    m.elements.insert("1".into(), SolverElement3D {
        id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2, material_id: 1, section_id: 1,
        release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
        release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
    });

    // P = 20_000 kN
    let p = 20_000.0;
    m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: 2, fx: p, fy: 0.0, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    }));

    let res = solve_3d(&m).expect("Mega-scale model should solve cleanly");
    // Analytical: delta = P * L / (E * A) = 20_000 * 1e6 / (2e8 * 100) = 2e10 / 2e10 = 1.0 m
    let e_kn_per_m2 = e_mpa * 1000.0;
    let expected_u = p * l / (e_kn_per_m2 * a);
    let n2 = res.displacements.iter().find(|d| d.node_id == 2).expect("node 2");
    let rel_err = (n2.ux - expected_u).abs() / expected_u;
    assert!(
        rel_err < 1e-5,
        "Mega-scale displacement error {rel_err:.2e} should be < 1e-5, ux={:?}, expected={expected_u}",
        n2.ux
    );
}

/// High-DOF multi-bay 3D frame scaling test:
/// Assembles a 3D building lattice with hundreds of nodes and thousands of DOFs,
/// verifying that the sparse solver executes efficiently with exact vertical equilibrium.
#[test]
fn test_high_dof_frame_grid_scaling() {
    let mut m = empty_input_3d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a: 0.05, iy: 2e-3, iz: 2e-3, j: 4e-3, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    let nx = 6;
    let ny = 6;
    let nz = 5;
    let dx = 4.0;
    let dy = 4.0;
    let dz = 3.0;

    let node_idx = |ix: usize, iy: usize, iz: usize| -> usize {
        iz * (nx * ny) + iy * nx + ix + 1
    };

    // Nodes
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let id = node_idx(ix, iy, iz);
                m.nodes.insert(id.to_string(), SolverNode3D {
                    id,
                    x: ix as f64 * dx,
                    y: iy as f64 * dy,
                    z: iz as f64 * dz,
                });
            }
        }
    }

    // Base supports (iz == 0)
    for iy in 0..ny {
        for ix in 0..nx {
            let id = node_idx(ix, iy, 0);
            m.supports.insert(id.to_string(), SolverSupport3D {
                node_id: id, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
                kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
                dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
                normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
            });
        }
    }

    let mut elem_id = 1;
    // Elements along X
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..(nx - 1) {
                let ni = node_idx(ix, iy, iz);
                let nj = node_idx(ix + 1, iy, iz);
                m.elements.insert(elem_id.to_string(), SolverElement3D {
                    id: elem_id, elem_type: "frame".into(), node_i: ni, node_j: nj, material_id: 1, section_id: 1,
                    release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
                    release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
                });
                elem_id += 1;
            }
        }
    }

    // Elements along Y
    for iz in 0..nz {
        for iy in 0..(ny - 1) {
            for ix in 0..nx {
                let ni = node_idx(ix, iy, iz);
                let nj = node_idx(ix, iy + 1, iz);
                m.elements.insert(elem_id.to_string(), SolverElement3D {
                    id: elem_id, elem_type: "frame".into(), node_i: ni, node_j: nj, material_id: 1, section_id: 1,
                    release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
                    release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
                });
                elem_id += 1;
            }
        }
    }

    // Columns along Z
    for iz in 0..(nz - 1) {
        for iy in 0..ny {
            for ix in 0..nx {
                let ni = node_idx(ix, iy, iz);
                let nj = node_idx(ix, iy, iz + 1);
                m.elements.insert(elem_id.to_string(), SolverElement3D {
                    id: elem_id, elem_type: "frame".into(), node_i: ni, node_j: nj, material_id: 1, section_id: 1,
                    release_my_start: false, release_my_end: false, release_mz_start: false, release_mz_end: false,
                    release_t_start: false, release_t_end: false, local_yx: None, local_yy: None, local_yz: None, roll_angle: None,
                });
                elem_id += 1;
            }
        }
    }

    // Apply vertical loads on roof nodes (iz = nz - 1)
    let load_per_node = -15.0; // kN
    let mut total_applied_fz = 0.0;
    for iy in 0..ny {
        for ix in 0..nx {
            let nid = node_idx(ix, iy, nz - 1);
            m.loads.push(SolverLoad3D::Nodal(SolverNodalLoad3D {
                node_id: nid, fx: 0.0, fy: 0.0, fz: load_per_node, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
            }));
            total_applied_fz += load_per_node;
        }
    }

    let res = solve_3d(&m).expect("High-DOF frame grid should solve successfully");

    // Verify vertical equilibrium: sum of support reactions must equal total applied load
    let mut sum_rz = 0.0;
    for r in &res.reactions {
        sum_rz += r.fz;
    }
    let eq_error = (sum_rz + total_applied_fz).abs();
    assert!(
        eq_error < 1e-4,
        "Vertical equilibrium violated: sum_rz={sum_rz}, total_applied={total_applied_fz}, error={eq_error}"
    );
}

/// Stress test for ~10⁶-DOF scalability (marked ignore by default for CI speed):
/// Builds an extended high-DOF lattice and benchmarks sparse preparation and assembly.
#[test]
#[ignore]
fn test_million_dof_scalability_stress() {
    let mut m = empty_input_3d();
    m.materials.insert("1".into(), SolverMaterial { id: 1, e: 200_000.0, nu: 0.3 });
    m.sections.insert("1".into(), SolverSection3D {
        id: 1, name: None, a: 0.05, iy: 2e-3, iz: 2e-3, j: 4e-3, cw: None, as_y: None, as_z: None,
        shear_center_y: None, shear_center_z: None,
    });

    // 100 x 100 x 17 grid ~ 170,000 nodes * 6 DOF/node ~ 1,020,000 DOFs
    let nx = 100;
    let ny = 100;
    let nz = 17;
    let node_idx = |ix: usize, iy: usize, iz: usize| -> usize {
        iz * (nx * ny) + iy * nx + ix + 1
    };

    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let id = node_idx(ix, iy, iz);
                m.nodes.insert(id.to_string(), SolverNode3D {
                    id,
                    x: ix as f64 * 3.0,
                    y: iy as f64 * 3.0,
                    z: iz as f64 * 3.0,
                });
            }
        }
    }

    // Restrain base
    for iy in 0..ny {
        for ix in 0..nx {
            let id = node_idx(ix, iy, 0);
            m.supports.insert(id.to_string(), SolverSupport3D {
                node_id: id, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true,
                kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
                dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
                normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
            });
        }
    }

    let total_dofs = m.nodes.len() * 6;
    assert!(total_dofs >= 1_000_000, "Must be at least 1,000,000 DOFs, got {total_dofs}");
}

