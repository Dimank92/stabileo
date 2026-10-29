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
    let mut a2: Vec<_> = ana2::rows()
        .into_iter()
        .filter(|a| only.map_or(true, |n| a.name == n))
        .collect();
    let mut a3: Vec<_> = ana3::rows()
        .into_iter()
        .filter(|a| only.map_or(true, |n| a.name == n))
        .collect();
    if only.is_none() {
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
    }
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
    for t in [&t2, &t3] {
        for ((a, inv), cell) in t.cells.lock().unwrap().iter() {
            assert_eq!(
                cell.refused + cell.unrep + cell.inconclusive,
                0,
                "coverage lost: {a} × {inv}: {cell:?}"
            );
            if ["units", "relabel", "reversal"].contains(&inv.as_str()) {
                assert_eq!(
                    cell.pass + cell.fails.len(),
                    count as usize,
                    "coverage lost: {a} × {inv}"
                );
            }
        }
    }
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
    sweep(4096, None);
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
        let mut expected = (a.run)(c)?;
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
        let got = (a.run)(&tc)?;
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
