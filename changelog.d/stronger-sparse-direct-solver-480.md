### Performance
#### Stronger sparse direct solver: nested dissection, AMG and iterative refinement (#480)

Adds METIS-quality nested dissection ordering (`nested_dissection_order` and
`CholOrdering::MetisNestedDissection`) to minimize fill-in on 2D and 3D
structural meshes, an Algebraic Multigrid preconditioner (`AmgPreconditioner`)
for large shell models, and Wilkinson iterative refinement
(`sparse_cholesky_solve_with_refinement`) to eliminate precision loss and
dense LU fallback on thin-shell models (#454).
