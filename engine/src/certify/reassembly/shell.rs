//! Re-assembly of plate, shell and solid elements (DKT, MITC4, MITC9,
//! solid shells, and curved shells) and their equivalent nodal loads.

use std::collections::HashMap;

use super::ia::*;
use super::system::Model;
use crate::element;
use crate::linalg::{transform_force, transform_stiffness};
use crate::types::*;

fn scatter_dense_element(
    model: &mut Model,
    dofs: &[usize],
    k_glob: &[f64],
    f_glob: &[f64],
) {
    let m = dofs.len();
    let mut k_opt = Vec::with_capacity(m * m);
    for &v in k_glob {
        if v != 0.0 {
            k_opt.push(Some(inp(v)));
        } else {
            k_opt.push(None);
        }
    }
    let mut f_opt = Vec::with_capacity(m);
    for &v in f_glob {
        if v != 0.0 {
            f_opt.push(Some(inp(v)));
        } else {
            f_opt.push(None);
        }
    }
    model.scatter(dofs, &k_opt, &f_opt, None);
}

/// Reassemble triangular DKT plates (18 DOFs per element).
pub fn reassemble_plates(
    input: &SolverInput3D,
    nodes: &HashMap<usize, &SolverNode3D>,
    mats: &HashMap<usize, &SolverMaterial>,
    by_plate: &HashMap<usize, Vec<&SolverLoad3D>>,
    model: &mut Model,
    _dpn: usize,
) -> Result<(), String> {
    let mut plates: Vec<&SolverPlateElement> = input.plates.values().collect();
    plates.sort_by_key(|p| p.id);

    for pl in plates {
        let n0 = nodes.get(&pl.nodes[0]).ok_or("plate references missing node")?;
        let n1 = nodes.get(&pl.nodes[1]).ok_or("plate references missing node")?;
        let n2 = nodes.get(&pl.nodes[2]).ok_or("plate references missing node")?;
        let coords = [[n0.x, n0.y, n0.z], [n1.x, n1.y, n1.z], [n2.x, n2.y, n2.z]];

        let mat = mats.get(&pl.material_id).ok_or("plate references missing material")?;
        let e = mat.e * 1000.0;
        let nu = mat.nu;
        let thickness = pl.thickness;

        let k_loc = element::plate_local_stiffness(&coords, e, nu, thickness);
        let t_plate = element::plate_transform_3d(&coords);
        let k_glob = transform_stiffness(&k_loc, &t_plate, 18);

        let mut f_glob = vec![0.0f64; 18];
        if let Some(loads) = by_plate.get(&pl.id) {
            for load in loads {
                match load {
                    SolverLoad3D::Pressure(p) => {
                        let fp = element::plate_pressure_load(&coords, p.pressure);
                        for (i, &v) in fp.iter().enumerate() {
                            if i < 18 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::PlateThermal(t) => {
                        let alpha = t.alpha.unwrap_or(12e-6);
                        let ft = element::plate_thermal_load(
                            &coords,
                            e,
                            nu,
                            thickness,
                            alpha,
                            t.dt_uniform,
                            t.dt_gradient,
                        );
                        for (i, &v) in ft.iter().enumerate() {
                            if i < 18 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dofs = Vec::with_capacity(18);
        for &nid in &pl.nodes {
            for d in 0..6 {
                dofs.push(model.dof(nid, d).ok_or_else(|| {
                    format!("missing DOF ({nid}, {d}) for plate {}", pl.id)
                })?);
            }
        }

        scatter_dense_element(model, &dofs, &k_glob, &f_glob);
    }
    Ok(())
}

/// Reassemble MITC4 quadrilateral shells (24 DOFs per element).
pub fn reassemble_quads(
    input: &SolverInput3D,
    nodes: &HashMap<usize, &SolverNode3D>,
    mats: &HashMap<usize, &SolverMaterial>,
    by_quad: &HashMap<usize, Vec<&SolverLoad3D>>,
    model: &mut Model,
    _dpn: usize,
) -> Result<(), String> {
    let mut quads: Vec<&SolverQuadElement> = input.quads.values().collect();
    quads.sort_by_key(|q| q.id);

    for qd in quads {
        let n0 = nodes.get(&qd.nodes[0]).ok_or("quad references missing node")?;
        let n1 = nodes.get(&qd.nodes[1]).ok_or("quad references missing node")?;
        let n2 = nodes.get(&qd.nodes[2]).ok_or("quad references missing node")?;
        let n3 = nodes.get(&qd.nodes[3]).ok_or("quad references missing node")?;
        let coords = [
            [n0.x, n0.y, n0.z],
            [n1.x, n1.y, n1.z],
            [n2.x, n2.y, n2.z],
            [n3.x, n3.y, n3.z],
        ];

        let mat = mats.get(&qd.material_id).ok_or("quad references missing material")?;
        let e = mat.e * 1000.0;
        let nu = mat.nu;
        let thickness = qd.thickness;

        let k_loc = element::quad::mitc4_local_stiffness(&coords, e, nu, thickness);
        let t_quad = element::quad::quad_transform_3d(&coords);
        let k_glob = transform_stiffness(&k_loc, &t_quad, 24);

        let mut f_glob = vec![0.0f64; 24];
        if let Some(loads) = by_quad.get(&qd.id) {
            for load in loads {
                match load {
                    SolverLoad3D::QuadPressure(p) => {
                        let fp = element::quad::quad_pressure_load(&coords, p.pressure);
                        for (i, &v) in fp.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::QuadThermal(t) => {
                        let alpha = t.alpha.unwrap_or(1.2e-5);
                        let ft = element::quad::quad_thermal_load(
                            &coords,
                            e,
                            nu,
                            thickness,
                            alpha,
                            t.dt_uniform,
                            t.dt_gradient,
                        );
                        for (i, &v) in ft.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::QuadSelfWeight(sw) => {
                        let f_sw_loc = element::quad::quad_self_weight_load(
                            &coords,
                            sw.density,
                            thickness,
                            sw.gx,
                            sw.gy,
                            sw.gz,
                        );
                        let f_sw = transform_force(&f_sw_loc, &t_quad, 24);
                        for (i, &v) in f_sw.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::QuadEdge(el) => {
                        let fe = element::quad::quad_edge_load(&coords, el.edge, el.qn, el.qt);
                        for (i, &v) in fe.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dofs = Vec::with_capacity(24);
        for &nid in &qd.nodes {
            for d in 0..6 {
                dofs.push(model.dof(nid, d).ok_or_else(|| {
                    format!("missing DOF ({nid}, {d}) for quad {}", qd.id)
                })?);
            }
        }

        scatter_dense_element(model, &dofs, &k_glob, &f_glob);
    }
    Ok(())
}

/// Reassemble MITC9 quadrilateral shells (54 DOFs per element).
pub fn reassemble_quad9s(
    input: &SolverInput3D,
    nodes: &HashMap<usize, &SolverNode3D>,
    mats: &HashMap<usize, &SolverMaterial>,
    by_quad9: &HashMap<usize, Vec<&SolverLoad3D>>,
    model: &mut Model,
    _dpn: usize,
) -> Result<(), String> {
    let mut quad9s: Vec<&SolverQuad9Element> = input.quad9s.values().collect();
    quad9s.sort_by_key(|q| q.id);

    for q9 in quad9s {
        let mut coords = [[0.0; 3]; 9];
        for (i, &nid) in q9.nodes.iter().enumerate() {
            let n = nodes.get(&nid).ok_or("quad9 references missing node")?;
            coords[i] = [n.x, n.y, n.z];
        }

        let mat = mats.get(&q9.material_id).ok_or("quad9 references missing material")?;
        let e = mat.e * 1000.0;
        let nu = mat.nu;
        let thickness = q9.thickness;

        let k_loc = element::quad9::mitc9_local_stiffness(&coords, e, nu, thickness);
        let t_q9 = element::quad9::quad9_transform_3d(&coords);
        let k_glob = transform_stiffness(&k_loc, &t_q9, 54);

        let mut f_glob = vec![0.0f64; 54];
        if let Some(loads) = by_quad9.get(&q9.id) {
            for load in loads {
                match load {
                    SolverLoad3D::Quad9Pressure(p) => {
                        let fp = element::quad9::quad9_pressure_load(&coords, p.pressure);
                        for (i, &v) in fp.iter().enumerate() {
                            if i < 54 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::Quad9Thermal(t) => {
                        let alpha = t.alpha.unwrap_or(1.2e-5);
                        let ft = element::quad9::quad9_thermal_load(
                            &coords,
                            e,
                            nu,
                            thickness,
                            alpha,
                            t.dt_uniform,
                            t.dt_gradient,
                        );
                        for (i, &v) in ft.iter().enumerate() {
                            if i < 54 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::Quad9SelfWeight(sw) => {
                        let f_sw_loc = element::quad9::quad9_self_weight_load(
                            &coords,
                            sw.density,
                            thickness,
                            sw.gx,
                            sw.gy,
                            sw.gz,
                        );
                        let f_sw = transform_force(&f_sw_loc, &t_q9, 54);
                        for (i, &v) in f_sw.iter().enumerate() {
                            if i < 54 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::Quad9Edge(el) => {
                        let fe = element::quad9::quad9_edge_load(&coords, el.edge, el.qn, el.qt);
                        for (i, &v) in fe.iter().enumerate() {
                            if i < 54 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dofs = Vec::with_capacity(54);
        for &nid in &q9.nodes {
            for d in 0..6 {
                dofs.push(model.dof(nid, d).ok_or_else(|| {
                    format!("missing DOF ({nid}, {d}) for quad9 {}", q9.id)
                })?);
            }
        }

        scatter_dense_element(model, &dofs, &k_glob, &f_glob);
    }
    Ok(())
}

/// Reassemble SHB8-ANS solid shell elements (24 DOFs per element, 3 translational DOFs per node).
pub fn reassemble_solid_shells(
    input: &SolverInput3D,
    nodes: &HashMap<usize, &SolverNode3D>,
    mats: &HashMap<usize, &SolverMaterial>,
    by_solid_shell: &HashMap<usize, Vec<&SolverLoad3D>>,
    model: &mut Model,
    _dpn: usize,
) -> Result<(), String> {
    let mut solid_shells: Vec<&SolverSolidShellElement> = input.solid_shells.values().collect();
    solid_shells.sort_by_key(|s| s.id);

    for ss in solid_shells {
        let mut coords = [[0.0; 3]; 8];
        for (i, &nid) in ss.nodes.iter().enumerate() {
            let n = nodes.get(&nid).ok_or("solid_shell references missing node")?;
            coords[i] = [n.x, n.y, n.z];
        }

        let mat = mats.get(&ss.material_id).ok_or("solid_shell references missing material")?;
        let e = mat.e * 1000.0;
        let nu = mat.nu;

        let k_elem = element::solid_shell::solid_shell_stiffness(&coords, e, nu);

        let mut f_glob = vec![0.0f64; 24];
        if let Some(loads) = by_solid_shell.get(&ss.id) {
            for load in loads {
                match load {
                    SolverLoad3D::SolidShellPressure(p) => {
                        let fp = element::solid_shell::solid_shell_pressure_load(&coords, p.pressure);
                        for (i, &v) in fp.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::SolidShellSelfWeight(sw) => {
                        let f_sw = element::solid_shell::solid_shell_self_weight_load(
                            &coords,
                            sw.density,
                            sw.gx,
                            sw.gy,
                            sw.gz,
                        );
                        for (i, &v) in f_sw.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dofs = Vec::with_capacity(24);
        for &nid in &ss.nodes {
            for d in 0..3 {
                dofs.push(model.dof(nid, d).ok_or_else(|| {
                    format!("missing DOF ({nid}, {d}) for solid_shell {}", ss.id)
                })?);
            }
        }

        scatter_dense_element(model, &dofs, &k_elem, &f_glob);
    }
    Ok(())
}

/// Reassemble curved shell elements (24 DOFs per element).
pub fn reassemble_curved_shells(
    input: &SolverInput3D,
    nodes: &HashMap<usize, &SolverNode3D>,
    mats: &HashMap<usize, &SolverMaterial>,
    by_curved_shell: &HashMap<usize, Vec<&SolverLoad3D>>,
    model: &mut Model,
    _dpn: usize,
) -> Result<(), String> {
    let mut curved_shells: Vec<&SolverCurvedShellElement> = input.curved_shells.values().collect();
    curved_shells.sort_by_key(|c| c.id);

    for cs in curved_shells {
        let mut coords = [[0.0; 3]; 4];
        for (i, &nid) in cs.nodes.iter().enumerate() {
            let n = nodes.get(&nid).ok_or("curved_shell references missing node")?;
            coords[i] = [n.x, n.y, n.z];
        }

        let mat = mats.get(&cs.material_id).ok_or("curved_shell references missing material")?;
        let e = mat.e * 1000.0;
        let nu = mat.nu;

        let dirs = cs
            .normals
            .unwrap_or_else(|| element::curved_shell::compute_element_directors(&coords));
        let k_elem = element::curved_shell::curved_shell_stiffness(
            &coords,
            &dirs,
            e,
            nu,
            cs.thickness,
        );

        let mut f_glob = vec![0.0f64; 24];
        if let Some(loads) = by_curved_shell.get(&cs.id) {
            for load in loads {
                match load {
                    SolverLoad3D::CurvedShellPressure(p) => {
                        let fp = element::curved_shell::curved_shell_pressure_load(
                            &coords,
                            &dirs,
                            cs.thickness,
                            p.pressure,
                        );
                        for (i, &v) in fp.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::CurvedShellThermal(t) => {
                        let alpha = t.alpha.unwrap_or(1.2e-5);
                        let ft = element::curved_shell::curved_shell_thermal_load(
                            &coords,
                            &dirs,
                            e,
                            nu,
                            cs.thickness,
                            alpha,
                            t.dt_uniform,
                            t.dt_gradient,
                        );
                        for (i, &v) in ft.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::CurvedShellSelfWeight(sw) => {
                        let f_sw = element::curved_shell::curved_shell_self_weight_load(
                            &coords,
                            &dirs,
                            sw.density,
                            cs.thickness,
                            sw.gx,
                            sw.gy,
                            sw.gz,
                        );
                        for (i, &v) in f_sw.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    SolverLoad3D::CurvedShellEdge(el) => {
                        let fe = element::curved_shell::curved_shell_edge_load(
                            &coords,
                            &dirs,
                            cs.thickness,
                            el.edge,
                            el.qn,
                            el.qt,
                        );
                        for (i, &v) in fe.iter().enumerate() {
                            if i < 24 {
                                f_glob[i] += v;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut dofs = Vec::with_capacity(24);
        for &nid in &cs.nodes {
            for d in 0..6 {
                dofs.push(model.dof(nid, d).ok_or_else(|| {
                    format!("missing DOF ({nid}, {d}) for curved_shell {}", cs.id)
                })?);
            }
        }

        scatter_dense_element(model, &dofs, &k_elem, &f_glob);
    }
    Ok(())
}
