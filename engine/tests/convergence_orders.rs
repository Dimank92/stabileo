//! Convergence-order tests (Method of Manufactured Solutions) for shells, plates and solids (issue #380).
//!
//! From PRODUCT_ROADMAP.md § "Solver and proof depth", item 6:
//! "Convergence-order tests (method of manufactured solutions) for every shell, plate and solid
//!  element: the observed rate matches the theory."
//!
//! Theoretical rates under systematic h-refinement:
//! - Bilinear / linear elements (DKT triangle, MITC4 quad, curved shell, 8-node solid shell):
//!   asymptotic convergence rate in displacement is O(h²), i.e. p = 2.0.
//! - Biquadratic elements (MITC9 9-node quad shell):
//!   asymptotic convergence rate in displacement is higher-order O(h³) to O(h⁴), i.e. p ≥ 3.0.

use dedaliano_engine::solver::linear;
use dedaliano_engine::types::*;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElementFamily {
    Dkt,
    Mitc4,
    Mitc9,
    Curved,
    Solid,
}

const E_MODULUS: f64 = 200_000.0; // MPa
const POISSON: f64 = 0.3;

/// Computes the exact Navier analytical centre deflection for simply supported square plate.
/// `kappa`: None for Kirchhoff thin plate; Some(κ) for Mindlin thick plate with shear correction factor κ.
pub fn navier_centre(a: f64, t: f64, q0: f64, kappa: Option<f64>) -> f64 {
    let e = E_MODULUS * 1000.0; // kPa
    let d = e * t.powi(3) / (12.0 * (1.0 - POISSON * POISSON));
    let g = e / (2.0 * (1.0 + POISSON));
    let kgt = kappa.map(|k| k * g * t);

    let mut wc = 0.0;
    for m in (1..600).step_by(2) {
        for n in (1..600).step_by(2) {
            let (mf, nf) = (m as f64, n as f64);
            let a2 = PI * PI * (mf * mf / (a * a) + nf * nf / (a * a));
            let qmn = 16.0 * q0 / (PI * PI * mf * nf);
            let sign = if (m / 2 + n / 2) % 2 == 0 { 1.0 } else { -1.0 };
            let phi = if let Some(shear_stiffness) = kgt {
                1.0 + d * a2 / shear_stiffness
            } else {
                1.0
            };
            wc += sign * qmn / (d * a2 * a2) * phi;
        }
    }
    wc
}

/// Observed asymptotic convergence order p between mesh sizes h and h/2:
/// p = log2(e_coarse / e_fine).
pub fn convergence_order(e_coarse: f64, e_fine: f64) -> f64 {
    (e_coarse / e_fine).log2()
}

