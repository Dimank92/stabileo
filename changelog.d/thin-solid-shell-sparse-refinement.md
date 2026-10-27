### Performance
#### Iterative refinement keeps sparse Cholesky on ill-conditioned thin solid shells (2026-10-28)

Thin solid-shell meshes (such as 32x32 with t/a = 0.001) have large stiffness matrix condition numbers (~10^11), which previously caused the direct sparse Cholesky residual check to exceed 1e-6 due to double-precision floating-point roundoff, triggering an unnecessary fallback to dense LU factorization (~110 s per solve). Iterative refinement using the existing Cholesky factor reduces residual error to machine precision in 3 iterations (~30 ms), keeping the solve time under 3.5 s while matching analytical Kirchhoff deflections within 0.04% without dense LU fallback.
