//! Certified bounds on the internal-force diagrams the result views show, and
//! on their extremes. See "Diagrams, extremes, deflections and reactions" in
//! `docs/verification/CERTIFY.md`.
//!
//! The diagram evaluators (`postprocess::diagrams::compute_diagram_value_at`,
//! `postprocess::diagrams_3d::evaluate_diagram_3d_at`, and their TypeScript
//! twins) compute, at any x ∈ [0, L],
//!
//!   2D  N(x) = N_start + (loads)  [TS: (1−t)·N_start + t·(N_end + Σpx) − …]
//!       V(x) = V_start + (loads),   M(x) = M_start − V_start·x + (loads)
//!   3D  N(x) = N_start + t·(N_end − N_start) or N_start + (loads)
//!       Vy = Vy_start + …, Vz = Vz_start + …, T = Mx_start + t·(Mx_end − Mx_start)
//!       Mz(x) = Mz_start − Vy_start·x + …,   My(x) = My_start + Vz_start·x + …
//!
//! where the load terms do not depend on the displacements and the end forces
//! are `±(k_local T u_e)_i − fef`. So the error of a diagram value is
//! `r(x)·T·(u* − ũ)_e` with `r(x)` an affine function of x built from rows of
//! `k_local` (or a convex combination of two rows), and
//! `|r(x)·T·Δu| ≤ ‖r(x)‖₂·‖T‖₂·‖Δu‖₂`. `x ↦ ‖r(x)‖₂` is convex, so its sup over
//! [0, L] is attained at x = 0 or x = L, both of which are evaluated in interval
//! arithmetic. The bound therefore holds at **every** x of the member, not
//! only at stations, and for both the Rust and the TypeScript evaluator.
//!
//! **Extremes.** max, min and max-abs over any fixed set of stations are
//! 1-Lipschitz in the sup norm, so the station bound also bounds the error of
//! the displayed extremes. The sup of the *computed* diagram between stations
//! can exceed the station extreme; `*_gap` bounds by how much (see
//! [`diagram_gaps_2d`]): `sup_x f̃ ≤ max_stations f̃ + gap`. Hence the exact
//! continuous extreme lies in `[shown − bound, shown + bound + gap]`. Where the
//! extreme occurs (argmax) is not certified: it is not a continuous function of
//! the forces.
//!
//! As for the end forces, rounding in the f64 evaluation of the diagram
//! formulas is not included.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::forces::{local_stiffness_2d, local_stiffness_3d, row_norm, up, LocalK, T_NORM};
use super::interval::Interval;
use crate::postprocess::diagrams::{compute_diagram_value_at_sorted, diagram_positions_2d, sorted_point_loads};
use crate::postprocess::diagrams_3d::diagram_positions_3d;
use crate::types::*;

/// Bounds on |exact − computed| valid at every x along one 2D member, for the
/// N, V, M diagrams (kN, kN·m); hence also on their extremes over any stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationBound2D {
    pub element_id: usize,
    pub axial: f64,
    pub shear: f64,
    pub moment: f64,
    /// `sup_x f̃(x) − max_stations f̃` (and likewise for min, max-abs) of the
    /// computed diagram over the stations `compute_diagrams_2d` samples. Absent
    /// when not computed (envelopes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gaps: Option<DiagramGaps2D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramGaps2D {
    pub axial: f64,
    pub shear: f64,
    pub moment: f64,
}

/// Bounds valid at every x along one 3D member (local axes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationBound3D {
    pub element_id: usize,
    pub axial: f64,
    pub shear_y: f64,
    pub shear_z: f64,
    pub torsion: f64,
    pub moment_y: f64,
    pub moment_z: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gaps: Option<DiagramGaps3D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramGaps3D {
    pub axial: f64,
    pub shear_y: f64,
    pub shear_z: f64,
    pub torsion: f64,
    pub moment_y: f64,
    pub moment_z: f64,
}

// ───────────────────────────── station bounds ─────────────────────────────