fn empty_input() -> SolverInput3D {
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

/// Builds a structured N × N simply supported square plate model of the given element family.
/// Uses hard Navier boundary conditions: w = 0, edge-tangential rotation θ_s = 0.
pub fn build_plate_model(fam: ElementFamily, n: usize, a: f64, t: f64, q: f64) -> (SolverInput3D, (usize, usize)) {
    let mut input = empty_input();
    input.materials.insert("1".into(), SolverMaterial { id: 1, e: E_MODULUS, nu: POISSON });

    let k = if fam == ElementFamily::Mitc9 { 2 } else { 1 };
    let mx = k * n;
    let my = k * n;
    let mut lat = vec![vec![(0usize, 0usize); my + 1]; mx + 1];
    let mut nid = 1;

    for i in 0..=mx {
        for j in 0..=my {
            let x = (i as f64 / mx as f64) * a;
            let y = (j as f64 / my as f64) * a;
            if fam == ElementFamily::Solid {
                let b = [x, y, -0.5 * t];
                let tp = [x, y, 0.5 * t];
                input.nodes.insert(nid.to_string(), SolverNode3D { id: nid, x: b[0], y: b[1], z: b[2] });
                input.nodes.insert((nid + 1).to_string(), SolverNode3D { id: nid + 1, x: tp[0], y: tp[1], z: tp[2] });
                lat[i][j] = (nid, nid + 1);
                nid += 2;
            } else {
                input.nodes.insert(nid.to_string(), SolverNode3D { id: nid, x, y, z: 0.0 });
                lat[i][j] = (nid, nid);
                nid += 1;
            }
        }
    }

    let mut eid = 1;
    let mut eids = Vec::new();
    for i in 0..n {
        for j in 0..n {
            match fam {
                ElementFamily::Dkt => {
                    let (n1, n2, n3, n4) = (lat[i][j].0, lat[i + 1][j].0, lat[i + 1][j + 1].0, lat[i][j + 1].0);
                    for nodes in [[n1, n2, n3], [n1, n3, n4]] {
                        input.plates.insert(eid.to_string(), SolverPlateElement { id: eid, nodes, material_id: 1, thickness: t });
                        eids.push(eid);
                        eid += 1;
                    }
                }
                ElementFamily::Mitc4 => {
                    let nodes = [lat[i][j].0, lat[i + 1][j].0, lat[i + 1][j + 1].0, lat[i][j + 1].0];
                    input.quads.insert(eid.to_string(), SolverQuadElement { id: eid, nodes, material_id: 1, thickness: t });
                    eids.push(eid);
                    eid += 1;
                }
                ElementFamily::Curved => {
                    let nodes = [lat[i][j].0, lat[i + 1][j].0, lat[i + 1][j + 1].0, lat[i][j + 1].0];
                    let normals = Some([[0.0, 0.0, 1.0]; 4]);
                    input.curved_shells.insert(eid.to_string(), SolverCurvedShellElement { id: eid, nodes, material_id: 1, thickness: t, normals });
                    eids.push(eid);
                    eid += 1;
                }
                ElementFamily::Mitc9 => {
                    let (a_i, b_j) = (2 * i, 2 * j);
                    let at = |di: usize, dj: usize| lat[a_i + di][b_j + dj].0;
                    let nodes = [at(0, 0), at(2, 0), at(2, 2), at(0, 2), at(1, 0), at(2, 1), at(1, 2), at(0, 1), at(1, 1)];
                    input.quad9s.insert(eid.to_string(), SolverQuad9Element { id: eid, nodes, material_id: 1, thickness: t });
                    eids.push(eid);
                    eid += 1;
                }
                ElementFamily::Solid => {
                    let c = [lat[i][j], lat[i + 1][j], lat[i + 1][j + 1], lat[i][j + 1]];
                    let nodes = [c[0].0, c[1].0, c[2].0, c[3].0, c[0].1, c[1].1, c[2].1, c[3].1];
                    input.solid_shells.insert(eid.to_string(), SolverSolidShellElement { id: eid, nodes, material_id: 1 });
                    eids.push(eid);
                    eid += 1;
                }
            }
        }
    }

    // Boundary conditions: hard Navier simple support
    let mut sid = 1;
    for i in 0..=mx {
        for j in 0..=my {
            let (xe, ye) = (i == 0 || i == mx, j == 0 || j == my);
            if xe || ye {
                let (b, tp) = lat[i][j];
                if fam == ElementFamily::Solid {
                    // For solid-shells: edge x = const has tangential along y; edge y = const has tangential along x.
                    let rx = ye || (i == 0 && j == 0);
                    let ry = xe;
                    for node_id in [b, tp] {
                        input.supports.insert(
                            sid.to_string(),
                            SolverSupport3D {
                                node_id,
                                rx,
                                ry,
                                rz: true,
                                rrx: false,
                                rry: false,
                                rrz: false,
                                kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
                                dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
                                normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
                            },
                        );
                        sid += 1;
                    }
                } else {
                    let pin = (i == 0 && j == 0, i == mx && j == 0);
                    // On edge x = const (xe), normal is x, so restrain twist rrx = true, bending rry = false.
                    // On edge y = const (ye), normal is y, so restrain twist rry = true, bending rrx = false.
                    input.supports.insert(
                        sid.to_string(),
                        SolverSupport3D {
                            node_id: b,
                            rx: pin.0,
                            ry: pin.0 || pin.1,
                            rz: true,
                            rrx: xe,
                            rry: ye,
                            rrz: false,
                            kx: None, ky: None, kz: None, krx: None, kry: None, krz: None,
                            dx: None, dy: None, dz: None, drx: None, dry: None, drz: None,
                            normal_x: None, normal_y: None, normal_z: None, is_inclined: None, rw: None, kw: None,
                        },
                    );
                    sid += 1;
                }
            }
        }
    }

    // Apply pressure load q
    for &id in &eids {
        // For solid shells, positive pressure pushes inward (downwards towards bottom face);
        // for surface shells, negative pressure along normal [0,0,1] pushes downwards.
        let pressure_val = if fam == ElementFamily::Solid { q } else { -q };
        let p = SolverPressureLoad { element_id: id, pressure: pressure_val };
        let l = match fam {
            ElementFamily::Dkt => SolverLoad3D::Pressure(p),
            ElementFamily::Mitc4 => SolverLoad3D::QuadPressure(p),
            ElementFamily::Curved => SolverLoad3D::CurvedShellPressure(p),
            ElementFamily::Mitc9 => SolverLoad3D::Quad9Pressure(p),
            ElementFamily::Solid => SolverLoad3D::SolidShellPressure(p),
        };
        input.loads.push(l);
    }

    let center_pair = lat[mx / 2][my / 2];
    (input, center_pair)
}

pub fn solve_centre_deflection(fam: ElementFamily, n: usize, a: f64, t: f64, q: f64) -> f64 {
    let (input, (b, tp)) = build_plate_model(fam, n, a, t, q);
    let r = linear::solve_3d(&input).expect("solve");
    if b == tp {
        let d = r.displacements.iter().find(|d| d.node_id == b).expect("center displacement");
        -d.uz
    } else {
        let db = r.displacements.iter().find(|d| d.node_id == b).expect("bottom displacement");
        let dt = r.displacements.iter().find(|d| d.node_id == tp).expect("top displacement");
        0.5 * (-db.uz + -dt.uz)
    }
}

/// Verification of convergence order for DKT plate elements against Navier analytical series.
#[test]
fn convergence_order_dkt_plate() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;
    let w_exact = navier_centre(a, t, q, None);

    let meshes = [4, 8, 16, 32];
    let ws: Vec<f64> = meshes.iter().map(|&n| solve_centre_deflection(ElementFamily::Dkt, n, a, t, q)).collect();
    let errs: Vec<f64> = ws.iter().map(|&w| (w - w_exact).abs()).collect();
    let orders: Vec<f64> = errs.windows(2).map(|e| convergence_order(e[0], e[1])).collect();

    eprintln!("DKT Plate ws: {ws:?}");
    eprintln!("DKT Plate errors: {errs:?}");
    eprintln!("DKT Plate observed convergence orders: {orders:?}");

    // Asymptotic convergence order for linear DKT triangle matches theoretical quadratic rate O(h^2):
    let last_order = *orders.last().unwrap();
    assert!(
        last_order >= 1.95 && last_order <= 2.15,
        "DKT plate asymptotic order p = {:.3}, expected ~2.0",
        last_order
    );
}

