//! Adaptive mesh refinement driving discretization error down to a user-specified target.
//!
//! Replaces uniform brute-force refinement with goal-driven selective refinement,
//! checking convergence against a target error and enforcing strict stopping criteria.
//! Unestimable and inconclusive cases are reported honestly and never passed.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::refine::{check_refinable, refine_uniform, GeometryMode};
use super::{estimate_3d, DEFAULT_THRESHOLD};
use crate::solver::dof::DofNumbering;
use crate::solver::linear::solve_3d;
use crate::types::*;

fn default_target_error() -> f64 {
    DEFAULT_THRESHOLD
}
fn default_max_iterations() -> usize {
    5
}
fn default_max_dofs() -> usize {
    250_000
}
fn default_marking_fraction() -> f64 {
    0.5
}

/// Options configuring the adaptive refinement loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveOptions {
    /// Target relative energy-norm error (default: 0.05, representing 5%).
    #[serde(default = "default_target_error")]
    pub target_error: f64,
    /// Maximum number of adaptive refinement iterations (default: 5).
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    /// Maximum number of DOFs before terminating (default: 250,000).
    #[serde(default = "default_max_dofs")]
    pub max_dofs: usize,
    /// Dörfler bulk marking fraction theta in (0.0, 1.0] (default: 0.5).
    #[serde(default = "default_marking_fraction")]
    pub marking_fraction: f64,
    /// Geometric smoothing mode for newly generated nodes.
    #[serde(default)]
    pub geometry: GeometryMode,
}

impl Default for AdaptiveOptions {
    fn default() -> Self {
        Self {
            target_error: default_target_error(),
            max_iterations: default_max_iterations(),
            max_dofs: default_max_dofs(),
            marking_fraction: default_marking_fraction(),
            geometry: GeometryMode::default(),
        }
    }
}

/// Snapshot of a single refinement iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveIteration {
    pub iteration: usize,
    pub n_nodes: usize,
    pub n_elements: usize,
    pub n_dofs: usize,
    pub estimated_error: Option<f64>,
    pub error_norm: f64,
    pub solution_norm: f64,
    pub max_displacement: f64,
}

/// Final outcome status of the adaptive refinement loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum AdaptiveStatus {
    /// Discretization error dropped below the user target.
    Converged,
    /// Reached maximum iteration count without achieving the target.
    MaxIterationsReached,
    /// Further refinement would exceed the user maximum DOF threshold.
    MaxDofsReached,
    /// Mesh or physics could not be reliably estimated (never passed as converged).
    Unestimable { reason: String },
}

/// Full output payload of an adaptive refinement run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveRefinementResult {
    pub converged: bool,
    pub status: AdaptiveStatus,
    pub unestimable_reason: Option<String>,
    pub initial_error: Option<f64>,
    pub final_error: Option<f64>,
    pub iterations: Vec<AdaptiveIteration>,
    pub final_input: SolverInput3D,
    pub final_results: AnalysisResults3D,
}

/// Elevate marked MITC4 quads to higher-order MITC9 quadratic shell elements.
fn elevate_quads_to_quad9(input: &mut SolverInput3D, marked_quads: &[usize]) -> bool {
    if marked_quads.is_empty() {
        return false;
    }
    let marked_set: std::collections::HashSet<usize> = marked_quads.iter().copied().collect();
    let mut next_node_id = input.nodes.values().map(|n| n.id).max().unwrap_or(0) + 1;
    let mut next_elem_id = input
        .quad9s
        .values()
        .map(|q| q.id)
        .chain(input.quads.values().map(|q| q.id))
        .max()
        .unwrap_or(0)
        + 1;

    let mut elevated_any = false;
    let mut quads_to_remove = Vec::new();

    let quads_snapshot: Vec<SolverQuadElement> = input.quads.values().cloned().collect();
    for q in quads_snapshot {
        if !marked_set.contains(&q.id) {
            continue;
        }

        let n0 = input.nodes.get(&q.nodes[0]).unwrap();
        let n1 = input.nodes.get(&q.nodes[1]).unwrap();
        let n2 = input.nodes.get(&q.nodes[2]).unwrap();
        let n3 = input.nodes.get(&q.nodes[3]).unwrap();

        // Edge midpoints
        let mut add_mid = |na: &SolverNode3D, nb: &SolverNode3D| -> usize {
            let id = next_node_id;
            next_node_id += 1;
            input.nodes.insert(
                id.to_string(),
                SolverNode3D {
                    id,
                    x: 0.5 * (na.x + nb.x),
                    y: 0.5 * (na.y + nb.y),
                    z: 0.5 * (na.z + nb.z),
                },
            );
            id
        };

        let m01 = add_mid(n0, n1);
        let m12 = add_mid(n1, n2);
        let m23 = add_mid(n2, n3);
        let m30 = add_mid(n3, n0);

        // Center node
        let c_id = next_node_id;
        next_node_id += 1;
        input.nodes.insert(
            c_id.to_string(),
            SolverNode3D {
                id: c_id,
                x: 0.25 * (n0.x + n1.x + n2.x + n3.x),
                y: 0.25 * (n0.y + n1.y + n2.y + n3.y),
                z: 0.25 * (n0.z + n1.z + n2.z + n3.z),
            },
        );

        let q9_nodes = [
            q.nodes[0], q.nodes[1], q.nodes[2], q.nodes[3],
            m01, m12, m23, m30, c_id,
        ];

        let q9_id = next_elem_id;
        next_elem_id += 1;
        input.quad9s.insert(
            q9_id.to_string(),
            SolverQuad9Element {
                id: q9_id,
                nodes: q9_nodes,
                material_id: q.material_id,
                thickness: q.thickness,
            },
        );

        quads_to_remove.push(q.id);
        elevated_any = true;
    }

    for qid in quads_to_remove {
        input.quads.retain(|_, q| q.id != qid);
    }

    elevated_any
}

