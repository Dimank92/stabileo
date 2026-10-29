use crate::{
    ana2, ana3,
    case::{Case2, Case3},
    checks2, checks3,
    tally::Tally,
};
use dedaliano_engine::metamorphic::Rng;
use serde_json::{json, Value};
use std::cell::Cell;

thread_local! { static STRICT: Cell<bool> = const { Cell::new(false) }; }

pub fn strict() -> bool {
    STRICT.with(Cell::get)
}

pub(crate) fn model(seed: u64, spatial: bool) -> Value {
    let mut rng = Rng::new(seed);
    let l = rng.range(1.5, 3.0);
    let a = rng.range(0.015, 0.025);
    let iz = rng.range(1.5e-4, 3e-4);
    let mut v = json!({
        "nodes": {
            "1": {"id":1,"x":0.0,"z":0.0},
            "2": {"id":2,"x":l,"z":0.0},
            "3": {"id":3,"x":2.0*l,"z":0.0}
        },
        "elements": {
            "1": {"id":1,"type":"frame","nodeI":1,"nodeJ":2,"materialId":1,"sectionId":1},
            "2": {"id":2,"type":"frame","nodeI":2,"nodeJ":3,"materialId":1,"sectionId":1}
        },
        "materials": {"1":{"id":1,"e":rng.range(180000.0,220000.0),"nu":0.3}},
        "sections": {"1":{"id":1,"a":a,"iz":iz,"iy":iz*0.63,"j":iz*0.4}},
        "supports": {"1":{"id":1,"nodeId":1,"type":"fixed"}},
        "loads": [
            {"type":"nodal","data":{"nodeId":2,"fx":-0.1,"fz":-0.15,"my":0.03}},
            {"type":"nodal","data":{"nodeId":3,"fx":-0.2,"fz":-0.3,"my":0.02}}
        ]
    });
    if spatial {
        for n in v["nodes"].as_object_mut().unwrap().values_mut() {
            n["y"] = json!(0.0);
        }
        v["supports"]["1"] =
            json!({"nodeId":1,"rx":true,"ry":true,"rz":true,"rrx":true,"rry":true,"rrz":true});
        for load in v["loads"].as_array_mut().unwrap() {
            load["data"]["fy"] = json!(0.07);
            load["data"]["mx"] = json!(0.01);
            load["data"]["mz"] = json!(0.04);
        }
    }
    v
}

struct Strict(bool);

impl Drop for Strict {
    fn drop(&mut self) {
        STRICT.with(|s| s.set(self.0));
    }
}