/// `sup_{x∈[0,L]} ‖row_a + s·x·row_b‖₂`, rounded up: the norm is convex in x,
/// so the sup is the larger of the values at x = 0 and x = L (the latter
/// evaluated in interval arithmetic).
fn affine_row_sup(lk: &LocalK, a: usize, b: usize, s: f64) -> f64 {
    let (ra, rb) = (lk.row(a), lk.row(b));
    let at0 = row_norm(ra, ra.len(), 0);
    let sl = Interval::point(s) * Interval::point(lk.length);
    let mut sq = 0.0f64;
    for (&x, &y) in ra.iter().zip(rb) {
        let m = (Interval::point(x) + sl * Interval::point(y)).mag();
        sq = up(sq + up(m * m));
    }
    let at_l = up(sq.sqrt());
    if at_l.is_finite() { at0.max(at_l) } else { f64::INFINITY }
}

/// `max over rows of ‖row‖₂` (a convex combination of rows is bounded by it).
fn rows_sup(lk: &LocalK, rows: &[usize]) -> f64 {
    rows.iter().map(|&i| row_norm(lk.row(i), lk.row(i).len(), 0)).fold(0.0, f64::max)
}

fn scale(r: f64, abs_bound: f64) -> f64 {
    up(up(r * T_NORM) * abs_bound)
}

/// Per-member diagram bounds for a 2D model, given `‖u* − ũ‖₂ ≤ abs_bound`
/// (no gaps; see [`with_gaps_2d`]).
pub fn station_bounds_2d(input: &SolverInput, abs_bound: f64) -> Vec<StationBound2D> {
    station_rows_2d(input).into_iter().map(|(eid, r)| station_from_rows_2d(eid, &r, abs_bound)).collect()
}

/// The `sup_x ‖r(x)‖₂` of each 2D diagram (axial, shear, moment) per member;
/// multiply by `‖T‖·abs_bound` for the bound.
pub(crate) fn station_rows_2d(input: &SolverInput) -> Vec<(usize, [f64; 3])> {
    local_stiffness_2d(input)
        .iter()
        .map(|lk| {
            if lk.truss {
                return (lk.element_id, [rows_sup(lk, &[0]), 0.0, 0.0]);
            }
            // n_start = −f0, n_end = f3, v_start = f1, m_start = f2: M(x) = f2 − x·f1.
            (lk.element_id, [rows_sup(lk, &[0, 3]), rows_sup(lk, &[1]), affine_row_sup(lk, 2, 1, -1.0)])
        })
        .collect()
}

pub(crate) fn station_from_rows_2d(eid: usize, r: &[f64; 3], abs_bound: f64) -> StationBound2D {
    StationBound2D {
        element_id: eid,
        axial: scale(r[0], abs_bound),
        shear: scale(r[1], abs_bound),
        moment: scale(r[2], abs_bound),
        gaps: None,
    }
}

/// Per-member diagram bounds for a 3D model (frames, warping frames, trusses),
/// given `‖u* − ũ‖₂ ≤ abs_bound` (no gaps; see [`with_gaps_3d`]).
pub fn station_bounds_3d(input: &SolverInput3D, abs_bound: f64) -> Vec<StationBound3D> {
    station_rows_3d(input).into_iter().map(|(eid, r)| station_from_rows_3d(eid, &r, abs_bound)).collect()
}

/// `sup_x ‖r(x)‖₂` of (axial, shear_y, shear_z, torsion, moment_y, moment_z).
pub(crate) fn station_rows_3d(input: &SolverInput3D) -> Vec<(usize, [f64; 6])> {
    local_stiffness_3d(input)
        .iter()
        .map(|lk| {
            if lk.truss {
                return (lk.element_id, [rows_sup(lk, &[0]), 0.0, 0.0, 0.0, 0.0, 0.0]);
            }
            // Start rows 0..5, end rows j..j+5 (14-DOF warping: j = 7).
            let j = if lk.n == 14 { 7 } else { 6 };
            (
                lk.element_id,
                [
                    rows_sup(lk, &[0, j]),
                    rows_sup(lk, &[1]),
                    rows_sup(lk, &[2]),
                    rows_sup(lk, &[3, j + 3]),
                    // My(x) = my_start + vz_start·x = f4 + x·f2
                    affine_row_sup(lk, 4, 2, 1.0),
                    // Mz(x) = mz_start − vy_start·x = f5 − x·f1
                    affine_row_sup(lk, 5, 1, -1.0),
                ],
            )
        })
        .collect()
}

pub(crate) fn station_from_rows_3d(eid: usize, r: &[f64; 6], abs_bound: f64) -> StationBound3D {
    StationBound3D {
        element_id: eid,
        axial: scale(r[0], abs_bound),
        shear_y: scale(r[1], abs_bound),
        shear_z: scale(r[2], abs_bound),
        torsion: scale(r[3], abs_bound),
        moment_y: scale(r[4], abs_bound),
        moment_z: scale(r[5], abs_bound),
        gaps: None,
    }
}

