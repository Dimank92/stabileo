//! Exact mechanism detection (engine/src/solver/mechanism.rs).
//!
//! Each mechanism the check exists for is refused by the linear solvers with
//! a message naming the nodes that move, and the same model made stable is
//! solved. The kinematic conditions are checked against the assembled
//! stiffness: every free motion the check reports is a zero-energy mode of K
//! (K·v ≈ 0), on random models exercising every feature. No fixture is
//! flagged.

mod fuzz_models;

use dedaliano_engine::solver::assembly::{
    assemble_stiffness_2d, assemble_stiffness_3d, inclined_rotation_matrix, inclined_rotation_matrix_2d,
};
use dedaliano_engine::solver::dof::DofNumbering;
use dedaliano_engine::solver::linear::{solve_2d, solve_3d};
use dedaliano_engine::solver::mechanism::{check_2d, check_3d, free_motion_vectors};
use dedaliano_engine::types::*;
use serde_json::{json, Value};

fn in2(v: Value) -> SolverInput {
    serde_json::from_value(v).unwrap()
}
fn in3(v: Value) -> SolverInput3D {
    serde_json::from_value(v).unwrap()
}

fn refused_2d(input: &SolverInput) -> String {
    match solve_2d(input) {
        Err(e) => e,
        Ok(_) => panic!("mechanism solved: {:?}", check_2d(input, &input.constraints)),
    }
}
fn refused_3d(input: &SolverInput3D) -> String {
    match solve_3d(input) {
        Err(e) => e,
        Ok(_) => panic!("mechanism solved: {:?}", check_3d(input, &input.constraints)),
    }
}
fn assert_mechanism(msg: &str, nodes: &[usize], what: &str) {
    assert!(msg.contains("Structure is a mechanism"), "{msg}");
    for n in nodes {
        assert!(msg.contains(&n.to_string()), "node {n} not named: {msg}");
    }
    assert!(msg.contains(what), "expected '{what}' in: {msg}");
}

const MAT: &str = r#"{"1": {"id": 1, "e": 200000.0, "nu": 0.3}}"#;

fn mats() -> Value {
    serde_json::from_str(MAT).unwrap()
}
fn sec2() -> Value {
    json!({"1": {"id": 1, "a": 0.01, "iz": 1e-4}})
}
fn sec3() -> Value {
    json!({"1": {"id": 1, "a": 0.01, "iy": 2e-4, "iz": 1e-4, "j": 5e-5}})
}
fn frame2(id: usize, i: usize, j: usize, hs: bool, he: bool) -> Value {
    json!({"id": id, "type": "frame", "nodeI": i, "nodeJ": j, "materialId": 1, "sectionId": 1,
           "hingeStart": hs, "hingeEnd": he})
}
fn sup2(id: usize, node: usize, t: &str) -> Value {
    json!({"id": id, "nodeId": node, "type": t})
}
fn nodal2(node: usize, fx: f64, fz: f64, my: f64) -> Value {
    json!({"type": "nodal", "data": {"nodeId": node, "fx": fx, "fz": fz, "my": my}})
}
fn sup3(node: usize, f: [bool; 6]) -> Value {
    json!({"nodeId": node, "rx": f[0], "ry": f[1], "rz": f[2], "rrx": f[3], "rry": f[4], "rrz": f[5]})
}
fn nodal3(node: usize, f: [f64; 6]) -> Value {
    json!({"type": "nodal", "data": {"nodeId": node, "fx": f[0], "fy": f[1], "fz": f[2],
           "mx": f[3], "my": f[4], "mz": f[5]}})
}

// ---------------------------------------------------------------------------
// The mechanisms
// ---------------------------------------------------------------------------

/// A member hinged at the tip of a cantilever and free at its other end
/// swings about the hinge. Fixed at its far end instead, it is a propped
/// structure and solves.
#[test]
fn dangling_hinged_member_2d() {
    let model = |far: Option<&str>| {
        let mut sups = json!({"1": sup2(1, 1, "fixed")});
        if let Some(t) = far {
            sups["2"] = sup2(2, 3, t);
        }
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "z": 0.0},
                      "3": {"id": 3, "x": 5.0, "z": 1.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, false), "2": frame2(2, 2, 3, true, false)},
            "supports": sups, "loads": [nodal2(3, 0.0, -10.0, 0.0)]
        }))
    };
    let msg = refused_2d(&model(None));
    assert_mechanism(&msg, &[3], "node 3 can move freely");
    assert!(!msg.contains("node 1:"), "{msg}");
    solve_2d(&model(Some("rollerX"))).expect("propped member solves");
}