fn sweep(count: u64, only: Option<&str>) {
    let _strict = Strict(STRICT.with(|s| s.replace(true)));
    let inv3: Vec<_> = crate::INV_3D
        .iter()
        .copied()
        .chain(["reflection"])
        .collect();
    let start = std::time::Instant::now();
    let t2 = Tally::default();
    let t3 = Tally::default();
    let mut a2 = ana2::rows();
    let mut a3 = ana3::rows();
    a2.push(ana2::Ana2 {
        name: "Winkler (active)",
        law: ana2::Law::Linear,
        axis_bound: false,
        iter_tol: 0.0,
        full_static: false,
        run: foundation_2d,
    });
    a3.push(ana3::Ana3 {
        name: "Winkler (active)",
        law: ana2::Law::Linear,
        motion: ana3::Motion::Any,
        iter_tol: 0.0,
        full_static: false,
        run: foundation_3d,
    });
    a2.push(ana2::Ana2 {
        name: "contact (active gap)",
        law: ana2::Law::Nonlinear,
        axis_bound: true,
        iter_tol: 1e-10,
        full_static: false,
        run: gap_2d,
    });
    a3.push(ana3::Ana3 {
        name: "contact (active gap)",
        law: ana2::Law::Nonlinear,
        motion: ana3::Motion::AxisPerm,
        iter_tol: 1e-10,
        full_static: false,
        run: gap_3d,
    });
    a2.retain(|a| only.map_or(true, |name| a.name == name));
    a3.retain(|a| only.map_or(true, |name| a.name == name));
    assert!(
        !a2.is_empty() || !a3.is_empty(),
        "unknown generated analysis: {only:?}"
    );
    for k in 0..count {
        let before = t2.failures().len() + t3.failures().len();
        let seed = crate::seed().wrapping_add(k);
        let c2 = Case2::new(
            format!("generated/{seed}/2d"),
            serde_json::from_value(model(seed, false)).unwrap(),
        );
        let c3 = Case3::new(
            format!("generated/{seed}/3d"),
            serde_json::from_value(model(seed, true)).unwrap(),
        );
        for a in &a2 {
            let invs: Vec<_> = crate::INV_2D
                .iter()
                .copied()
                .filter(|inv| {
                    if a.name == "contact (active gap)" {
                        return ["units", "relabel", "reversal"].contains(inv);
                    }
                    if only.is_none()
                        && a.name == "corotational"
                        && ["rotation", "mirror"].contains(inv)
                    {
                        t2.record(
                            a.name,
                            inv,
                            &c2.name,
                            crate::tally::Verdict::Ignored("ignored(#761)"),
                        );
                        false
                    } else {
                        true
                    }
                })
                .collect();
            checks2::check_case(
                &checks2::Ctx {
                    tally: &t2,
                    rng_seed: seed,
                },
                &c2,
                std::slice::from_ref(a),
                &invs,
            );
        }
        for a in &a3 {
            let invs: Vec<_> = crate::INV_3D
                .iter()
                .copied()
                .filter(|inv| {
                    if a.name == "contact (active gap)" {
                        return ["units", "relabel", "reversal"].contains(inv);
                    }
                    if only.is_none() && a.name == "corotational" && *inv == "rotation" {
                        t3.record(
                            a.name,
                            inv,
                            &c3.name,
                            crate::tally::Verdict::Ignored("ignored(#761)"),
                        );
                        false
                    } else {
                        true
                    }
                })
                .collect();
            checks3::check_case(
                &checks3::Ctx {
                    tally: &t3,
                    rng_seed: seed,
                },
                &c3,
                std::slice::from_ref(a),
                &invs,
            );
            t3.record(a.name, "reflection", &c3.name, reflection_3d(a, &c3));
        }
        if t2.failures().len() + t3.failures().len() > before {
            if let Ok(dir) = std::env::var("METAMORPHIC_REPORT_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                for (dim, spatial) in [("2d", false), ("3d", true)] {
                    std::fs::write(
                        format!("{dir}/seed-{seed}-{dim}.json"),
                        serde_json::to_string_pretty(&model(seed, spatial)).unwrap(),
                    )
                    .unwrap();
                }
            }
            eprintln!(
                "Reproducer seed={seed}\n2D={}\n3D={}",
                model(seed, false),
                model(seed, true)
            );
        }
    }
    println!(
        "generated 2D\n{}",
        crate::render(
            &t2,
            &a2.iter().map(|a| a.name).collect::<Vec<_>>(),
            crate::INV_2D
        )
    );
    println!(
        "generated 3D\n{}",
        crate::render(&t3, &a3.iter().map(|a| a.name).collect::<Vec<_>>(), &inv3)
    );
    println!("generated sweep: {count} seeds, {:?}", start.elapsed());
    println!("{}{}", t2.report(), t3.report());
    if let Ok(dir) = std::env::var("METAMORPHIC_REPORT_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        for (dim, tally, names, invs) in [
            (
                "2d",
                &t2,
                a2.iter().map(|a| a.name).collect::<Vec<_>>(),
                crate::INV_2D,
            ),
            (
                "3d",
                &t3,
                a3.iter().map(|a| a.name).collect::<Vec<_>>(),
                &inv3,
            ),
        ] {
            std::fs::write(
                format!("{dir}/generated_{dim}.md"),
                crate::render(tally, &names, invs) + &tally.report(),
            )
            .unwrap();
        }
    }
    assert_coverage(
        &t2,
        &a2.iter().map(|a| a.name).collect::<Vec<_>>(),
        false,
        count,
        only,
    );
    assert_coverage(
        &t3,
        &a3.iter().map(|a| a.name).collect::<Vec<_>>(),
        true,
        count,
        only,
    );
    let failures: Vec<_> = t2.failures().into_iter().chain(t3.failures()).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn generated_fast() {
    sweep(128, None);
}