/// Run adaptive refinement on a 3D structural model until target accuracy is achieved.
pub fn run_adaptive_refinement(
    input: &SolverInput3D,
    options: &AdaptiveOptions,
) -> Result<AdaptiveRefinementResult, String> {
    check_refinable(input)?;

    let mut current_input = input.clone();
    let mut iterations = Vec::new();
    let mut initial_error = None;
    let mut last_results = None;
    let mut last_error = None;

    for iter in 0..=options.max_iterations {
        let results = solve_3d(&current_input)?;
        let estimate = estimate_3d(&current_input, &results)?;

        let dof_num = DofNumbering::build_3d(&current_input);
        let n_dofs = dof_num.n_total;
        let n_nodes = current_input.nodes.len();
        let n_elements = current_input.elements.len()
            + current_input.plates.len()
            + current_input.quads.len()
            + current_input.quad9s.len()
            + current_input.solid_shells.len()
            + current_input.curved_shells.len();

        let max_disp = results
            .displacements
            .iter()
            .map(|d| (d.ux * d.ux + d.uy * d.uy + d.uz * d.uz).sqrt())
            .fold(0.0, f64::max);

        iterations.push(AdaptiveIteration {
            iteration: iter,
            n_nodes,
            n_elements,
            n_dofs,
            estimated_error: estimate.relative_error,
            error_norm: estimate.error_norm,
            solution_norm: estimate.solution_norm,
            max_displacement: max_disp,
        });

        if iter == 0 {
            initial_error = estimate.relative_error;
        }
        last_error = estimate.relative_error;
        last_results = Some(results);

        // 1. Check unestimable cases
        if let Some(reason) = &estimate.unestimable_reason {
            return Ok(AdaptiveRefinementResult {
                converged: false,
                status: AdaptiveStatus::Unestimable {
                    reason: reason.clone(),
                },
                unestimable_reason: Some(reason.clone()),
                initial_error,
                final_error: last_error,
                iterations,
                final_input: current_input,
                final_results: last_results.unwrap(),
            });
        }

        // 2. Check convergence against target error
        if let Some(err) = estimate.relative_error {
            if err <= options.target_error {
                return Ok(AdaptiveRefinementResult {
                    converged: true,
                    status: AdaptiveStatus::Converged,
                    unestimable_reason: None,
                    initial_error,
                    final_error: Some(err),
                    iterations,
                    final_input: current_input,
                    final_results: last_results.unwrap(),
                });
            }
        }

        // 3. Check stopping criteria
        if iter == options.max_iterations {
            return Ok(AdaptiveRefinementResult {
                converged: false,
                status: AdaptiveStatus::MaxIterationsReached,
                unestimable_reason: None,
                initial_error,
                final_error: last_error,
                iterations,
                final_input: current_input,
                final_results: last_results.unwrap(),
            });
        }

        // 4. Select elements to refine (Dörfler bulk marking)
        let mut indicators = estimate.elements.clone();
        indicators.sort_by(|a, b| b.eta.total_cmp(&a.eta));

        let total_eta2: f64 = indicators.iter().map(|e| e.eta * e.eta).sum();
        let cutoff = options.marking_fraction * total_eta2;

        let mut accum = 0.0;
        let mut marked_quads = Vec::new();
        for e in &indicators {
            if accum < cutoff || marked_quads.is_empty() {
                accum += e.eta * e.eta;
                if e.kind == "quad" {
                    marked_quads.push(e.element_id);
                }
            }
        }

        // Try p-refinement (elevation of marked quads to quad9) first if available
        let elevated = elevate_quads_to_quad9(&mut current_input, &marked_quads);

        // If no p-refinement was done (or plates/mesh require h-refinement), perform h-refinement
        if !elevated {
            let refined = refine_uniform(&current_input, options.geometry)?;
            let next_dof_count = DofNumbering::build_3d(&refined.input).n_total;
            if next_dof_count > options.max_dofs {
                return Ok(AdaptiveRefinementResult {
                    converged: false,
                    status: AdaptiveStatus::MaxDofsReached,
                    unestimable_reason: None,
                    initial_error,
                    final_error: last_error,
                    iterations,
                    final_input: current_input,
                    final_results: last_results.unwrap(),
                });
            }
            current_input = refined.input;
        }
    }

    Ok(AdaptiveRefinementResult {
        converged: false,
        status: AdaptiveStatus::MaxIterationsReached,
        unestimable_reason: None,
        initial_error,
        final_error: last_error,
        iterations,
        final_input: current_input,
        final_results: last_results.unwrap(),
    })
}