/// The same in 3D (the differential-testing case rich3d-184): a member
/// released about its local y at the tip of another, free at its far end.
#[test]
fn dangling_released_member_3d() {
    let model = |release: bool| {
        in3(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "y": 0.0, "z": 0.0},
                      "3": {"id": 3, "x": 4.0, "y": 1.0, "z": 2.0}},
            "materials": mats(), "sections": sec3(),
            "elements": {"1": {"id": 1, "type": "frame", "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1},
                         "2": {"id": 2, "type": "frame", "nodeI": 2, "nodeJ": 3, "materialId": 1, "sectionId": 1,
                               "releaseMyStart": release}},
            "supports": {"1": sup3(1, [true; 6])},
            "loads": [nodal3(3, [0.0, 0.0, -10.0, 0.0, 0.0, 0.0])]
        }))
    };
    let msg = refused_3d(&model(true));
    assert_mechanism(&msg, &[3], "node 3 can move freely");
    solve_3d(&model(false)).expect("unreleased cantilever solves");
}

/// A portal whose beam is pinned to both columns, on pinned bases, sways
/// (the columns are links pinned at both ends). On fixed bases the columns
/// are cantilevers and it stands.
#[test]
fn portal_with_pinned_beam_column_joints_2d() {
    let model = |base: &str| {
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 0.0, "z": 4.0},
                      "3": {"id": 3, "x": 6.0, "z": 4.0}, "4": {"id": 4, "x": 6.0, "z": 0.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, false), "2": frame2(2, 2, 3, true, true),
                         "3": frame2(3, 4, 3, false, false)},
            "supports": {"1": sup2(1, 1, base), "2": sup2(2, 4, base)},
            "loads": [nodal2(2, 5.0, -10.0, 0.0)]
        }))
    };
    let msg = refused_2d(&model("pinned"));
    assert_mechanism(&msg, &[2, 3], "move freely together");
    let c = check_2d(&model("pinned"), &[]);
    assert_eq!(c.free_motions, 1, "{c:?}");
    solve_2d(&model("fixed")).expect("portal on fixed bases solves");
}

/// A plate restrained out of plane along its edges with one corner pinned
/// turns freely in its own plane: a rigid-body rotation about the normal
/// through that corner. A second corner held in plane stops it.
#[test]
fn plate_free_to_turn_in_its_plane() {
    let mut nodes = serde_json::Map::new();
    let mut id = 0;
    for j in 0..3 {
        for i in 0..3 {
            id += 1;
            nodes.insert(id.to_string(), json!({"id": id, "x": i as f64, "y": j as f64, "z": 0.0}));
        }
    }
    let mut quads = serde_json::Map::new();
    let mut q = 0;
    for j in 0..2 {
        for i in 0..2 {
            q += 1;
            let n = |a: usize, b: usize| 1 + a + 3 * b;
            quads.insert(q.to_string(), json!({"id": q, "nodes": [n(i, j), n(i + 1, j), n(i + 1, j + 1), n(i, j + 1)],
                                               "materialId": 1, "thickness": 0.1}));
        }
    }
    let model = |held: bool| {
        let mut supports = serde_json::Map::new();
        for n in [1, 2, 3, 4, 6, 7, 8, 9] {
            supports.insert(n.to_string(), sup3(n, [n == 1, n == 1 || (held && n == 3), true, false, false, false]));
        }
        in3(json!({
            "nodes": nodes.clone(), "materials": mats(), "sections": {}, "elements": {},
            "supports": supports, "quads": quads.clone(),
            "loads": [nodal3(5, [0.0, 0.0, -10.0, 0.0, 0.0, 0.0])]
        }))
    };
    let msg = refused_3d(&model(false));
    assert_mechanism(&msg, &[1], "rigid-body motion of the part containing node 1");
    assert!(msg.contains("rotation about an axis along (0, 0, 1) through (0, 0, 0)"), "{msg}");
    solve_3d(&model(true)).expect("plate held in plane solves");
}