#[test]
#[ignore = "large deterministic seed sweep"]
fn metamorphic_nightly_generated() {
    sweep(4096, crate::only().as_deref());
}

fn foundation_2d(c: &Case2) -> Result<crate::response::Resp, String> {
    use dedaliano_engine::solver::winkler::*;
    let springs =
        c.s.elements
            .values()
            .map(|e| {
                let sec = &c.s.sections[&e.section_id.to_string()];
                let mat = &c.s.materials[&e.material_id.to_string()];
                FoundationSpring {
                    element_id: e.id,
                    kf: mat.e * 1000.0 * sec.iz / sec.a.powi(2) * 1e-3,
                }
            })
            .collect();
    Ok(crate::response::static_2d(&solve_winkler_2d(
        &WinklerInput {
            solver: c.s.clone(),
            foundation_springs: springs,
        },
    )?))
}

fn foundation_3d(c: &Case3) -> Result<crate::response::Resp, String> {
    use dedaliano_engine::solver::winkler::*;
    let springs =
        c.s.elements
            .values()
            .map(|e| {
                let sec = &c.s.sections[&e.section_id.to_string()];
                let mat = &c.s.materials[&e.material_id.to_string()];
                let k = mat.e * 1000.0 * sec.iz / sec.a.powi(2) * 1e-3;
                FoundationSpring3D {
                    element_id: e.id,
                    ky: Some(k),
                    kz: Some(k * 0.7),
                }
            })
            .collect();
    Ok(crate::response::static_3d(&solve_winkler_3d(
        &WinklerInput3D {
            solver: c.s.clone(),
            foundation_springs: springs,
        },
    )?))
}

#[test]
#[ignore = "safety #761"]
fn corotational_rigid_motion_reproducer() {
    use dedaliano_engine::metamorphic::transform::{Iso2, Iso3};
    use dedaliano_engine::solver::corotational;
    let mut failures = Vec::new();
    for spatial in [false, true] {
        let mut v = model(20261020, spatial);
        v["nodes"].as_object_mut().unwrap().remove("3");
        v["elements"].as_object_mut().unwrap().remove("2");
        v["loads"].as_array_mut().unwrap().pop();
        if spatial {
            let c = Case3::new("minimal", serde_json::from_value(v).unwrap());
            let base = corotational::solve_corotational_3d(&c.s, 100, 1e-10, 4, false).unwrap();
            for x in [0.0] {
                let tc = c
                    .iso(&Iso3::from_euler(0.3, 0.7, 1.1).translated([x, x, x]))
                    .unwrap();
                let got = corotational::solve_corotational_3d(&tc.s, 100, 1e-10, 4, false).unwrap();
                assert!(base.converged);
                if !got.converged {
                    failures.push("3D rotation");
                }
            }
        } else {
            let c = Case2::new("minimal", serde_json::from_value(v).unwrap());
            let base = corotational::solve_corotational_2d(&c.s, 100, 1e-10, 4, false).unwrap();
            for x in [0.0] {
                let tc = c.iso(&Iso2::rotation(0.7).translated(x, x)).unwrap();
                let got = corotational::solve_corotational_2d(&tc.s, 100, 1e-10, 4, false).unwrap();
                assert!(base.converged);
                if !got.converged {
                    failures.push("2D rotation");
                }
            }
        }
    }
    assert!(failures.is_empty(), "safety #761: {failures:?}");
}

#[test]
#[ignore = "safety #761"]
fn generated_corotational_rigid_motions() {
    sweep(128, Some("corotational"));
}