// ───────────────────────────── gaps between stations ─────────────────────────────

/// A linearly varying load `q(x)` on `[a, b]`.
#[derive(Clone, Copy)]
struct Dist {
    qi: f64,
    qj: f64,
    a: f64,
    b: f64,
}

impl Dist {
    fn qmax(&self) -> f64 {
        self.qi.abs().max(self.qj.abs())
    }
    fn slope(&self) -> f64 {
        (self.qj - self.qi).abs() / (self.b - self.a)
    }
}

/// What makes a diagram non-smooth at a point: a jump in slope (`kink`) or in
/// value (`jump`).
#[derive(Clone, Copy)]
struct Event {
    x: f64,
    kink: f64,
    jump: f64,
}

/// A diagram on one member, as far as its smoothness goes: on each smooth
/// piece `|f''| ≤ curvature(loads acting there)`, slope and value jumps at the
/// events, and a bound on `|f'|` over a station interval (`lip`).
struct Shape<'a> {
    dists: Vec<Dist>,
    events: Vec<Event>,
    curvature: fn(&Dist) -> f64,
    lip: Box<dyn Fn(usize, f64) -> f64 + 'a>,
}

/// `max_k G_k` over consecutive stations `x_k < x_{k+1}` (h = x_{k+1} − x_k) with
///
///   G_k = K·h²/8 + Σ|S_i|·ρ_i + min(Σ|J_i|, D·h),
///
/// K bounding |f''| on the smooth pieces, D bounding |f'| there, `S_i` the
/// slope jumps at `c_i` inside the interval with `ρ_i = min(c_i − x_k, x_{k+1} −
/// c_i, h/4)`, `J_i` the value jumps inside it. Then `f(x) ≤ max(f(x_k),
/// f(x_{k+1})) + G_k` on the interval, so `sup f ≤ max_stations f + max_k G_k`
/// (and `inf f ≥ min − …`). Event positions are widened by a tolerance covering
/// the evaluators' `± 1e-10` comparisons.
///
/// Proof. Write `f = c + Σ J_i·H(x − c_i)` with H the unit step, so `c` is
/// continuous, C¹ except at the kinks, `|c''| ≤ K` elsewhere and `|c'| ≤ D`.
/// With ℓ the chord of `c` through the stations, `c − ℓ = g + Σ S_i ρ_i(x)` where
/// g is C¹ with `|g''| ≤ K`, zero at both ends (`|g| ≤ K h²/8`), and `ρ_i(x)` is
/// the chord error of `(x − c_i)₊`, at most `(c_i − x_k)(x_{k+1} − c_i)/h ≤ ρ_i`.
/// So `c ≤ max(c(x_k), c(x_{k+1})) + K h²/8 + Σ|S_i|ρ_i`. Adding the steps:
/// either bound each by its chord error (`≤ |J|`), or note `|c(x_k) − c(x_{k+1})|
/// ≤ D h` while `f(x_k)` and `f(x_{k+1})` differ from `c` by the steps before
/// x_k and by all steps.
fn gap(stations: &[f64], shape: &Shape) -> f64 {
    let mut g = 0.0f64;
    for (k, w) in stations.windows(2).enumerate() {
        let (x0, x1) = (w[0], w[1]);
        let h = up(x1 - x0).max(0.0);
        let tol = 1e-9 * (1.0 + x1.abs());
        let curv: f64 = shape
            .dists
            .iter()
            .filter(|d| d.a < x1 + tol && d.b > x0 - tol)
            .map(|d| (shape.curvature)(d))
            .sum();
        let (mut kinks, mut jumps) = (0.0, 0.0);
        for e in &shape.events {
            let rho = (e.x + tol - x0).min(x1 - e.x + tol).min(h / 4.0);
            if rho > 0.0 {
                kinks += e.kink.abs() * rho;
            }
            if e.x + tol >= x0 && e.x - tol < x1 {
                jumps += e.jump.abs();
            }
        }
        let step = if jumps > 0.0 { jumps.min((shape.lip)(k, h) * h) } else { 0.0 };
        let gk = curv * h * h / 8.0 + kinks + step;
        // Generous outward rounding of the few f64 operations above.
        g = g.max(gk * (1.0 + 1e-12));
    }
    if g.is_finite() { g } else { f64::INFINITY }
}