/// Four columns tied at the top by a rigid diaphragm. The slave columns
/// are pinned at the base (torsion held); the master column is fixed, so its
/// torsion holds the diaphragm's rotation — unless that torsion is released
/// at its base, and then the floor turns freely about the vertical.
#[test]
fn constrained_diaphragm_with_a_free_rotation() {
    let model = |release_torsion: bool| {
        let mut nodes = serde_json::Map::new();
        let mut elements = serde_json::Map::new();
        let mut supports = serde_json::Map::new();
        let pts = [(0.0, 0.0), (6.0, 0.0), (6.0, 4.0), (0.0, 4.0), (3.0, 2.0)];
        for (k, (x, y)) in pts.iter().enumerate() {
            let (b, t) = (k + 1, k + 11);
            nodes.insert(b.to_string(), json!({"id": b, "x": x, "y": y, "z": 0.0}));
            nodes.insert(t.to_string(), json!({"id": t, "x": x, "y": y, "z": 3.0}));
            let master = k == 4;
            elements.insert(b.to_string(), json!({"id": b, "type": "frame", "nodeI": b, "nodeJ": t,
                "materialId": 1, "sectionId": 1, "releaseTStart": master && release_torsion}));
            supports.insert(b.to_string(), sup3(b, [true, true, true, master, master, true]));
        }
        let mut m = in3(json!({
            "nodes": nodes, "materials": mats(), "sections": sec3(), "elements": elements,
            "supports": supports, "loads": [nodal3(11, [10.0, 5.0, 0.0, 0.0, 0.0, 0.0])]
        }));
        m.constraints = vec![Constraint::Diaphragm(DiaphragmConstraint {
            master_node: 15,
            slave_nodes: vec![11, 12, 13, 14],
            plane: "XY".into(),
        })];
        m
    };
    let msg = refused_3d(&model(true));
    assert_mechanism(&msg, &[11, 12, 13, 14, 15], "rotation about an axis along (0, 0, 1) through (3, 2,");
    solve_3d(&model(false)).expect("diaphragm held by the master column's torsion solves");
}

/// A node hung from a cantilever by a connector: with an axial spring only
/// it moves freely across the connector; with axial, shear and rotational
/// springs it is held.
#[test]
fn connector_only_connection() {
    let model = |full: bool| {
        let k = if full { 1e5 } else { 0.0 };
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "z": 0.0},
                      "3": {"id": 3, "x": 4.0, "z": 0.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, false)},
            "supports": {"1": sup2(1, 1, "fixed")},
            "connectors": {"1": {"id": 1, "nodeI": 2, "nodeJ": 3, "kAxial": 1e5, "kShear": k, "kMoment": k}},
            "loads": [nodal2(3, 1.0, -10.0, 0.0)]
        }))
    };
    let msg = refused_2d(&model(false));
    assert_mechanism(&msg, &[3], "node 3");
    solve_2d(&model(true)).expect("fully sprung connector solves");
}

/// A beam on two inclined rollers with the same slope slides along it. With
/// the slopes different it is still a mechanism (it turns about the point
/// where the normals meet); one pinned end holds it.
#[test]
fn inclined_rollers_with_a_free_motion_along_the_slope() {
    let model = |a1: f64, a2: Option<f64>| {
        let s2 = match a2 {
            Some(a) => json!({"id": 2, "nodeId": 2, "type": "inclinedRoller", "angle": a}),
            None => sup2(2, 2, "pinned"),
        };
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 5.0, "z": 0.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, false)},
            "supports": {"1": {"id": 1, "nodeId": 1, "type": "inclinedRoller", "angle": a1}, "2": s2},
            "loads": [nodal2(1, 0.0, -10.0, 0.0)]
        }))
    };
    let a = 30f64.to_radians();
    let msg = refused_2d(&model(a, Some(a)));
    assert_mechanism(&msg, &[1, 2], "translation along");
    // the restrained direction is (sin a, cos a): the free one is (cos a, −sin a)
    assert!(msg.contains("(0.866, -0.5)") || msg.contains("(-0.866, 0.5)"), "{msg}");
    let msg = refused_2d(&model(a, Some(-a)));
    assert_mechanism(&msg, &[1, 2], "rotation about the point");
    solve_2d(&model(a, None)).expect("pinned at one end solves");
}