fn gap_2d(c: &Case2) -> Result<crate::response::Resp, String> {
    use dedaliano_engine::solver::contact;
    let support = c.s.supports.values().next().unwrap().node_id;
    let sec = c.s.sections.values().next().unwrap();
    let e = c.s.materials.values().next().unwrap().e;
    let input: contact::ContactInput = serde_json::from_value(json!({
        "solver": c.s, "tolerance": 1e-10,
        "gapElements": [{"id":1,"nodeI":support,"nodeJ":c.probes.last().unwrap(),"direction":0,
            "initialGap":sec.a.sqrt()*1e-8,"stiffness":e*1000.0*sec.a.sqrt()}]
    }))
    .unwrap();
    let r = contact::solve_contact_2d(&input)?;
    if !r.converged || r.gap_status.len() != 1 || r.gap_status[0].status != "closed" {
        return Err(format!("active contact not exercised: {:?}", r.gap_status));
    }
    Ok(crate::response::static_2d(&r.results))
}

fn gap_3d(c: &Case3) -> Result<crate::response::Resp, String> {
    use dedaliano_engine::solver::contact;
    let support = c.s.supports.values().next().unwrap().node_id;
    let sec = c.s.sections.values().next().unwrap();
    let e = c.s.materials.values().next().unwrap().e;
    let input: contact::ContactInput3D = serde_json::from_value(json!({
        "solver": c.s, "tolerance": 1e-10,
        "gapElements": [{"id":1,"nodeI":support,"nodeJ":c.probes.last().unwrap(),"direction":0,
            "initialGap":sec.a.sqrt()*1e-8,"stiffness":e*1000.0*sec.a.sqrt()}]
    }))
    .unwrap();
    let r = contact::solve_contact_3d(&input)?;
    if !r.converged || r.gap_status.len() != 1 || r.gap_status[0].status != "closed" {
        return Err(format!("active contact not exercised: {:?}", r.gap_status));
    }
    Ok(crate::response::static_3d(&r.results))
}

#[test]
fn independent_mass_and_time_units() {
    use dedaliano_engine::metamorphic::{compare::compare_with_floor, units::Units};
    let u = Units {
        length: 0.2,
        force: 3.0,
        time: 7.0,
    };
    for seed in 0..8 {
        let c2 = Case2::new(
            "independent units",
            serde_json::from_value(model(seed, false)).unwrap(),
        );
        let c3 = Case3::new(
            "independent units",
            serde_json::from_value(model(seed, true)).unwrap(),
        );
        for a in ana2::rows()
            .into_iter()
            .filter(|a| ["modal", "time history", "harmonic"].contains(&a.name))
        {
            let expected = (a.run)(&c2).unwrap().units(&u);
            let tc = c2.units(&u);
            let got = (a.run)(&tc).unwrap();
            compare_with_floor(&got.flat(), &expected.flat(), 1.0, 1e-9, 0.0)
                .unwrap_or_else(|e| panic!("2d {} seed={seed}: {e}", a.name));
        }
        for a in ana3::rows()
            .into_iter()
            .filter(|a| ["modal", "time history", "harmonic"].contains(&a.name))
        {
            let expected = (a.run)(&c3).unwrap().units(&u);
            let tc = c3.units(&u);
            let got = (a.run)(&tc).unwrap();
            compare_with_floor(&got.flat(), &expected.flat(), 1.0, 1e-9, 0.0)
                .unwrap_or_else(|e| panic!("3d {} seed={seed}: {e}", a.name));
        }
    }
}