fn dists_of(loads: &[DistributedLoadInfo]) -> Vec<Dist> {
    loads
        .iter()
        .filter(|d| d.b - d.a >= 1e-12)
        .map(|d| Dist { qi: d.q_i, qj: d.q_j, a: d.a, b: d.b })
        .collect()
}

/// Slope jumps of a shear-type diagram (`f' = ±q`) where a load starts and ends.
fn load_end_kinks(dists: &[Dist]) -> Vec<Event> {
    dists
        .iter()
        .flat_map(|d| [Event { x: d.a, kink: d.qi, jump: 0.0 }, Event { x: d.b, kink: d.qj, jump: 0.0 }])
        .collect()
}

fn xs(ef_len: f64, ts: &[f64]) -> Vec<f64> {
    ts.iter().map(|t| t * ef_len).collect()
}

/// Gaps (axial, shear, moment) of the computed 2D diagrams of one member over
/// the stations `compute_diagrams_2d` samples.
pub fn diagram_gaps_2d(ef: &ElementForces) -> DiagramGaps2D {
    let l = ef.length;
    let mut dists = dists_of(&ef.distributed_loads);
    if dists.is_empty() && (ef.q_i.abs() > 1e-10 || ef.q_j.abs() > 1e-10) && l >= 1e-12 {
        dists.push(Dist { qi: ef.q_i, qj: ef.q_j, a: 0.0, b: l });
    }
    let pls = sorted_point_loads(ef);
    let q_on = |x0: f64, x1: f64| -> f64 {
        dists.iter().filter(|d| d.a <= x1 && d.b >= x0).map(Dist::qmax).sum()
    };
    let p_on = |x0: f64, x1: f64| -> f64 {
        pls.iter().filter(|p| p.a >= x0 && p.a <= x1).map(|p| p.p.abs()).sum()
    };

    // M: M'' = −q, kinks |P|, jumps at couples; |M'| = |V| ≤ |V(x_k)|, |V(x_{k+1})| + q·h + Σ|P|.
    let mt = diagram_positions_2d(ef, "moment");
    let mx = xs(l, &mt);
    let v_at: Vec<f64> = mt.iter().map(|&t| compute_diagram_value_at_sorted("shear", t, ef, &pls)).collect();
    let moment = gap(
        &mx,
        &Shape {
            dists: dists.clone(),
            events: pls.iter().map(|p| Event { x: p.a, kink: p.p, jump: p.my.unwrap_or(0.0) }).collect(),
            curvature: Dist::qmax,
            lip: Box::new(|k, h| {
                let tol = 1e-9 * (1.0 + mx[k + 1].abs());
                v_at[k].abs().max(v_at[k + 1].abs()) + q_on(mx[k] - tol, mx[k + 1] + tol) * h
                    + p_on(mx[k] - tol, mx[k + 1] + tol)
            }),
        },
    );

    // V: V'' = q' on pieces, kinks where loads start/end, jumps |P|; |V'| = |q|.
    let vt = diagram_positions_2d(ef, "shear");
    let vx = xs(l, &vt);
    let mut ev = load_end_kinks(&dists);
    ev.extend(pls.iter().map(|p| Event { x: p.a, kink: 0.0, jump: p.p }));
    let shear = gap(
        &vx,
        &Shape {
            dists: dists.clone(),
            events: ev,
            curvature: Dist::slope,
            lip: Box::new(|k, _| {
                let tol = 1e-9 * (1.0 + vx[k + 1].abs());
                q_on(vx[k] - tol, vx[k + 1] + tol)
            }),
        },
    );

    // N: constant between axial point loads (which only jump it), or N' = −q_x with kinks
    // where an axial span load starts and ends (as the 3D gap reads `distributed_loads_x`).
    let dx = dists_of(&ef.distributed_loads_x);
    // Axial point loads jump it; counting them as events keeps the bound whether or not the
    // stations straddle them.
    let mut ev = load_end_kinks(&dx);
    ev.extend(pls.iter().filter_map(|p| p.px.filter(|px| *px != 0.0).map(|px| Event { x: p.a, kink: 0.0, jump: px })));
    // |N'| = |q_x|: bounded per interval by the axial span loads acting on it. A zero bound
    // here would make `gap` drop the point-load jumps (it takes min(Σ|J|, D·h)).
    let axial = if dx.is_empty() {
        0.0
    } else {
        let ax = xs(l, &diagram_positions_2d(ef, "axial"));
        let (lx, ld) = (ax.clone(), dx.clone());
        let lip_ax = move |k: usize, _h: f64| -> f64 {
            let tol = 1e-9 * (1.0 + lx[k + 1].abs());
            let (x0, x1) = (lx[k] - tol, lx[k + 1] + tol);
            ld.iter().filter(|d| d.a <= x1 && d.b >= x0).map(Dist::qmax).sum()
        };
        gap(&ax, &Shape { events: ev, dists: dx, curvature: Dist::slope, lip: Box::new(lip_ax) })
    };
    DiagramGaps2D { axial, shear, moment }
}