/// 3D: a beam whose only support is an inclined one (normal (0, 1, 1)) with
/// every rotation held translates freely in the plane normal to it.
#[test]
fn inclined_support_3d_free_in_its_plane() {
    let model = |inclined: bool| {
        let mut s = sup3(1, [true; 6]);
        if inclined {
            s["isInclined"] = json!(true);
            s["normalX"] = json!(0.0);
            s["normalY"] = json!(1.0);
            s["normalZ"] = json!(1.0);
        }
        in3(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 4.0, "y": 0.0, "z": 0.0}},
            "materials": mats(), "sections": sec3(),
            "elements": {"1": {"id": 1, "type": "frame", "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1}},
            "supports": {"1": s}, "loads": [nodal3(2, [0.0, 0.0, -10.0, 0.0, 0.0, 0.0])]
        }))
    };
    let msg = refused_3d(&model(true));
    assert_mechanism(&msg, &[1, 2], "2 independent free motions");
    solve_3d(&model(false)).expect("fixed cantilever solves");
}

/// A plane truss modelled in 3D with nothing holding it out of its plane:
/// its free nodes move normal to it. Held out of plane at every node it
/// solves.
#[test]
fn truss_nodes_free_out_of_plane() {
    let model = |held: bool| {
        let pts = [(0.0, 0.0), (4.0, 0.0), (8.0, 0.0), (2.0, 3.0), (6.0, 3.0)];
        let mut nodes = serde_json::Map::new();
        for (k, (x, z)) in pts.iter().enumerate() {
            nodes.insert((k + 1).to_string(), json!({"id": k + 1, "x": x, "y": 0.0, "z": z}));
        }
        let bars = [(1, 2), (2, 3), (1, 4), (2, 4), (2, 5), (3, 5), (4, 5)];
        let mut elements = serde_json::Map::new();
        for (k, (i, j)) in bars.iter().enumerate() {
            elements.insert((k + 1).to_string(), json!({"id": k + 1, "type": "truss", "nodeI": i, "nodeJ": j,
                                                       "materialId": 1, "sectionId": 1}));
        }
        let mut supports = serde_json::Map::new();
        supports.insert("1".into(), sup3(1, [true, true, true, false, false, false]));
        supports.insert("3".into(), sup3(3, [false, true, true, false, false, false]));
        if held {
            for n in [2, 4, 5] {
                supports.insert(n.to_string(), sup3(n, [false, true, false, false, false, false]));
            }
        }
        in3(json!({
            "nodes": nodes, "materials": mats(), "sections": sec3(), "elements": elements,
            "supports": supports, "loads": [nodal3(4, [0.0, 0.0, -10.0, 0.0, 0.0, 0.0])]
        }))
    };
    let msg = refused_3d(&model(false));
    assert_mechanism(&msg, &[2, 4, 5], "3 independent free motions");
    solve_3d(&model(true)).expect("truss held out of plane solves");
}

/// A joint where every member is hinged is held in translation by the
/// members and its rotation is free but harmless: the model solves. A moment
/// applied at that joint would not reach the structure: refused.
#[test]
fn all_hinged_joint_with_a_moment() {
    let model = |m: f64| {
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "z": 0.0},
                      "3": {"id": 3, "x": 6.0, "z": 0.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, true), "2": frame2(2, 2, 3, true, false)},
            "supports": {"1": sup2(1, 1, "fixed"), "2": sup2(2, 3, "fixed")},
            "loads": [nodal2(2, 0.0, -10.0, m)]
        }))
    };
    solve_2d(&model(0.0)).expect("hinged joint without a moment solves");
    // a small moment used to slip through (the artificial rotation stayed
    // under the 100 rad heuristic) and was lost
    let e = solve_2d(&model(1e-3)).unwrap_err();
    assert!(e.contains("Nodal moment at node 2") && e.contains("would not reach the structure"), "{e}");
}