fn reflection_3d(a: &ana3::Ana3, c: &Case3) -> crate::tally::Verdict {
    use crate::{response::Kind, tally::Verdict};
    use dedaliano_engine::{
        element::compute_local_axes_3d, metamorphic::compare::compare_with_floor,
        types::SolverLoad3D,
    };
    if a.motion == ana3::Motion::AboutZ || a.name == "contact (active gap)" {
        return Verdict::NotApplicable;
    }
    let check = || -> Result<(), String> {
        let mut expected = checks3::run(a, c)?;
        let mut tc = c.clone();
        for n in tc.s.nodes.values_mut() {
            (n.x, n.y, n.z) = (-n.x, -n.y, -n.z);
        }
        for (key, e) in &c.s.elements {
            let i = &c.s.nodes[&e.node_i.to_string()];
            let j = &c.s.nodes[&e.node_j.to_string()];
            let (_, y, _) = compute_local_axes_3d(
                i.x,
                i.y,
                i.z,
                j.x,
                j.y,
                j.z,
                e.local_yx,
                e.local_yy,
                e.local_yz,
                e.roll_angle,
                false,
            );
            let te = tc.s.elements.get_mut(key).unwrap();
            (te.local_yx, te.local_yy, te.local_yz, te.roll_angle) =
                (Some(-y[0]), Some(-y[1]), Some(-y[2]), None);
        }
        for l in &mut tc.s.loads {
            if let SolverLoad3D::Nodal(n) = l {
                (n.fx, n.fy, n.fz) = (-n.fx, -n.fy, -n.fz);
            }
        }
        tc.p.ground = tc.p.ground.map(|x| -x);
        for it in &mut expected.0 {
            match (&it.kind, it.tag.as_str()) {
                (Kind::V3(d), _)
                    if *d == crate::response::Dim::NONE || *d == crate::response::Dim::MOMENT => {}
                (Kind::V3(_), _) => it.v.iter_mut().for_each(|x| *x = -*x),
                (Kind::Local(_, _), "V") if it.v.len() == 4 => {
                    it.v[2..].iter_mut().for_each(|x| *x = -*x)
                }
                (Kind::Local(_, _), "M") if it.v.len() == 6 => {
                    it.v[..4].iter_mut().for_each(|x| *x = -*x)
                }
                _ => {}
            }
        }
        let got = checks3::run(a, &tc)?;
        let cond = dedaliano_engine::metamorphic::conditioning::of_3d(&c.s)?;
        let rel = checks2::rel_for(a.name, a.iter_tol, &cond);
        compare_with_floor(
            &got.flat(),
            &expected.flat(),
            cond.lchar,
            rel,
            checks3::load_floor_3d(&c.s, cond.lchar),
        )?;
        Ok(())
    };
    match check() {
        Ok(()) => Verdict::Pass(0.0),
        Err(e) => Verdict::Fail(e),
    }
}

const MATRIX_2D: &[(&str, &str)] = &[
    ("linear", "PPPPPPPP---"),
    ("linear (dense reference)", "PPPPPPPP-P-"),
    ("linear (sparse path)", "PPPPPPPP-P-"),
    ("linear (PCG)", "PPPPPPPP-P-"),
    ("constrained", "PPPPPPPP-P-"),
    ("certified linear", "PPPPPPPP-P-"),
    ("load combinations", "PPPPPPPP-P-"),
    ("staged (one stage)", "PPPPPPPP-P-"),
    ("staged (loads in two stages)", "PPPPPPP--P-"),
    ("Winkler (no foundation)", "PPPPPPPP-P-"),
    ("contact (no gaps)", "PPPPPPP--P-"),
    ("SSI (no soil springs)", "PPPPPPP--P-"),
    ("creep & shrinkage (t = t0)", "PPPPPPP--P-"),
    ("P-Delta", "PPPPP------"),
    ("corotational", "IIPPP------"),
    ("material nonlinear", "PPPPP------"),
    ("fiber nonlinear", "PPPPP------"),
    ("cable solver", "-PPPP------"),
    ("modal", "PPPPP-----P"),
    ("certified modal", "PPPPP------"),
    ("buckling", "PPPPPP-----"),
    ("time history", "PPPPPPP----"),
    ("spectral", "-PPPP------"),
    ("harmonic", "PPPPPPP----"),
    ("plastic collapse", "PPPPPP-----"),
    ("kinematic check", "PPPPP------"),
    ("moving loads", "-PPPP------"),
    ("influence line", "-PPPP------"),
    ("Winkler (active)", "PPPPPPP----"),
    ("contact (active gap)", "--PPP------"),
];

