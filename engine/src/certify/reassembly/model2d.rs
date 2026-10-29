//! Re-assembly of a 2D model (`SolverInput`) from its JSON fields.
//!
//! Conventions of the 2D model (the meaning of the input, mirrored from the
//! input documentation, not from the assembly code): nodes in the x–z plane;
//! DOFs (ux, uz, θ) with θ counter-clockwise (θ = dv/dx in member axes), only
//! (ux, uz) when the model has no frame member; member local x from i to j,
//! local y = (−sin, cos); E in MPa (×1000 → kN/m²), G = E/(2(1+ν)); shear area
//! `asY` pairs with Iz; thermal α = the load's `alpha`, else 12e-6 /°C; section
//! depth h = the load's `depth`, else √(12·Iz/A),
//! `dtGradient` = T(−y face) − T(+y face) (curvature +α·Δt/h); distributed
//! loads act along local y between `a` (default 0) and `b` (default L), axial
//! distributed loads along local x between `a` and `b` (`qI` at `a`, `qJ` at
//! `b`, linear between), point loads `p` along local y, `px` along local x,
//! `my` a counter-clockwise couple. Trusses and cables carry axial force only; their member loads reach
//! the nodes as simply supported reactions.

use std::collections::HashMap;

use super::element::*;
use super::ia::*;
use super::system::*;
use crate::types::*;

const ALPHA: f64 = 12e-6;