/// Gaps of the computed 3D diagrams of one member over the stations
/// `compute_diagrams_3d` samples.
pub fn diagram_gaps_3d(ef: &ElementForces3D) -> DiagramGaps3D {
    let l = ef.length;
    let plane = |dl: &[DistributedLoadInfo], pl: &[PointLoadInfo3D], mkind: &str, vkind: &str| -> (f64, f64) {
        let dists = dists_of(dl);
        let q_on = |x0: f64, x1: f64| -> f64 { dists.iter().filter(|d| d.a <= x1 && d.b >= x0).map(Dist::qmax).sum() };
        let mx = xs(l, &diagram_positions_3d(ef, mkind));
        let moment = gap(
            &mx,
            &Shape {
                dists: dists.clone(),
                events: pl.iter().map(|p| Event { x: p.a, kink: p.p, jump: 0.0 }).collect(),
                curvature: Dist::qmax,
                lip: Box::new(|_, _| 0.0), // no value jumps
            },
        );
        let vx = xs(l, &diagram_positions_3d(ef, vkind));
        let mut ev = load_end_kinks(&dists);
        ev.extend(pl.iter().map(|p| Event { x: p.a, kink: 0.0, jump: p.p }));
        let shear = gap(
            &vx,
            &Shape {
                dists: dists.clone(),
                events: ev,
                curvature: Dist::slope,
                lip: Box::new(|k, _| {
                    let tol = 1e-9 * (1.0 + vx[k + 1].abs());
                    q_on(vx[k] - tol, vx[k + 1] + tol)
                }),
            },
        );
        (moment, shear)
    };
    let (moment_z, shear_y) = plane(&ef.distributed_loads_y, &ef.point_loads_y, "momentZ", "shearY");
    let (moment_y, shear_z) = plane(&ef.distributed_loads_z, &ef.point_loads_z, "momentY", "shearZ");
    // N: linear between the ends, or N' = −q_x with kinks where the loads start/end.
    let dx = dists_of(&ef.distributed_loads_x);
    let axial = if dx.is_empty() {
        0.0
    } else {
        gap(
            &xs(l, &diagram_positions_3d(ef, "axial")),
            &Shape { events: load_end_kinks(&dx), dists: dx, curvature: Dist::slope, lip: Box::new(|_, _| 0.0) },
        )
    };
    DiagramGaps3D { axial, shear_y, shear_z, torsion: 0.0, moment_y, moment_z }
}

/// Attach the gaps of the computed diagrams (`forces`) to station bounds.
pub fn with_gaps_2d(bounds: &mut [StationBound2D], forces: &[ElementForces]) {
    let by_id: HashMap<usize, &ElementForces> = forces.iter().map(|f| (f.element_id, f)).collect();
    for b in bounds {
        b.gaps = by_id.get(&b.element_id).map(|ef| diagram_gaps_2d(ef));
    }
}