const MATRIX_3D: &[(&str, &str)] = &[
    ("linear", "PPPPPPP---P"),
    ("linear (sparse path)", "PPPPPPP-P-P"),
    ("linear (PCG)", "PPPPPPP-P-P"),
    ("constrained", "PPPPPPP-P-P"),
    ("certified linear", "PPPPPPP-P-P"),
    ("load combinations", "PPPPPPP-P-P"),
    ("staged (one stage)", "PPPPPPP-P-P"),
    ("staged (loads in two stages)", "PPPPPP--P-P"),
    ("Winkler (no foundation)", "PPPPPPP-P-P"),
    ("contact (no gaps)", "PPPPPP--P-P"),
    ("SSI (no soil springs)", "PPPPPP--P-P"),
    ("creep & shrinkage (t = t0)", "PPPPPP--P-P"),
    ("P-Delta", "PPPP------P"),
    ("corotational", "IPPP------P"),
    ("material nonlinear", "PPPP------P"),
    ("fiber nonlinear", "PPPP------P"),
    ("cable solver", "PPPP-------"),
    ("modal", "PPPP-----PP"),
    ("certified modal", "PPPP------P"),
    ("buckling", "PPPPP-----P"),
    ("time history", "PPPPPP----P"),
    ("time history (ground motion)", "PPPP------P"),
    ("spectral", "PPPP------P"),
    ("harmonic", "PPPPPP----P"),
    ("plastic collapse", "PPPPP-----P"),
    ("kinematic check", "PPPP------P"),
    ("moving loads", "PPPP-------"),
    ("influence line", "PPPP-------"),
    ("Winkler (active)", "PPPPPP----P"),
    ("contact (active gap)", "-PPP-------"),
];

fn assert_coverage(t: &Tally, names: &[&str], spatial: bool, count: u64, only: Option<&str>) {
    let matrix = if spatial { MATRIX_3D } else { MATRIX_2D };
    let invs: Vec<_> = if spatial {
        crate::INV_3D
            .iter()
            .copied()
            .chain(["reflection"])
            .collect()
    } else {
        crate::INV_2D.to_vec()
    };
    let cells = t.cells.lock().unwrap();
    for name in names {
        let mask = matrix
            .iter()
            .find(|(n, _)| n == name)
            .expect("missing expected analysis")
            .1;
        assert_eq!(mask.len(), invs.len());
        for (inv, expected) in invs.iter().zip(mask.chars()) {
            let cell = cells
                .get(&(name.to_string(), inv.to_string()))
                .cloned()
                .unwrap_or_default();
            assert_eq!(
                cell.refused + cell.unrep + cell.inconclusive,
                0,
                "coverage lost: {name} × {inv}: {cell:?}"
            );
            if expected == 'I' && only.is_none() {
                assert_eq!(
                    cell.ignored.iter().copied().collect::<Vec<_>>(),
                    vec!["ignored(#761)"]
                );
                assert_eq!(cell.pass + cell.fails.len(), 0);
            } else {
                assert!(cell.ignored.is_empty(), "unexpected ignore: {name} × {inv}");
                assert_eq!(
                    cell.pass + cell.fails.len(),
                    if expected == '-' { 0 } else { count as usize },
                    "coverage lost: {name} × {inv}"
                );
            }
        }
    }
}

pub(crate) fn validate_response(
    name: &str,
    r: &crate::response::Resp,
    nodes: &[usize],
    spatial: bool,
) -> Result<(), String> {
    if r.0.is_empty()
        || r.0
            .iter()
            .any(|i| i.v.is_empty() || i.v.iter().any(|v| !v.is_finite()))
    {
        return Err(format!("{name}: empty or nonfinite response"));
    }
    let require = |node, tag: &str, width| -> Result<(), String> {
        let items: Vec<_> =
            r.0.iter()
                .filter(|i| i.node == node && i.tag == tag)
                .collect();
        if items.len() != 1 || items[0].v.len() != width {
            return Err(format!("{name}: invalid response shape for {node:?}/{tag}"));
        }
        Ok(())
    };
    let width = if spatial { 3 } else { 2 };
    if name.starts_with("time history") {
        let rotations = name == "time history";
        if r.0.len() != nodes.len() * 61 * if rotations { 2 } else { 1 } {
            return Err(format!("{name}: time-step count"));
        }
        for &node in nodes {
            for k in 0..=60 {
                require(Some(node), &format!("u_t{k:03}"), width)?;
                if rotations {
                    require(
                        Some(node),
                        &format!("{}_t{k:03}", if spatial { "r" } else { "ry" }),
                        if spatial { 3 } else { 1 },
                    )?;
                }
            }
        }
    } else if ["modal", "certified modal"].contains(&name) {
        require(None, "total_mass", 1)?;
        for k in 0..6 {
            require(None, &format!("f{k:02}"), 1)?;
        }
    } else if ![
        "buckling",
        "harmonic",
        "plastic collapse",
        "kinematic check",
        "moving loads",
        "influence line",
    ]
    .contains(&name)
    {
        for &node in nodes {
            require(Some(node), "u", width)?;
        }
        if r.0.iter().filter(|i| i.tag == "u").count() != nodes.len() {
            return Err(format!("{name}: node count"));
        }
    }
    Ok(())
}