/// 3D, about an axis that is not a global one: two inclined members meet at
/// a node, each released in bending there but not in torsion. Their torsion
/// resists moments in the plane of their axes; about the normal to that
/// plane nothing does.
#[test]
fn released_joint_moment_about_a_skew_axis() {
    let model = |m: [f64; 3]| {
        let mut e = |id: usize, i: usize| {
            json!({"id": id, "type": "frame", "nodeI": i, "nodeJ": 3, "materialId": 1, "sectionId": 1,
                   "releaseMyEnd": true, "releaseMzEnd": true})
        };
        in3(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 4.0, "y": 0.0, "z": 0.0},
                      "3": {"id": 3, "x": 2.0, "y": 1.0, "z": 3.0}},
            "materials": mats(), "sections": sec3(),
            "elements": {"1": e(1, 1), "2": e(2, 2)},
            "supports": {"1": sup3(1, [true; 6]), "2": sup3(2, [true; 6])},
            "loads": [nodal3(3, [0.0, 0.0, -10.0, m[0], m[1], m[2]])]
        }))
    };
    solve_3d(&model([0.0; 3])).expect("no moment: solves");
    // in the plane of the two axes (2, 1, 3) and (−2, 1, 3): resisted by torsion
    solve_3d(&model([2.0, 1.0, 3.0])).expect("moment along a member axis solves");
    // the normal (0, 3, −1) of that plane: nothing resists it
    let e = solve_3d(&model([0.0, 3.0, -1.0])).unwrap_err();
    assert!(e.contains("Nodal moment at node 3"), "{e}");
}

/// A load on a node that nothing connects is refused (the node is otherwise
/// left alone: an unloaded stray node is the isolated-node gate's business).
#[test]
fn load_on_an_unconnected_node() {
    let model = |load: bool| {
        in2(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "z": 0.0},
                      "9": {"id": 9, "x": 9.0, "z": 9.0}},
            "materials": mats(), "sections": sec2(),
            "elements": {"1": frame2(1, 1, 2, false, false)},
            "supports": {"1": sup2(1, 1, "fixed"), "2": sup2(2, 9, "fixed")},
            "loads": [nodal2(if load { 9 } else { 2 }, 0.0, -10.0, 0.0)]
        }))
    };
    solve_2d(&model(false)).expect("unloaded stray node");
    let e = solve_2d(&model(true)).unwrap_err();
    assert!(e.contains("node 9") && e.contains("not connected"), "{e}");
}

/// A beam pinned at both ends in 3D spins about its own axis.
#[test]
fn beam_spinning_about_its_axis() {
    let model = |rrx: bool| {
        in3(json!({
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 6.0, "y": 0.0, "z": 0.0}},
            "materials": mats(), "sections": sec3(),
            "elements": {"1": {"id": 1, "type": "frame", "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1}},
            "supports": {"1": sup3(1, [true, true, true, rrx, false, false]), "2": sup3(2, [false, true, true, false, false, false])},
            "loads": [nodal3(2, [0.0, 0.0, -10.0, 0.0, 0.0, 0.0])]
        }))
    };
    let msg = refused_3d(&model(false));
    assert_mechanism(&msg, &[1, 2], "can rotate freely about an axis along (1, 0, 0) without any node translating");
    solve_3d(&model(true)).expect("torsion held");
}

// ---------------------------------------------------------------------------
// The kinematic conditions against the assembled stiffness
// ---------------------------------------------------------------------------

/// ‖K·v‖∞ / (max|K| · ‖v‖∞), v in the frame K is assembled in.
fn energy_residual(k: &[f64], n: usize, mut u: Vec<f64>) -> f64 {
    let _ = &mut u;
    let kmax = k.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let umax = u.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    assert!(umax > 0.0, "zero free motion");
    let mut r = 0.0f64;
    for i in 0..n {
        let s: f64 = (0..n).map(|j| k[i * n + j] * u[j]).sum();
        r = r.max(s.abs());
    }
    r / (kmax * umax)
}

fn residuals_2d(input: &SolverInput) -> Vec<f64> {
    let dn = DofNumbering::build_2d(input);
    let st = assemble_stiffness_2d(input, &dn);
    let n = dn.n_total;
    free_motion_vectors(Some(input), None, 3)
        .into_iter()
        .map(|v| {
            let mut u = vec![0.0; n];
            for (node, d, x) in v {
                if let Some(&i) = dn.map.get(&(node, d)) {
                    u[i] = x;
                }
            }
            for s in input.supports.values() {
                if let (true, Some(t)) = (s.support_type == "inclinedRoller", s.angle) {
                    let r = inclined_rotation_matrix_2d(t);
                    let (a, b) = (dn.map[&(s.node_id, 0)], dn.map[&(s.node_id, 1)]);
                    let (x, z) = (u[a], u[b]);
                    u[a] = r[0][0] * x + r[0][1] * z;
                    u[b] = r[1][0] * x + r[1][1] * z;
                }
            }
            energy_residual(&st.k, n, u)
        })
        .collect()
}