/// Verification of convergence order for MITC4 quadrilateral shell elements against Mindlin Navier limit.
#[test]
fn convergence_order_mitc4_quad_shell() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;
    // For MITC4 Mindlin plate, reference limit is Mindlin Navier series with kappa = 5/6:
    let w_exact = navier_centre(a, t, q, Some(5.0 / 6.0));

    let meshes = [4, 8, 16, 32];
    let ws: Vec<f64> = meshes.iter().map(|&n| solve_centre_deflection(ElementFamily::Mitc4, n, a, t, q)).collect();
    let errs: Vec<f64> = ws.iter().map(|&w| (w - w_exact).abs()).collect();
    let orders: Vec<f64> = errs.windows(2).map(|e| convergence_order(e[0], e[1])).collect();

    eprintln!("MITC4 Quad ws: {ws:?}, exact Mindlin: {w_exact}");
    eprintln!("MITC4 Quad errors: {errs:?}");
    eprintln!("MITC4 Quad observed convergence orders: {orders:?}");

    // Bilinear MITC4 quad matches theoretical quadratic convergence rate O(h^2):
    let last_order = *orders.last().unwrap();
    assert!(
        last_order >= 1.95 && last_order <= 2.15,
        "MITC4 quad asymptotic order p = {:.3}, expected ~2.0",
        last_order
    );
}

/// Verification of convergence order for 4-node curved shell elements against Mindlin limit.
#[test]
fn convergence_order_curved_shell() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;
    let w_exact = navier_centre(a, t, q, Some(5.0 / 6.0));

    let meshes = [4, 8, 16, 32];
    let ws: Vec<f64> = meshes.iter().map(|&n| solve_centre_deflection(ElementFamily::Curved, n, a, t, q)).collect();
    let errs: Vec<f64> = ws.iter().map(|&w| (w - w_exact).abs()).collect();
    let orders: Vec<f64> = errs.windows(2).map(|e| convergence_order(e[0], e[1])).collect();

    eprintln!("Curved Shell ws: {ws:?}, exact Mindlin: {w_exact}");
    eprintln!("Curved Shell errors: {errs:?}");
    eprintln!("Curved Shell observed convergence orders: {orders:?}");

    // 4-node curved shell matches theoretical quadratic convergence rate O(h^2):
    let last_order = *orders.last().unwrap();
    assert!(
        last_order >= 1.95 && last_order <= 2.15,
        "Curved shell asymptotic order p = {:.3}, expected ~2.0",
        last_order
    );
}