fn foundation_changed(base: &crate::response::Resp, active: &crate::response::Resp) -> bool {
    let mut norm = 0.0_f64;
    let mut delta = 0.0_f64;
    for item in base.0.iter().filter(|i| i.tag == "u") {
        let Some(got) = active.node_vec(item.node.unwrap(), "u") else {
            return false;
        };
        if got.len() != item.v.len() {
            return false;
        }
        for (a, b) in item.v.iter().zip(got) {
            norm = norm.max(a.abs());
            delta = delta.max((a - b).abs());
        }
    }
    norm > 0.0 && delta > norm * 0.01
}

#[test]
fn active_foundation_changes_displacements() {
    for seed in 0..128 {
        let c2 = Case2::new(
            "foundation",
            serde_json::from_value(model(seed, false)).unwrap(),
        );
        let c3 = Case3::new(
            "foundation",
            serde_json::from_value(model(seed, true)).unwrap(),
        );
        let a2 = ana2::rows()
            .into_iter()
            .find(|a| a.name == "Winkler (no foundation)")
            .unwrap();
        let a3 = ana3::rows()
            .into_iter()
            .find(|a| a.name == "Winkler (no foundation)")
            .unwrap();
        for (base, active) in [
            ((a2.run)(&c2).unwrap(), foundation_2d(&c2).unwrap()),
            ((a3.run)(&c3).unwrap(), foundation_3d(&c3).unwrap()),
        ] {
            assert!(
                foundation_changed(&base, &active),
                "foundation ignored: seed={seed}"
            );
            assert!(
                !foundation_changed(&base, &base),
                "bypassed foundation must be rejected"
            );
        }
    }
}

#[test]
fn response_shape_rejects_missing_data() {
    let _strict = Strict(STRICT.with(|s| s.replace(true)));
    for spatial in [false, true] {
        let response = if spatial {
            let c = Case3::new("shape", serde_json::from_value(model(1, true)).unwrap());
            (ana3::rows()
                .into_iter()
                .find(|a| a.name == "time history")
                .unwrap()
                .run)(&c)
            .unwrap()
        } else {
            let c = Case2::new("shape", serde_json::from_value(model(1, false)).unwrap());
            (ana2::rows()
                .into_iter()
                .find(|a| a.name == "time history")
                .unwrap()
                .run)(&c)
            .unwrap()
        };
        validate_response("time history", &response, &[1, 2, 3], spatial).unwrap();
        for mutation in 0..3 {
            let mut bad = response.clone();
            match mutation {
                0 => bad.0.clear(),
                1 => bad.0.retain(|i| i.node != Some(3)),
                _ => bad.0.retain(|i| !i.tag.ends_with("060")),
            }
            assert!(validate_response("time history", &bad, &[1, 2, 3], spatial).is_err());
        }
    }
}

#[test]
fn coverage_rejects_vacuous_cells() {
    for spatial in [false, true] {
        let t = Tally::default();
        t.record(
            "linear",
            "rotation",
            "mutation",
            crate::tally::Verdict::NotApplicable,
        );
        assert!(
            std::panic::catch_unwind(|| assert_coverage(&t, &["linear"], spatial, 1, None))
                .is_err()
        );
    }
}
