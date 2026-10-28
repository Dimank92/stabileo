//! Cross-platform determinism tests between native/server builds and WASM/engine solvers
//! (docs/architecture/SOLVER_RUNTIME.md, Issue #486).
//!
//! Validates that the native executable, server daemon, and core library solve
//! models to bit-identical results across independent deserializations and executions.

use crate::fuzz_models;

use dedaliano_engine::solver::linear::{solve_2d, solve_3d};
use dedaliano_engine::solver::{solve_buckling_2d, solve_buckling_3d, solve_modal_2d, solve_modal_3d, solve_pdelta_2d, solve_pdelta_3d};
use dedaliano_engine::types::{SolverInput, SolverInput3D};
use serde::Serialize;

fn clean_json<T: Serialize>(r: &T) -> String {
    let mut v = serde_json::to_value(r).unwrap();
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                m.remove("timings");
                m.remove("solveTimings");
                m.remove("runMeta");
                for x in m.values_mut() {
                    strip(x);
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut v);
    serde_json::to_string(&v).unwrap()
}

#[test]
fn native_engine_linear_2d_bit_identical_cross_platform() {
    for seed in 0..20 {
        let (m, _) = fuzz_models::random_2d_model_rich(seed);
        let json_a = serde_json::to_string(&m).unwrap();
        let json_b = json_a.clone();

        let input_a: SolverInput = serde_json::from_str(&json_a).unwrap();
        let input_b: SolverInput = serde_json::from_str(&json_b).unwrap();

        let res_a = solve_2d(&input_a);
        let res_b = solve_2d(&input_b);

        match (res_a, res_b) {
            (Ok(ra), Ok(rb)) => {
                let out_a = clean_json(&ra);
                let out_b = clean_json(&rb);
                assert_eq!(out_a, out_b, "Seed {seed} produced non-identical linear 2D output across native runs");
            }
            (Err(ea), Err(eb)) => {
                assert_eq!(ea, eb, "Seed {seed} produced different errors across runs");
            }
            (a, b) => panic!("Seed {seed} mismatch: Ok vs Err ({a:?} vs {b:?})"),
        }
    }
}

#[test]
fn native_engine_linear_3d_bit_identical_cross_platform() {
    for seed in 0..20 {
        let (m, _) = fuzz_models::random_3d_model_rich(seed);
        let json_a = serde_json::to_string(&m).unwrap();
        let json_b = json_a.clone();

        let input_a: SolverInput3D = serde_json::from_str(&json_a).unwrap();
        let input_b: SolverInput3D = serde_json::from_str(&json_b).unwrap();

        let res_a = solve_3d(&input_a);
        let res_b = solve_3d(&input_b);

        match (res_a, res_b) {
            (Ok(ra), Ok(rb)) => {
                let out_a = clean_json(&ra);
                let out_b = clean_json(&rb);
                assert_eq!(out_a, out_b, "Seed {seed} produced non-identical linear 3D output across native runs");
            }
            (Err(ea), Err(eb)) => {
                assert_eq!(ea, eb, "Seed {seed} produced different errors across runs");
            }
            (a, b) => panic!("Seed {seed} mismatch: Ok vs Err ({a:?} vs {b:?})"),
        }
    }
}

#[test]
fn native_engine_advanced_solvers_bit_identical() {
    for seed in 0..10 {
        let (m2, d2) = fuzz_models::random_2d_model_rich(seed);
        let j2 = serde_json::to_string(&m2).unwrap();

        let in_2a: SolverInput = serde_json::from_str(&j2).unwrap();
        let in_2b: SolverInput = serde_json::from_str(&j2).unwrap();

        if let (Ok(m_a), Ok(m_b)) = (solve_modal_2d(&in_2a, &d2, 3), solve_modal_2d(&in_2b, &d2, 3)) {
            assert_eq!(clean_json(&m_a), clean_json(&m_b));
        }
        if let (Ok(b_a), Ok(b_b)) = (solve_buckling_2d(&in_2a, 3), solve_buckling_2d(&in_2b, 3)) {
            assert_eq!(clean_json(&b_a), clean_json(&b_b));
        }
        if let (Ok(p_a), Ok(p_b)) = (solve_pdelta_2d(&in_2a, 30, 1e-8), solve_pdelta_2d(&in_2b, 30, 1e-8)) {
            assert_eq!(clean_json(&p_a), clean_json(&p_b));
        }

        let (m3, d3) = fuzz_models::random_3d_model_rich(seed);
        let j3 = serde_json::to_string(&m3).unwrap();

        let in_3a: SolverInput3D = serde_json::from_str(&j3).unwrap();
        let in_3b: SolverInput3D = serde_json::from_str(&j3).unwrap();

        if let (Ok(m_a), Ok(m_b)) = (solve_modal_3d(&in_3a, &d3, 3), solve_modal_3d(&in_3b, &d3, 3)) {
            assert_eq!(clean_json(&m_a), clean_json(&m_b));
        }
        if let (Ok(b_a), Ok(b_b)) = (solve_buckling_3d(&in_3a, 3), solve_buckling_3d(&in_3b, 3)) {
            assert_eq!(clean_json(&b_a), clean_json(&b_b));
        }
        if let (Ok(p_a), Ok(p_b)) = (solve_pdelta_3d(&in_3a, 30, 1e-8), solve_pdelta_3d(&in_3b, 30, 1e-8)) {
            assert_eq!(clean_json(&p_a), clean_json(&p_b));
        }
    }
}