pub(super) fn build(input: &SolverInput, loads: &[SolverLoad]) -> Result<Model, String> {
    if !input.connectors.is_empty() {
        return Err("out of scope: connector elements".into());
    }
    let has_frame = input.elements.values().any(|e| e.elem_type == "frame");
    let dpn = if has_frame { 3 } else { 2 };
    let node_ids: Vec<usize> = input.nodes.values().map(|n| n.id).collect();
    let nodes: HashMap<usize, &SolverNode> = input.nodes.values().map(|n| (n.id, n)).collect();
    if nodes.len() != input.nodes.len() {
        return Err("duplicate node ids".into());
    }
    let mats: HashMap<usize, &SolverMaterial> = input.materials.values().map(|m| (m.id, m)).collect();
    let secs: HashMap<usize, &SolverSection> = input.sections.values().map(|s| (s.id, s)).collect();
    let mut model = Model::new(&node_ids, dpn);

    let mut by_elem: HashMap<usize, Vec<&SolverLoad>> = HashMap::new();
    for ld in loads {
        let id = match ld {
            SolverLoad::Distributed(d) => d.element_id,
            SolverLoad::PointOnElement(p) => p.element_id,
            SolverLoad::Thermal(t) => t.element_id,
            SolverLoad::AxialDistributed(d) => d.element_id,
            SolverLoad::Nodal(_) => continue,
        };
        by_elem.entry(id).or_default().push(ld);
    }
    let members: std::collections::HashSet<usize> = input.elements.values().map(|e| e.id).collect();
    check_member_load_ids(by_elem.keys().copied(), &members)?;
    let mut elems: Vec<&SolverElement> = input.elements.values().collect();
    elems.sort_by_key(|e| e.id);
    for el in elems {
        let ni = nodes.get(&el.node_i).ok_or("element references a missing node")?;
        let nj = nodes.get(&el.node_j).ok_or("element references a missing node")?;
        let mat = mats.get(&el.material_id).ok_or("missing material")?;
        let sec = secs.get(&el.section_id).ok_or("missing section")?;
        let dx = inp(nj.x) - inp(ni.x);
        let dz = inp(nj.z) - inp(ni.z);
        let l = (dx.square() + dz.square()).sqrt();
        if !certainly_gt(l, 0.0) {
            return Err(format!("element {}: length not certainly positive", el.id));
        }
        let (co, si) = (dx.div(l), dz.div(l));
        let e = c(1000.0) * inp(mat.e);
        let nu = inp(mat.nu);
        let g = e.div(c(2.0) * (c(1.0) + nu));
        let area = inp(sec.a);
        let iz = inp(sec.iz);
        let el_loads = by_elem.get(&el.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let is_frame = el.elem_type == "frame";
        if !is_frame && el.elem_type != "truss" && el.elem_type != "cable" {
            return Err(format!("out of scope: element type '{}'", el.elem_type));
        }
        let mut k = LMat::zeros(6);
        let mut f: LVec = vec![None; 6];
        place_axial(&mut k, 0, 3, (e * area).div(l));
        let both = el.hinge_start && el.hinge_end;
        let bending = is_frame && !both;
        let plane = if is_frame {
            let shear_flex = match sec.as_y {
                Some(asy) => {
                    let asy = inp(asy);
                    if !certainly_gt(asy, 0.0) {
                        return Err(format!("element {}: shear area not certainly positive", el.id));
                    }
                    l.div(g * asy)
                }
                None => I::ZERO,
            };
            let ei = e * iz;
            let phi = if shear_flex == I::ZERO { I::ZERO } else { (c(12.0) * ei * shear_flex).div(l * l.square()) };
            Some(Plane { ei, shear_flex, phi })
        } else {
            None
        };
        if bending {
            let p = plane.unwrap();
            let b = bending4(&p, l).ok_or_else(|| format!("element {}: bending stiffness not finite", el.id))?;
            place_bending(&mut k, [1, 2, 4, 5], &b, 1.0);
        }
        let (nsh, psh) = if bending { iie_shapes(plane.unwrap().phi, l) } else { lever_shapes(l) };
        for ld in el_loads {
            match ld {
                SolverLoad::Distributed(dl) => {
                    let a = dl.a.map(inp).unwrap_or(I::ZERO);
                    let b = dl.b.map(inp).unwrap_or(l);
                    let v = dist_enl(&nsh, l, a, b, inp(dl.q_i), inp(dl.q_j));
                    place_enl(&mut f, [1, 2, 4, 5], &v, 1.0);
                }
                SolverLoad::PointOnElement(pl) => {
                    let a = inp(pl.a);
                    let m = pl.my.map(inp).unwrap_or(I::ZERO);
                    let v = point_enl(&nsh, &psh, l, a, inp(pl.p), m);
                    place_enl(&mut f, [1, 2, 4, 5], &v, 1.0);
                    if let Some(px) = pl.px {
                        let px = inp(px);
                        let xi = a.div(l);
                        lvec_add(&mut f, 0, px * (c(1.0) - xi));
                        lvec_add(&mut f, 3, px * xi);
                    }
                }
                SolverLoad::Thermal(tl) => {
                    let alpha = tl.alpha.map(inp).unwrap_or_else(|| dec(ALPHA));
                    let n = e * area * alpha * inp(tl.dt_uniform);
                    lvec_add(&mut f, 0, -n);
                    lvec_add(&mut f, 3, n);
                    if is_frame && tl.dt_gradient != 0.0 && sec.iz != 0.0 {
                        let m = thermal_moment(e, iz, area, alpha, inp(tl.dt_gradient), tl.depth.map(inp), el.id)?;
                        lvec_add(&mut f, 2, -m);
                        lvec_add(&mut f, 5, m);
                    }
                }
                SolverLoad::AxialDistributed(al) => {
                    // Work of q(x), linear from qI at a to qJ at b, on the
                    // axial shapes 1 − x/L and x/L: Fj = ∫ q x dx / L,
                    // Fi = ∫ q dx − Fj. Nothing transverse, so releases leave it.
                    let a = al.a.map(inp).unwrap_or(I::ZERO);
                    let b = al.b.map(inp).unwrap_or(l);
                    let s = b - a;
                    if s.hi <= 0.0 {
                        continue;
                    }
                    if s.lo <= 0.0 {
                        return Err(format!("unsupported: element {}: axial load span of uncertain sign", el.id));
                    }
                    let (qi, qj) = (inp(al.q_i), inp(al.q_j));
                    let half = s * c(0.5);
                    let total = half * (qi + qj);
                    let moment = half * (qi * (a + s.div(c(3.0))) + qj * (a + (s * c(2.0)).div(c(3.0))));
                    let fj = moment.div(l);
                    lvec_add(&mut f, 0, total - fj);
                    lvec_add(&mut f, 3, fj);
                }
                SolverLoad::Nodal(_) => {}
            }
        }
        if bending {
            if el.hinge_start {
                condense(&mut k, Some(&mut f), 2).map_err(|e| format!("element {}: {e}", el.id))?;
            }
            if el.hinge_end {
                condense(&mut k, Some(&mut f), 5).map_err(|e| format!("element {}: {e}", el.id))?;
            }
        } else {
            // Moments on a member that carries none: thermal curvature of a
            // simply supported member is free (no force).
            f[2] = None;
            f[5] = None;
        }
        let r = vec![vec![co, si], vec![-si, co]];
        let gd = |n: usize, d: usize| model.dof(n, d).expect("node dof");
        let mut blocks = vec![Block { local: vec![0, 1], global: vec![gd(el.node_i, 0), gd(el.node_i, 1)], r: Some(r.clone()) }];
        if dpn == 3 {
            blocks.push(Block { local: vec![2], global: vec![gd(el.node_i, 2)], r: None });
        }
        blocks.push(Block { local: vec![3, 4], global: vec![gd(el.node_j, 0), gd(el.node_j, 1)], r: Some(r) });
        if dpn == 3 {
            blocks.push(Block { local: vec![5], global: vec![gd(el.node_j, 2)], r: None });
        }
        let (dofs, kg, fg) = to_global(&k, &f, &blocks);
        let rigid = Rigid { ntrans: 2, nrot: if dpn == 3 { 1 } else { 0 }, delta: vec![dx, dz], handed: 1.0 };
        model.scatter(&dofs, &kg, &fg, Some(rigid));
    }

    // Nodal loads.
    for ld in loads {
        if let SolverLoad::Nodal(nl) = ld {
            for (d, v) in [(0, nl.fx), (1, nl.fz), (2, nl.my)] {
                if v != 0.0 {
                    let i = model.dof(nl.node_id, d).ok_or_else(|| missing_dof_load(&model, nl.node_id, d))?;
                    model.add_f(i, inp(v));
                }
            }
        }
    }

    let mut kin = Kinematics::new(model.n());
    supports(input, &mut model, &mut kin, dpn)?;
    constraints(input, &model, &mut kin, dpn, has_frame)?;
    kin.build(&mut model)?;
    Ok(model)
}

/// A nodal load component on a DOF the model does not have: the node is
/// missing, or the model has no rotations (only trusses and cables) and a
/// moment acts. Nothing resists it: the model has no solution.
pub(super) fn missing_dof_load(model: &Model, node: usize, d: usize) -> String {
    if model.dof(node, 0).is_none() {
        format!("a load acts on node {node}, which is not in the model: nothing resists it")
    } else {
        format!("a load acts on DOF {d} of node {node}, which the model does not have (no member carries moments): nothing resists it")
    }
}

/// Member loads must name a member of the model.
pub(super) fn check_member_load_ids(ids: impl Iterator<Item = usize>, members: &std::collections::HashSet<usize>) -> Result<(), String> {
    for id in ids {
        if !members.contains(&id) {
            return Err(format!("a load acts on element {id}, which is not in the model: nothing resists it"));
        }
    }
    Ok(())
}

/// EI·α·Δt/h with h the depth the load states, else √(12·I/A) (the depth of a
/// rectangle with this A and I), or 0 when h ≤ 1e-12, as the model defines it.
pub(super) fn thermal_moment(e: I, i: I, a: I, alpha: I, dt: I, depth: Option<I>, id: usize) -> Result<I, String> {
    let h = match (depth, decide_gt(a, 1e-15)) {
        (Some(h), _) => h,
        (None, Some(true)) => (c(12.0) * i).div(a).sqrt(),
        (None, Some(false)) => c(0.1),
        (None, None) => return Err(format!("element {id}: thermal section depth ambiguous")),
    };
    match decide_gt(h, 1e-12) {
        Some(true) => Ok((e * i * alpha * dt).div(h)),
        Some(false) => Ok(I::ZERO),
        None => Err(format!("element {id}: thermal section depth ambiguous")),
    }
}

const RIGID_2D: [&str; 8] = ["fixed", "pinned", "rollerX", "rollerY", "rollerZ", "guidedX", "guidedY", "guidedZ"];

fn restrained_2d(t: &str, d: usize) -> bool {
    match t {
        "fixed" => true,
        "pinned" => d < 2,
        "rollerX" => d == 1,
        "rollerY" | "rollerZ" => d == 0,
        "guidedX" => d == 1 || d == 2,
        "guidedY" | "guidedZ" => d == 0 || d == 2,
        "inclinedRoller" => d == 1,
        _ => false,
    }
}

fn plain_2d(s: &SolverSupport) -> bool {
    RIGID_2D.contains(&s.support_type.as_str())
        && [s.kx, s.ky, s.kz, s.dx, s.dz, s.dry, s.angle].into_iter().all(|v| v.unwrap_or(0.0) == 0.0)
}

/// Support fields the model would not use: refused, never dropped.
fn ignored_support_fields_2d(s: &SolverSupport, dpn: usize) -> Result<(), String> {
    let nz = |v: Option<f64>| v.unwrap_or(0.0) != 0.0;
    let t = s.support_type.as_str();
    let node = s.node_id;
    if dpn < 3 && (s.kz.unwrap_or(0.0) > 0.0 || nz(s.dry)) {
        return Err(format!("unsupported: a rotational spring or prescribed rotation at node {node} of a model without rotations"));
    }
    if nz(s.angle) && t != "spring" && t != "inclinedRoller" {
        return Err(format!("unsupported: an angle on the {t} support at node {node}"));
    }
    // Which prescribed values the support uses.
    let used = |d: usize| match t {
        "spring" | "free" => false,
        "inclinedRoller" if s.angle.is_some() => d < 2,
        _ => restrained_2d(t, d) && d < dpn,
    };
    for (d, v) in [s.dx, s.dz, s.dry].into_iter().enumerate() {
        if nz(v) && !used(d) {
            return Err(format!("unsupported: a prescribed displacement on DOF {d} at node {node}, which its {t} support leaves free"));
        }
    }
    Ok(())
}

fn supports(input: &SolverInput, model: &mut Model, kin: &mut Kinematics, dpn: usize) -> Result<(), String> {
    let mut at: HashMap<usize, Vec<&SolverSupport>> = HashMap::new();
    for s in input.supports.values() {
        at.entry(s.node_id).or_default().push(s);
    }
    let mut nodes: Vec<usize> = at.keys().copied().collect();
    nodes.sort_unstable();
    for node in nodes {
        let sups = &at[&node];
        if model.dof(node, 0).is_none() {
            return Err(format!("support on missing node {node}"));
        }
        let known = |t: &str| RIGID_2D.contains(&t) || t == "inclinedRoller" || t == "spring" || t == "free";
        for s in sups {
            if !known(&s.support_type) {
                return Err(format!("unknown support type '{}'", s.support_type));
            }
            for v in [s.kx, s.ky, s.kz].into_iter().flatten() {
                if v < 0.0 {
                    return Err(format!("unsupported: negative spring at node {node}"));
                }
            }
        }
        if sups.len() > 1 {
            if !sups.iter().all(|s| plain_2d(s)) {
                return Err(format!("unsupported: several supports on node {node}"));
            }
            for d in 0..dpn {
                if sups.iter().any(|s| restrained_2d(&s.support_type, d)) {
                    kin.fix(model.dof(node, d).unwrap(), I::ZERO);
                }
            }
            continue;
        }
        let s = sups[0];
        let t = s.support_type.as_str();
        ignored_support_fields_2d(s, dpn)?;
        let node_dofs: Vec<Option<usize>> = (0..3).map(|d| model.dof(node, d)).collect();
        let dof = |d: usize| node_dofs[d];
        // Springs.
        let kx = s.kx.unwrap_or(0.0);
        let ky = s.ky.unwrap_or(0.0);
        let kz = s.kz.unwrap_or(0.0);
        let rotated = t == "spring" && s.angle.is_some_and(|a| a != 0.0) && (kx > 0.0 || ky > 0.0);
        if rotated {
            let (sn, cs) = sin_cos(inp(s.angle.unwrap())).ok_or("spring angle out of range")?;
            let (kxi, kyi) = (inp(kx), inp(ky));
            let kxx = kxi * cs.square() + kyi * sn.square();
            let kzz = kxi * sn.square() + kyi * cs.square();
            let kxz = (kxi - kyi) * sn * cs;
            let (d0, d1) = (dof(0).unwrap(), dof(1).unwrap());
            model.spring(&[d0, d1], vec![kxx, kxz, kxz, kzz]);
        } else {
            if kx > 0.0 {
                let d = dof(0).unwrap();
                model.spring(&[d], vec![inp(kx)]);
            }
            if ky > 0.0 {
                let d = dof(1).unwrap();
                model.spring(&[d], vec![inp(ky)]);
            }
        }
        if kz > 0.0 && dpn >= 3 {
            let d = dof(2).unwrap();
            model.spring(&[d], vec![inp(kz)]);
        }
        if t == "spring" || t == "free" {
            continue;
        }
        if t == "inclinedRoller" && s.angle.is_some() {
            // Normal n = (sin θ, cos θ) restrained, tangent t = (−cos θ, sin θ)
            // free; prescribed normal displacement n·(dx, dz).
            let (sn, cs) = sin_cos(inp(s.angle.unwrap())).ok_or("support angle out of range")?;
            let un = inp(s.dx.unwrap_or(0.0)) * sn + inp(s.dz.unwrap_or(0.0)) * cs;
            let p = vec![sn * un, cs * un];
            kin.incline(vec![dof(0).unwrap(), dof(1).unwrap()], vec![vec![-cs, sn]], p);
            continue;
        }
        let pres = [s.dx, s.dz, s.dry];
        for d in 0..dpn {
            if restrained_2d(t, d) {
                kin.fix(dof(d).unwrap(), pres[d].map(inp).unwrap_or(I::ZERO));
            }
        }
    }
    Ok(())
}

/// Multi-point constraints as equations `u_s = Σ c·u_m` (rigid-body
/// kinematics in the x–z plane: u_s = u_m − Δz·θ_m, w_s = w_m + Δx·θ_m).
pub(super) fn constraints(input: &SolverInput, model: &Model, kin: &mut Kinematics, dpn: usize, _has_frame: bool) -> Result<(), String> {
    let nodes: HashMap<usize, &SolverNode> = input.nodes.values().map(|n| (n.id, n)).collect();
    let off = |m: usize, s: usize| -> Result<(I, I), String> {
        let (a, b) = (nodes.get(&m).ok_or("constraint node missing")?, nodes.get(&s).ok_or("constraint node missing")?);
        // equal coordinates: an exact 0, not a rounding-wide interval
        let dd = |p: f64, q: f64| if p == q { I::ZERO } else { inp(q) - inp(p) };
        Ok((dd(a.x, b.x), dd(a.z, b.z)))
    };
    let dof = |n: usize, d: usize| model.dof(n, d);
    // A DOF a constraint names must exist: never drop part of a constraint.
    let need = |n: usize, d: usize| dof(n, d).ok_or_else(|| format!("unsupported: a constraint names DOF {d} of node {n}, which the model does not have"));
    // Rigid kinematics of slave DOF d from master `m` with offset (ox, oz).
    let rigid = |m: usize, d: usize, ox: I, oz: I| -> Result<Vec<(usize, I)>, String> {
        let mut t = vec![(need(m, d)?, c(1.0))];
        let coef = match d {
            0 => -oz,
            1 => ox,
            _ => I::ZERO,
        };
        if coef != I::ZERO {
            let r = dof(m, 2).ok_or_else(|| format!("unsupported: a rigid offset from node {m}, which has no rotation in this model"))?;
            t.push((r, coef));
        }
        Ok(t)
    };
    for con in &input.constraints {
        match con {
            Constraint::RigidLink(rl) => {
                let dofs: Vec<usize> = if rl.dofs.is_empty() { (0..dpn.min(3)).collect() } else { rl.dofs.clone() };
                let (ox, oz) = off(rl.master_node, rl.slave_node)?;
                for d in dofs {
                    kin.equation(need(rl.slave_node, d)?, rigid(rl.master_node, d, ox, oz)?);
                }
            }
            Constraint::Diaphragm(dia) => {
                // "XY" and "XZ" both name the model plane. Every slave moves
                // with the master as one rigid body, its rotation included.
                if dia.plane != "XY" && dia.plane != "XZ" {
                    return Err(format!("out of scope: 2D diaphragm in plane {}", dia.plane));
                }
                if dpn != 3 {
                    return Err("unsupported: diaphragm in a model without rotations".into());
                }
                for &sn in &dia.slave_nodes {
                    let (ox, oz) = off(dia.master_node, sn)?;
                    for d in [0, 1, 2] {
                        kin.equation(need(sn, d)?, rigid(dia.master_node, d, ox, oz)?);
                    }
                }
            }
            Constraint::EqualDOF(eq) => {
                for &d in &eq.dofs {
                    kin.equation(need(eq.slave_node, d)?, vec![(need(eq.master_node, d)?, c(1.0))]);
                }
            }
            Constraint::EccentricConnection(ec) => {
                let (ox, oz) = (inp(ec.offset_x), inp(ec.offset_y));
                for d in 0..dpn.min(3) {
                    if ec.releases.get(d).copied().unwrap_or(false) {
                        continue;
                    }
                    kin.equation(need(ec.slave_node, d)?, rigid(ec.master_node, d, ox, oz)?);
                }
            }
            Constraint::LinearMPC(mpc) => linear_mpc(mpc, model, kin)?,
        }
    }
    Ok(())
}

/// Σ cᵢ·uᵢ = 0: eliminate the term with the largest |c| among DOFs that are
/// neither restrained, nor on an inclined support, nor already a slave.
pub(super) fn linear_mpc(mpc: &LinearMPCConstraint, model: &Model, kin: &mut Kinematics) -> Result<(), String> {
    let mut terms: Vec<(usize, f64)> = Vec::with_capacity(mpc.terms.len());
    for t in &mpc.terms {
        let d = model.dof(t.node_id, t.dof).ok_or_else(|| {
            format!("unsupported: a linear MPC names DOF {} of node {}, which the model does not have", t.dof, t.node_id)
        })?;
        terms.push((d, t.coefficient));
    }
    let dep = terms
        .iter()
        .enumerate()
        .filter(|(_, (d, cf))| *cf != 0.0 && !kin.is_fixed(*d) && !kin.is_inclined(*d) && !kin.has_slave(*d))
        .max_by(|a, b| a.1 .1.abs().total_cmp(&b.1 .1.abs()))
        .map(|(i, _)| i);
    let Some(k) = dep else {
        return Err("unsupported: linear MPC without a free term to eliminate".into());
    };
    let (s, cs) = terms[k];
    if terms.iter().enumerate().any(|(i, (d, _))| i != k && *d == s) {
        return Err("unsupported: linear MPC with a repeated DOF".into());
    }
    let cs = inp(cs);
    let masters = terms.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, &(d, cf))| (d, -(inp(cf).div(cs)))).collect();
    kin.equation(s, masters);
    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restrained_2d_variants() {
        assert!(restrained_2d("rollerY", 0));
        assert!(!restrained_2d("rollerY", 1));
        assert!(restrained_2d("rollerZ", 0));
        assert!(!restrained_2d("rollerZ", 1));

        assert!(restrained_2d("guidedY", 0));
        assert!(!restrained_2d("guidedY", 1));
        assert!(restrained_2d("guidedY", 2));

        assert!(restrained_2d("guidedZ", 0));
        assert!(!restrained_2d("guidedZ", 1));
        assert!(restrained_2d("guidedZ", 2));

        assert!(!restrained_2d("inclinedRoller", 0));
        assert!(restrained_2d("inclinedRoller", 1));
        assert!(!restrained_2d("inclinedRoller", 2));
    }

    #[test]
    fn test_plain_2d_checks() {
        let sup_rigid = SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        };
        assert!(plain_2d(&sup_rigid));

        let mut sup_spring = sup_rigid.clone();
        sup_spring.support_type = "spring".into();
        assert!(!plain_2d(&sup_spring));

        let mut sup_with_dx = sup_rigid.clone();
        sup_with_dx.dx = Some(1e-3);
        assert!(!plain_2d(&sup_with_dx));

        let mut sup_with_angle = sup_rigid.clone();
        sup_with_angle.angle = Some(0.5);
        assert!(!plain_2d(&sup_with_angle));
    }

    #[test]
    fn test_ignored_support_fields() {
        let sup_free = SolverSupport {
            id: 1, node_id: 1, support_type: "free".into(),
            kx: None, ky: None, kz: None, dx: Some(1.0), dz: None, dry: None, angle: None,
        };
        assert!(ignored_support_fields_2d(&sup_free, 3).is_err());

        let sup_spring = SolverSupport {
            id: 1, node_id: 1, support_type: "spring".into(),
            kx: Some(100.0), ky: None, kz: None, dx: Some(1.0), dz: None, dry: None, angle: None,
        };
        assert!(ignored_support_fields_2d(&sup_spring, 3).is_err());

        let sup_dry = SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: Some(0.1), angle: None,
        };
        assert!(ignored_support_fields_2d(&sup_dry, 2).is_err());
    }

    fn make_test_input() -> SolverInput {
        SolverInput {
            nodes: HashMap::from([
                ("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 }),
                ("2".into(), SolverNode { id: 2, x: 1.0, z: 0.0 }),
            ]),
            elements: HashMap::new(),
            sections: HashMap::new(),
            materials: HashMap::new(),
            masses: Default::default(),
            loads: vec![],
            constraints: vec![],
            connectors: HashMap::new(),
            solver_options: None,
            supports: HashMap::new(),
        }
    }

    #[test]
    fn test_supports_and_guards() {
        let mk_mk = || (Model::new(&[1, 2], 3), Kinematics::new(6));

        // Negative spring
        let mut neg_sup = make_test_input();
        neg_sup.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "spring".into(),
            kx: Some(-10.0), ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        });
        let (mut m, mut kin) = mk_mk();
        assert!(supports(&neg_sup, &mut m, &mut kin, 3).is_err());

        // Free support should be accepted
        let mut free_sup = make_test_input();
        free_sup.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "free".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        });
        let (mut m, mut kin) = mk_mk();
        assert!(supports(&free_sup, &mut m, &mut kin, 3).is_ok());

        // Multiple supports on a single node: if one is non-plain, return Err
        let mut multi_sup = make_test_input();
        multi_sup.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        });
        multi_sup.supports.insert("2".into(), SolverSupport {
            id: 2, node_id: 1, support_type: "spring".into(),
            kx: Some(10.0), ky: None, kz: None, dx: None, dz: None, dry: None, angle: None,
        });
        let (mut m, mut kin) = mk_mk();
        assert!(supports(&multi_sup, &mut m, &mut kin, 3).is_err());

        // Unrotated spring with kx == 0.0 vs kx > 0.0
        let mut zero_spring = make_test_input();
        zero_spring.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "spring".into(),
            kx: Some(0.0), ky: Some(0.0), kz: None, dx: None, dz: None, dry: None, angle: None,
        });
        let (mut m_zero, mut kin) = mk_mk();
        supports(&zero_spring, &mut m_zero, &mut kin, 3).unwrap();
        assert_eq!(m_zero.elems.len(), 0);

        // Rotated spring with angle and kx > 0
        let mut rot_spring = make_test_input();
        rot_spring.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "spring".into(),
            kx: Some(100.0), ky: Some(50.0), kz: None, dx: None, dz: None, dry: None, angle: Some(30.0),
        });
        let (mut m_rot, mut kin) = mk_mk();
        supports(&rot_spring, &mut m_rot, &mut kin, 3).unwrap();
        assert_eq!(m_rot.elems.len(), 1);

        // Rotated spring with angle != 0 but kx == 0.0 and ky == 0.0: rotated must be false
        let mut rot_zero = make_test_input();
        rot_zero.supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "spring".into(),
            kx: Some(0.0), ky: Some(0.0), kz: None, dx: None, dz: None, dry: None, angle: Some(30.0),
        });
        let (mut m_rot_zero, mut kin) = mk_mk();
        supports(&rot_zero, &mut m_rot_zero, &mut kin, 3).unwrap();
        assert_eq!(m_rot_zero.elems.len(), 0);
    }

    #[test]
    fn test_constraints_diaphragm_planes() {
        let input = make_test_input();

        let mut inp_xz = input.clone();
        inp_xz.constraints.push(Constraint::Diaphragm(DiaphragmConstraint {
            master_node: 1, slave_nodes: vec![2], plane: "XZ".into(),
        }));
        let (m, mut kin) = (Model::new(&[1, 2], 3), Kinematics::new(6));
        assert!(constraints(&inp_xz, &m, &mut kin, 3, true).is_ok());

        let mut inp_xy = input.clone();
        inp_xy.constraints.push(Constraint::Diaphragm(DiaphragmConstraint {
            master_node: 1, slave_nodes: vec![2], plane: "XY".into(),
        }));
        let (m, mut kin) = (Model::new(&[1, 2], 3), Kinematics::new(6));
        assert!(constraints(&inp_xy, &m, &mut kin, 3, true).is_ok());

        let mut inp_yz = input.clone();
        inp_yz.constraints.push(Constraint::Diaphragm(DiaphragmConstraint {
            master_node: 1, slave_nodes: vec![2], plane: "YZ".into(),
        }));
        let (m, mut kin) = (Model::new(&[1, 2], 3), Kinematics::new(6));
        assert!(constraints(&inp_yz, &m, &mut kin, 3, true).is_err());

        let (m_truss, mut kin_truss) = (Model::new(&[1, 2], 2), Kinematics::new(4));
        assert!(constraints(&inp_xz, &m_truss, &mut kin_truss, 2, false).is_err());
    }

    #[test]
    fn test_linear_mpc_guards() {
        let model = Model::new(&[1, 2], 3);
        let mut kin = Kinematics::new(model.n());
        kin.fix(0, crate::certify::reassembly::ia::I::ZERO);

        let mpc_fixed = LinearMPCConstraint {
            terms: vec![MPCTerm { node_id: 1, dof: 0, coefficient: 1.0 }],
        };
        assert!(linear_mpc(&mpc_fixed, &model, &mut kin).is_err());

        let mpc_zero_cf = LinearMPCConstraint {
            terms: vec![MPCTerm { node_id: 1, dof: 1, coefficient: 0.0 }],
        };
        assert!(linear_mpc(&mpc_zero_cf, &model, &mut kin).is_err());
    }

    #[test]
    fn test_build_thermal_no_gradient() {
        let input = SolverInput {
            nodes: HashMap::from([
                ("1".into(), SolverNode { id: 1, x: 0.0, z: 0.0 }),
                ("2".into(), SolverNode { id: 2, x: 2.0, z: 0.0 }),
            ]),
            elements: HashMap::from([
                ("1".into(), SolverElement {
                    id: 1, elem_type: "frame".into(), node_i: 1, node_j: 2,
                    material_id: 1, section_id: 1, hinge_start: false, hinge_end: false,
                }),
            ]),
            materials: HashMap::from([("1".into(), SolverMaterial { id: 1, e: 200000.0, nu: 0.3 })]),
            sections: HashMap::from([("1".into(), SolverSection { id: 1, a: 0.01, iz: 0.0001, as_y: None })]),
            masses: Default::default(),
            loads: vec![],
            supports: HashMap::new(),
            constraints: vec![],
            connectors: HashMap::new(),
            solver_options: None,
        };

        let loads = vec![
            SolverLoad::Thermal(SolverThermalLoad {
                element_id: 1, dt_uniform: 20.0, dt_gradient: 0.0, alpha: None, depth: None,
            }),
        ];
        let m = build(&input, &loads).unwrap();
        assert!(m.f[2].is_none());
        assert!(m.f[5].is_none());
    }
}