/// 3D version of [`with_gaps_2d`].
pub fn with_gaps_3d(bounds: &mut [StationBound3D], forces: &[ElementForces3D]) {
    let by_id: HashMap<usize, &ElementForces3D> = forces.iter().map(|f| (f.element_id, f)).collect();
    for b in bounds {
        b.gaps = by_id.get(&b.element_id).map(|ef| diagram_gaps_3d(ef));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        let r = 2.0;
        let abs_bound = 3.0;
        let s = scale(r, abs_bound);
        let expected = up(up(2.0 * T_NORM) * 3.0);
        assert!((s - expected).abs() < 1e-12);
        assert!(scale(0.0, 3.0) < 1e-150);
    }

    #[test]
    fn test_dist_methods_and_kinks() {
        let d = Dist { qi: -10.0, qj: 4.0, a: 1.0, b: 4.0 };
        assert_eq!(d.qmax(), 10.0);
        assert!((d.slope() - 14.0 / 3.0).abs() < 1e-12);

        let dl = vec![
            DistributedLoadInfo { q_i: 1.0, q_j: 2.0, a: 0.0, b: 1.0 },
            DistributedLoadInfo { q_i: 3.0, q_j: 4.0, a: 1.0, b: 1.0 + 1e-14 },
        ];
        let dists = dists_of(&dl);
        assert_eq!(dists.len(), 1);
        assert_eq!(dists[0].qi, 1.0);
        assert_eq!(dists[0].b, 1.0);

        let kinks = load_end_kinks(&dists);
        assert_eq!(kinks.len(), 2);
        assert_eq!(kinks[0].x, 0.0);
        assert_eq!(kinks[0].kink, 1.0);
        assert_eq!(kinks[1].x, 1.0);
        assert_eq!(kinks[1].kink, 2.0);
    }

    #[test]
    fn test_gap_curvature_kinks_and_jumps() {
        let stations = vec![0.0, 2.0];
        let shape_curv = Shape {
            dists: vec![Dist { qi: 4.0, qj: 4.0, a: 0.0, b: 2.0 }],
            events: vec![],
            curvature: Dist::qmax,
            lip: Box::new(|_, _| 0.0),
        };
        let g = gap(&stations, &shape_curv);
        assert!(g >= 2.0 && g < 2.0000001);

        let shape_kink = Shape {
            dists: vec![],
            events: vec![Event { x: 1.0, kink: 8.0, jump: 0.0 }],
            curvature: Dist::qmax,
            lip: Box::new(|_, _| 0.0),
        };
        let gk = gap(&stations, &shape_kink);
        assert!(gk >= 4.0 && gk < 4.0000001);

        let shape_jump = Shape {
            dists: vec![],
            events: vec![Event { x: 1.0, kink: 0.0, jump: 3.0 }],
            curvature: Dist::qmax,
            lip: Box::new(|_, _h| 1.0),
        };
        let gj = gap(&stations, &shape_jump);
        assert!(gj >= 2.0 && gj < 2.0000001);

        let shape_outside = Shape {
            dists: vec![],
            events: vec![Event { x: 3.0, kink: 8.0, jump: 3.0 }],
            curvature: Dist::qmax,
            lip: Box::new(|_, _| 1.0),
        };
        let go = gap(&stations, &shape_outside);
        assert_eq!(go, 0.0);
    }

    #[test]
    fn test_diagram_gaps_2d_and_with_gaps() {
        let mut ef = ElementForces {
            element_id: 1,
            length: 4.0,
            n_start: 0.0,
            n_end: 0.0,
            v_start: 0.0,
            v_end: 0.0,
            m_start: 0.0,
            m_end: 0.0,
            hinge_start: false,
            hinge_end: false,
            q_i: 2.0,
            q_j: 5.0,
            distributed_loads: vec![DistributedLoadInfo { q_i: 2.0, q_j: 5.0, a: 0.0, b: 4.0 }],
            distributed_loads_x: vec![DistributedLoadInfo { q_i: 1.0, q_j: 1.0, a: 0.0, b: 4.0 }],
            point_loads: vec![
                PointLoadInfo { a: 2.0, p: 10.0, my: Some(5.0), px: Some(3.0) }
            ],
        };
        let gaps = diagram_gaps_2d(&ef);
        assert!(gaps.axial > 0.0);
        assert!(gaps.shear > 0.0);
        assert!(gaps.moment > 0.0);

        ef.distributed_loads.clear();
        let gaps_fallback = diagram_gaps_2d(&ef);
        assert!(gaps_fallback.moment > 0.0);

        let mut bounds = vec![
            StationBound2D { element_id: 1, axial: 1.0, shear: 1.0, moment: 1.0, gaps: None },
            StationBound2D { element_id: 2, axial: 1.0, shear: 1.0, moment: 1.0, gaps: None },
        ];
        with_gaps_2d(&mut bounds, &[ef]);
        assert!(bounds[0].gaps.is_some());
        assert!(bounds[1].gaps.is_none());
    }

    #[test]
    fn test_diagram_gaps_3d_and_with_gaps() {
        let json = r##"{
            "elementId": 42,
            "length": 3.0,
            "nStart": 0.0, "nEnd": 0.0,
            "vyStart": 0.0, "vyEnd": 0.0,
            "vzStart": 0.0, "vzEnd": 0.0,
            "mxStart": 0.0, "mxEnd": 0.0,
            "myStart": 0.0, "myEnd": 0.0,
            "mzStart": 0.0, "mzEnd": 0.0,
            "qYI": 4.0, "qYJ": 4.0,
            "qZI": 1.0, "qZJ": 3.0,
            "distributedLoadsX": [{"qI": 2.0, "qJ": 2.0, "a": 0.0, "b": 3.0}],
            "distributedLoadsY": [{"qI": 4.0, "qJ": 4.0, "a": 0.0, "b": 3.0}],
            "distributedLoadsZ": [{"qI": 1.0, "qJ": 3.0, "a": 0.0, "b": 3.0}],
            "pointLoadsY": [{"a": 1.5, "p": 6.0}],
            "pointLoadsZ": [{"a": 1.0, "p": 4.0}]
        }"##;
        let ef3d: ElementForces3D = serde_json::from_str(json).unwrap();
        let gaps = diagram_gaps_3d(&ef3d);
        assert!(gaps.axial > 0.0);
        assert!(gaps.shear_y > 0.0);
        assert!(gaps.shear_z > 0.0);
        assert!(gaps.moment_y > 0.0);
        assert!(gaps.moment_z > 0.0);
        assert_eq!(gaps.torsion, 0.0);

        let mut bounds = vec![StationBound3D {
            element_id: 42,
            axial: 1.0,
            shear_y: 1.0,
            shear_z: 1.0,
            torsion: 1.0,
            moment_y: 1.0,
            moment_z: 1.0,
            gaps: None,
        }];
        with_gaps_3d(&mut bounds, &[ef3d]);
        assert!(bounds[0].gaps.is_some());
    }

    #[test]
    fn test_station_rows_sign() {
        let json = r#"{
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "y": 0.0, "z": 0.0}},
            "materials": {"1": {"id": 1, "e": 200.0, "nu": 0.3, "density": 7850.0}},
            "sections": {"1": {"id": 1, "a": 0.01, "iy": 0.0001, "iz": 0.0002, "j": 0.00005}},
            "elements": {
                "1": {"id": 1, "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1, "type": "frame", "releaseMzStart": false, "releaseMzEnd": false, "releaseMyStart": false, "releaseMyEnd": false}
            },
            "supports": {}, "loads": []
        }"#;
        let input: SolverInput3D = serde_json::from_str(json).unwrap();
        let rows = station_rows_3d(&input);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 1);
        let [n, vy, vz, t, my, mz] = rows[0].1;
        assert!(n > 0.0);
        assert!(vy > 0.0);
        assert!(vz > 0.0);
        assert!(t > 0.0);
        assert!(my > 0.0);
        assert!(mz > 0.0);
    }


    #[test]
    fn test_mutants_issue_751_stations() {
        assert!(scale(0.0, 5.0) <= 1e-15);
        let s = scale(2.0, 3.0);
        let exp = up(up(2.0 * T_NORM) * 3.0);
        assert!((s - exp).abs() < 1e-12);

        let d = Dist { qi: -5.0, qj: 3.0, a: 1.0, b: 5.0 };
        assert_eq!(d.qmax(), 5.0);
        assert!((d.slope() - 2.0).abs() < 1e-12);

        let dl = vec![
            DistributedLoadInfo { q_i: 1.0, q_j: 2.0, a: 0.0, b: 2.0 },
            DistributedLoadInfo { q_i: 3.0, q_j: 4.0, a: 2.0, b: 2.0 + 1e-14 },
        ];
        let dists = dists_of(&dl);
        assert_eq!(dists.len(), 1);
        let kinks = load_end_kinks(&dists);
        assert_eq!(kinks.len(), 2);

        let stations = vec![0.0, 2.0];
        let shape = Shape {
            dists: vec![Dist { qi: 4.0, qj: 4.0, a: 0.0, b: 2.0 }],
            events: vec![
                Event { x: 1.0, kink: 4.0, jump: 2.0 },
                Event { x: 0.0, kink: 2.0, jump: 0.0 },
                Event { x: 2.0, kink: 0.0, jump: 1.0 },
                Event { x: 5.0, kink: 8.0, jump: 8.0 },
            ],
            curvature: Dist::qmax,
            lip: Box::new(|_, _h| 1.5),
        };
        let g = gap(&stations, &shape);
        assert!(g > 0.0 && g.is_finite());

        let json2d = r#"{
            "elementId": 1, "length": 4.0,
            "nStart": 0.0, "nEnd": 0.0, "vStart": 0.0, "vEnd": 0.0, "mStart": 0.0, "mEnd": 0.0,
            "hingeStart": false, "hingeEnd": false,
            "qI": 2.0, "qJ": 2.0,
            "distributedLoads": [{"qI": 2.0, "qJ": 2.0, "a": 0.0, "b": 4.0}],
            "distributedLoadsX": [{"qI": 1.0, "qJ": 1.0, "a": 0.0, "b": 4.0}],
            "pointLoads": [{"a": 2.0, "p": 5.0}]
        }"#;
        let ef2d: ElementForces = serde_json::from_str(json2d).unwrap();
        let gaps2d = diagram_gaps_2d(&ef2d);
        assert!(gaps2d.axial > 0.0);
        assert!(gaps2d.shear > 0.0);
        assert!(gaps2d.moment > 0.0);

        let mut b2d = vec![StationBound2D { element_id: 1, axial: 1.0, shear: 1.0, moment: 1.0, gaps: None }];
        with_gaps_2d(&mut b2d, &[ef2d]);
        assert!(b2d[0].gaps.is_some());

        let json3d = r#"{
            "elementId": 1, "length": 3.0,
            "nStart": 0.0, "nEnd": 0.0, "vyStart": 0.0, "vyEnd": 0.0, "vzStart": 0.0, "vzEnd": 0.0,
            "mxStart": 0.0, "mxEnd": 0.0, "myStart": 0.0, "myEnd": 0.0, "mzStart": 0.0, "mzEnd": 0.0,
            "qYI": 2.0, "qYJ": 2.0, "qZI": 1.0, "qZJ": 1.0,
            "distributedLoadsX": [{"qI": 1.0, "qJ": 1.0, "a": 0.0, "b": 3.0}],
            "distributedLoadsY": [{"qI": 2.0, "qJ": 2.0, "a": 0.0, "b": 3.0}],
            "distributedLoadsZ": [{"qI": 1.0, "qJ": 1.0, "a": 0.0, "b": 3.0}],
            "pointLoadsY": [{"a": 1.5, "p": 4.0}],
            "pointLoadsZ": [{"a": 1.0, "p": 3.0}]
        }"#;
        let ef3d: ElementForces3D = serde_json::from_str(json3d).unwrap();
        let gaps3d = diagram_gaps_3d(&ef3d);
        assert!(gaps3d.axial > 0.0);
        assert!(gaps3d.shear_y > 0.0);
        assert!(gaps3d.shear_z > 0.0);
        assert!(gaps3d.moment_y > 0.0);
        assert!(gaps3d.moment_z > 0.0);
        assert_eq!(gaps3d.torsion, 0.0);

        let mut b3d = vec![StationBound3D {
            element_id: 1,
            axial: 1.0, shear_y: 1.0, shear_z: 1.0,
            torsion: 1.0, moment_y: 1.0, moment_z: 1.0,
            gaps: None,
        }];
        with_gaps_3d(&mut b3d, &[ef3d]);
        assert!(b3d[0].gaps.is_some());

        let json_model3d = r#"{
            "nodes": {"1": {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0}, "2": {"id": 2, "x": 3.0, "y": 0.0, "z": 0.0}},
            "materials": {"1": {"id": 1, "e": 200.0, "nu": 0.3, "density": 7850.0}},
            "sections": {"1": {"id": 1, "a": 0.01, "iy": 0.0001, "iz": 0.0002, "j": 0.00005}},
            "elements": {
                "1": {"id": 1, "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1, "type": "frame", "releaseMzStart": false, "releaseMzEnd": false, "releaseMyStart": false, "releaseMyEnd": false}
            },
            "supports": {}, "loads": []
        }"#;
        let in3d: SolverInput3D = serde_json::from_str(json_model3d).unwrap();
        let rows = station_rows_3d(&in3d);
        assert_eq!(rows.len(), 1);
        let [n, vy, vz, t, my, mz] = rows[0].1;
        assert!(n > 0.0 && vy > 0.0 && vz > 0.0 && t > 0.0 && my > 0.0 && mz > 0.0);
    }

}