/// Verification of convergence order for 8-node solid shell (SHB8-ANS) elements against Mindlin limit.
#[test]
fn convergence_order_solid_shell() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;
    // For 8-node solid shell, single linear layer converges to Mindlin with kappa = 1.0:
    let w_exact = navier_centre(a, t, q, Some(1.0));

    let meshes = [4, 8, 16, 32];
    let ws: Vec<f64> = meshes.iter().map(|&n| solve_centre_deflection(ElementFamily::Solid, n, a, t, q)).collect();
    let errs: Vec<f64> = ws.iter().map(|&w| (w - w_exact).abs()).collect();
    let orders: Vec<f64> = errs.windows(2).map(|e| convergence_order(e[0], e[1])).collect();

    eprintln!("Solid Shell ws: {ws:?}, exact limit: {w_exact}");
    eprintln!("Solid Shell errors: {errs:?}");
    eprintln!("Solid Shell observed convergence orders: {orders:?}");

    // 8-node solid shell matches theoretical quadratic convergence rate O(h^2):
    let last_order = *orders.last().unwrap();
    assert!(
        last_order >= 1.95 && last_order <= 2.15,
        "Solid shell asymptotic order p = {:.3}, expected ~2.0",
        last_order
    );
}

/// Verification of convergence order for biquadratic MITC9 quadrilateral shell elements.
#[test]
fn convergence_order_mitc9_quad_shell() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;
    let w_exact = navier_centre(a, t, q, Some(5.0 / 6.0));

    let meshes = [2, 4, 8, 16];
    let ws: Vec<f64> = meshes.iter().map(|&n| solve_centre_deflection(ElementFamily::Mitc9, n, a, t, q)).collect();
    let errs: Vec<f64> = ws.iter().map(|&w| (w - w_exact).abs()).collect();
    let orders: Vec<f64> = errs.windows(2).map(|e| convergence_order(e[0], e[1])).collect();

    eprintln!("MITC9 Quad ws: {ws:?}, exact Mindlin: {w_exact}");
    eprintln!("MITC9 Quad errors: {errs:?}");
    eprintln!("MITC9 Quad observed convergence orders: {orders:?}");

    // Biquadratic 9-node elements achieve higher-order asymptotic convergence (p >= 3.0):
    let last_order = *orders.last().unwrap();
    assert!(
        last_order >= 3.0,
        "MITC9 quad asymptotic order p = {:.3}, expected >= 3.0",
        last_order
    );
}

/// Method of Manufactured Solutions (MMS) test suite covering all element families simultaneously.
#[test]
fn method_of_manufactured_solutions_suite() {
    let a = 1.0;
    let t = 0.01;
    let q = 1.0;

    // Verify all 5 element families reach expected order:
    // DKT: p ~ 2.0
    let w_k = navier_centre(a, t, q, None);
    let e_dkt_16 = (solve_centre_deflection(ElementFamily::Dkt, 16, a, t, q) - w_k).abs();
    let e_dkt_32 = (solve_centre_deflection(ElementFamily::Dkt, 32, a, t, q) - w_k).abs();
    let p_dkt = convergence_order(e_dkt_16, e_dkt_32);
    assert!((p_dkt - 2.0).abs() < 0.1, "DKT MMS rate {p_dkt:.2}");

    // MITC4: p ~ 2.0
    let w_m56 = navier_centre(a, t, q, Some(5.0 / 6.0));
    let e_m4_16 = (solve_centre_deflection(ElementFamily::Mitc4, 16, a, t, q) - w_m56).abs();
    let e_m4_32 = (solve_centre_deflection(ElementFamily::Mitc4, 32, a, t, q) - w_m56).abs();
    let p_m4 = convergence_order(e_m4_16, e_m4_32);
    assert!((p_m4 - 2.0).abs() < 0.1, "MITC4 MMS rate {p_m4:.2}");

    // Curved shell: p ~ 2.0
    let e_cs_16 = (solve_centre_deflection(ElementFamily::Curved, 16, a, t, q) - w_m56).abs();
    let e_cs_32 = (solve_centre_deflection(ElementFamily::Curved, 32, a, t, q) - w_m56).abs();
    let p_cs = convergence_order(e_cs_16, e_cs_32);
    assert!((p_cs - 2.0).abs() < 0.1, "Curved shell MMS rate {p_cs:.2}");

    // Solid shell: p ~ 2.0
    let w_m1 = navier_centre(a, t, q, Some(1.0));
    let e_ss_16 = (solve_centre_deflection(ElementFamily::Solid, 16, a, t, q) - w_m1).abs();
    let e_ss_32 = (solve_centre_deflection(ElementFamily::Solid, 32, a, t, q) - w_m1).abs();
    let p_ss = convergence_order(e_ss_16, e_ss_32);
    assert!((p_ss - 2.0).abs() < 0.1, "Solid shell MMS rate {p_ss:.2}");

    // MITC9: p >= 3.0
    let e_m9_8 = (solve_centre_deflection(ElementFamily::Mitc9, 8, a, t, q) - w_m56).abs();
    let e_m9_16 = (solve_centre_deflection(ElementFamily::Mitc9, 16, a, t, q) - w_m56).abs();
    let p_m9 = convergence_order(e_m9_8, e_m9_16);
    assert!(p_m9 >= 3.0, "MITC9 MMS rate {p_m9:.2}");
}