fn residuals_3d(input: &SolverInput3D) -> Vec<f64> {
    let dn = DofNumbering::build_3d(input);
    let st = assemble_stiffness_3d(input, &dn);
    let n = dn.n_total;
    free_motion_vectors(None, Some(input), 3)
        .into_iter()
        .map(|v| {
            let mut u = vec![0.0; n];
            for (node, d, x) in v {
                if let Some(&i) = dn.map.get(&(node, d)) {
                    u[i] = x;
                }
            }
            for s in input.supports.values() {
                if let (Some(true), Some(x), Some(y), Some(z)) = (s.is_inclined, s.normal_x, s.normal_y, s.normal_z) {
                    if (x * x + y * y + z * z).sqrt() <= 1e-12 {
                        continue;
                    }
                    let r = inclined_rotation_matrix(x, y, z);
                    let ix = [0, 1, 2].map(|k| dn.map[&(s.node_id, k)]);
                    let g = ix.map(|i| u[i]);
                    for a in 0..3 {
                        u[ix[a]] = (0..3).map(|b| r[a][b] * g[b]).sum();
                    }
                }
            }
            energy_residual(&st.k, n, u)
        })
        .collect()
}

/// Every free motion reported on a random model — trusses, hinges, releases
/// about any axis, springs (rotated too), connectors, inclined supports,
/// constraints, shells — has zero energy in the stiffness the solver
/// assembles (constraints are imposed on v, which K does not see; a zero-
/// energy v of the PSD K is a null vector of it). Sign conventions or axes
/// that disagreed with the elements would show here. Supports are dropped on
/// a third of the models so that every model has free motions to test.
#[test]
fn reported_free_motions_have_zero_energy() {
    let (mut tested, mut flagged) = (0, 0);
    for seed in 0..400u64 {
        let (mut m2, _) = fuzz_models::random_2d_model_rich(seed);
        if seed % 2 == 0 {
            fuzz_models::add_random_constraints_2d(&mut m2, seed);
        }
        if seed % 3 == 0 {
            m2.supports.clear();
        }
        if check_2d(&m2, &m2.constraints).is_mechanism() {
            flagged += 1;
            for r in residuals_2d(&m2) {
                assert!(r < 1e-9, "2d seed {seed}: free motion with energy (residual {r:e})");
                tested += 1;
            }
        }
        let (mut m3, _) = fuzz_models::random_3d_model_rich(seed);
        if seed % 2 == 0 {
            fuzz_models::add_random_constraints_3d(&mut m3, seed);
        }
        if seed % 3 == 0 {
            m3.supports.clear();
        }
        if !m3.curved_beams.is_empty() || DofNumbering::dofs_per_node_3d(&m3) == 7 {
            continue;
        }
        if check_3d(&m3, &m3.constraints).is_mechanism() {
            flagged += 1;
            for r in residuals_3d(&m3) {
                assert!(r < 1e-9, "3d seed {seed}: free motion with energy (residual {r:e})");
                tested += 1;
            }
        }
    }
    assert!(tested > 200 && flagged > 100, "only {tested} motions on {flagged} models");
}

// ---------------------------------------------------------------------------
// No false positives
// ---------------------------------------------------------------------------

/// No fixture (example models, regression inputs) is a mechanism.
#[test]
fn no_fixture_is_flagged() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let mut checked = 0;
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_str().unwrap().to_string();
        if !name.ends_with(".json") {
            continue;
        }
        let json = std::fs::read_to_string(&p).unwrap();
        let c = if name.contains("3d") {
            match serde_json::from_str::<SolverInput3D>(&json) {
                Ok(i) => check_3d(&i, &i.constraints),
                Err(_) => continue,
            }
        } else {
            match serde_json::from_str::<SolverInput>(&json) {
                Ok(i) => check_2d(&i, &i.constraints),
                Err(_) => continue,
            }
        };
        assert!(!c.is_mechanism(), "{name}: {}", c.message().unwrap());
        assert!(c.unchecked.is_none(), "{name}: unchecked ({:?})", c.unchecked);
        checked += 1;
    }
    assert!(checked > 100, "only {checked} fixtures");
}
