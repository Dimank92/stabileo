use dedaliano_engine::linalg::sparse::CscMatrix;
use dedaliano_engine::solver::webgpu::{
    solve_batch_workloads, solve_iterative_webgpu, verify_gpu_cpu_roundoff_parity,
    BatchWorkloadConfig, WebGpuExecutionMode, WebGpuSolverOptions,
};

/// Helper to build an SPD 1D beam/bar stiffness matrix (tridiagonal) of dimension n.
fn build_spd_test_matrix(n: usize) -> CscMatrix {
    let mut col_ptr = Vec::with_capacity(n + 1);
    let mut row_idx = Vec::new();
    let mut values = Vec::new();

    // Storing lower triangle: for column j, entries (i, j) with i >= j
    col_ptr.push(0);
    for j in 0..n {
        // Diagonal entry (j, j)
        row_idx.push(j);
        values.push(4.0);

        // Sub-diagonal entry (j + 1, j) if within bounds
        if j + 1 < n {
            row_idx.push(j + 1);
            values.push(-1.0);
        }
        col_ptr.push(row_idx.len());
    }

    CscMatrix {
        n,
        col_ptr,
        row_idx,
        values,
    }
}

#[test]
fn test_webgpu_roundoff_parity() {
    let n = 40;
    let mat = build_spd_test_matrix(n);
    let mut rhs = vec![0.0; n];
    for i in 0..n {
        rhs[i] = 10.0 + (i as f64 * 0.5).sin();
    }

    let parity = verify_gpu_cpu_roundoff_parity(&mat, &rhs, 1e-6);
    assert!(parity.is_ok(), "Roundoff parity check failed: {:?}", parity.err());
}

#[test]
fn test_webgpu_automatic_cpu_fallback() {
    let n = 25;
    let mat = build_spd_test_matrix(n);
    let rhs = vec![5.0; n];

    let mut opts = WebGpuSolverOptions::default();
    opts.force_cpu_fallback = true;
    opts.tolerance = 1e-7;

    let res = solve_iterative_webgpu(&mat, &rhs, &opts);
    assert_eq!(res.execution_mode, WebGpuExecutionMode::CpuFallback);
    assert!(res.converged, "CPU fallback solve should converge");
    assert!(res.final_rel_residual <= 1e-6);
    assert_eq!(res.u.len(), n);
}

#[test]
fn test_webgpu_batch_workloads_speedup() {
    let n = 30;
    let mat = build_spd_test_matrix(n);

    let num_cases = 16;
    let mut rhs_batch = Vec::with_capacity(num_cases);
    for k in 0..num_cases {
        let mut rhs = vec![0.0; n];
        for i in 0..n {
            rhs[i] = (k as f64 + 1.0) * (1.0 + (i as f64).cos());
        }
        rhs_batch.push(rhs);
    }

    let mut stiffness_multipliers = Vec::with_capacity(num_cases);
    for k in 0..num_cases {
        stiffness_multipliers.push(0.8 + 0.05 * (k as f64));
    }

    let config = BatchWorkloadConfig {
        workload_name: "design_search_parameter_sweep".into(),
        right_hand_sides: rhs_batch,
        stiffness_multipliers,
    };

    let opts = WebGpuSolverOptions::default();
    let batch_res = solve_batch_workloads(&mat, &config, &opts);

    assert_eq!(batch_res.total_solves, num_cases);
    assert_eq!(batch_res.solutions.len(), num_cases);
    assert!(batch_res.speedup_vs_cpu_sequential >= 1.0, "Batch solve should exhibit measured speedup");
}

#[test]
fn test_webgpu_zero_rhs() {
    let n = 20;
    let mat = build_spd_test_matrix(n);
    let rhs = vec![0.0; n];

    let opts = WebGpuSolverOptions::default();
    let res = solve_iterative_webgpu(&mat, &rhs, &opts);

    assert_eq!(res.iterations, 0);
    assert!(res.converged);
    assert_eq!(res.final_rel_residual, 0.0);
    for val in res.u {
        assert_eq!(val, 0.0);
    }
}

