use dedaliano_engine::linalg::amg::AmgPreconditioner;
use dedaliano_engine::linalg::nested_dissection::nested_dissection_order;
use dedaliano_engine::linalg::pcg::pcg_solve;
use dedaliano_engine::linalg::sparse::CscMatrix;
use dedaliano_engine::linalg::sparse_chol::{
    numeric_cholesky, sparse_cholesky_solve, sparse_cholesky_solve_with_refinement,
    symbolic_cholesky_with, CholOrdering,
};

/// Builds a 2D grid Laplacian (N = k x k) as SPD CSC matrix.
fn build_grid_2d(k: usize) -> CscMatrix {
    let n = k * k;
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut vals = Vec::new();

    for y in 0..k {
        for x in 0..k {
            let i = y * k + x;
            // Diagonal entry
            rows.push(i);
            cols.push(i);
            vals.push(4.0);

            // X neighbor (i + 1, i)
            if x + 1 < k {
                rows.push(i + 1);
                cols.push(i);
                vals.push(-1.0);
            }
            // Y neighbor (i + k, i)
            if y + 1 < k {
                rows.push(i + k);
                cols.push(i);
                vals.push(-1.0);
            }
        }
    }

    CscMatrix::from_triplets(n, &rows, &cols, &vals)
}

#[test]
fn test_nested_dissection_ordering_produces_valid_cholesky() {
    let mat = build_grid_2d(6); // 36 DOFs
    let n = mat.n;

    let perm = nested_dissection_order(&mat);
    assert_eq!(perm.len(), n);

    let mut seen = vec![false; n];
    for &p in &perm {
        assert!(p < n);
        assert!(!seen[p], "Permutation must not contain duplicates");
        seen[p] = true;
    }

    let sym = symbolic_cholesky_with(&mat, CholOrdering::MetisNestedDissection);
    let num = numeric_cholesky(&std::rc::Rc::new(sym), &mat).expect("Numeric factorization must succeed");

    let b = vec![1.0; n];
    let x = sparse_cholesky_solve(&num, &b);

    // Verify residual
    let ax = mat.sym_mat_vec(&x);
    for i in 0..n {
        assert!((ax[i] - b[i]).abs() < 1e-10, "Residual at {i} must be < 1e-10");
    }
}

#[test]
fn test_amg_preconditioner_solves_system() {
    let mat = build_grid_2d(5); // 25 DOFs
    let n = mat.n;

    let amg = AmgPreconditioner::new(&mat).expect("AMG preconditioner construction must succeed");
    let b = vec![2.0; n];

    let res = pcg_solve(&mat, &b, &amg, 1e-8, 500);
    assert!(res.converged, "PCG with AMG preconditioner must converge");
    assert!(res.final_rel_residual <= 1e-8);
}

#[test]
fn test_thin_shell_iterative_refinement() {
    // Construct ill-conditioned system mimicking thin-shell element (t/a = 0.001)
    let n = 16;
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut vals = Vec::new();

    for i in 0..n {
        rows.push(i);
        cols.push(i);
        // Stiff membrane DOFs vs soft bending DOFs
        let diag_val = if i % 2 == 0 { 1e6 } else { 1.0 };
        vals.push(diag_val);

        if i + 1 < n {
            rows.push(i + 1);
            cols.push(i);
            vals.push(0.5);
        }
    }

    let mat = CscMatrix::from_triplets(n, &rows, &cols, &vals);
    let sym = symbolic_cholesky_with(&mat, CholOrdering::Amd);
    let num = numeric_cholesky(&std::rc::Rc::new(sym), &mat).expect("Numeric factorization");

    let b = vec![100.0; n];
    // Solve with refinement to achieve high accuracy without falling back to dense LU
    let x = sparse_cholesky_solve_with_refinement(&num, &mat, &b, 2);

    let ax = mat.sym_mat_vec(&x);
    let mut res_norm_sq = 0.0;
    let mut b_norm_sq = 0.0;
    for i in 0..n {
        let diff = b[i] - ax[i];
        res_norm_sq += diff * diff;
        b_norm_sq += b[i] * b[i];
    }
    let rel_res = (res_norm_sq / b_norm_sq).sqrt();
    assert!(rel_res < 1e-11, "Refined solution must have residual < 1e-11, got {rel_res}");
}
