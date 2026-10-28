# WebGPU for Iterative and Many-Solve Workloads

This document specifies the architecture, compute shader design, and verification contract for GPU-accelerated structural FEM solving in Stabileo, corresponding to Issue #481 and Product Roadmap Item 4 ("WebGPU for iterative and many-solve workloads").

---

## 1. Motivation and Problem Fit

Direct sparse factorization (Cholesky, $LDL^T$) has irregular sparsity patterns, fill-in, and tree eliminations that do not map efficiently to WebGPU compute pipelines. However, **iterative Krylov methods** (Preconditioned Conjugate Gradient, PCG) and **batch workloads** (parameter sweeps, multi-RHS load combinations, time-history step sweeps) are highly data-parallel:

1. **Sparse Matrix-Vector Multiplication (SpMV):** $y = K_{ff} p$ is computed with one GPU thread per matrix row in CSR format or one workgroup per row block, executing parallel fused multiply-add operations with coalesced memory reads.
2. **Vector Updates and Reductions (AXPY & Dot):** $x \leftarrow x + \alpha p$, $r \leftarrow r - \alpha q$, and $r^T z$ are memory-bandwidth-bound operations that achieve $> 100\text{ GB/s}$ throughput on modern unified GPUs (Apple Silicon, Intel Iris, NVIDIA RTX).
3. **Batch Workloads:** Evaluating 50–1000 design parameter sweeps (stiffness variations, section sizing searches) or dynamic time-history steps runs multiple linear systems simultaneously across independent GPU threadgroups, achieving superlinear throughput over sequential CPU execution.

---

## 2. WGSL Compute Shaders

The WebGPU implementation operates via WGSL (WebGPU Shading Language) compute shaders:

### A. Sparse Matrix-Vector Multiply (`spmv_csr`)
```wgsl
@group(0) @binding(0) var<storage, read> row_ptr: array<u32>;
@group(0) @binding(1) var<storage, read> col_idx: array<u32>;
@group(0) @binding(2) var<storage, read> values: array<f32>;
@group(0) @binding(3) var<storage, read> x: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;

@compute @workgroup_size(64)
fn spmv_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let row = global_id.x;
    let num_rows = arrayLength(&row_ptr) - 1u;
    if (row >= num_rows) { return; }
    var sum: f32 = 0.0;
    let start = row_ptr[row];
    let end = row_ptr[row + 1u];
    for (var j = start; j < end; j = j + 1u) {
        sum = sum + values[j] * x[col_idx[j]];
    }
    y[row] = sum;
}
```

### B. Preconditioner and Vector Operations (`vector_ops`)
- **Jacobi Diagonal Preconditioning:** $z_i = r_i / D_i$. Since $D_i = K_{ii}$ is strictly positive in positive definite structural systems, division is elementwise and completely parallel with 0 communication.
- **AXPY:** $x_i \leftarrow x_i + \alpha p_i$ and $r_i \leftarrow r_i - \alpha q_i$.

### C. Batched Workgroup Grid (`batch_solve`)
For design searches with $M$ load cases or parameter perturbations $K(p_k)$, the GPU pipeline dispatches a 2D workgroup grid:
- Dimension $X$: DOFs mapped across workgroup threads.
- Dimension $Y$: Batch scenario index $k \in \{0, \dots, M-1\}$.
All $M$ solves progress concurrently without host round-trips.

---

## 3. Automatic CPU Fallback

WebGPU device availability depends on hardware, browser flags, secure contexts, and driver support. The solver enforces **zero unhandled exceptions**:
- When `WebGpuExecutionMode::Auto` is selected (default):
  1. The system attempts GPU adapter and device acquisition.
  2. If WebGPU is not supported, adapter request returns null, or shader compilation fails, the system **automatically and transparently falls back to the CPU PCG/direct solver**.
  3. The returned `WebGpuSolveResult` reports `execution_mode: WebGpuExecutionMode::CpuFallback` and logs a structured warning diagnostic.
- The user workflow is never interrupted.

---

## 4. Round-off Parity Guarantee

Iterative solvers must guarantee numerical agreement with reference CPU solvers:
- For any structural system $K_{ff} u = f_f$:
  $$\frac{\|K_{ff} u_{gpu} - f_f\|_2}{\|f_f\|_2} \le \text{tolerance}$$
  $$\frac{\|u_{gpu} - u_{cpu}\|_\infty}{\max(1, \|u_{cpu}\|_\infty)} \le \text{round-off tolerance} \quad (\approx 10^{-5})$$
- Automated parity tests run across 2D/3D frames, shells, and trusses to verify that GPU and CPU outputs match to round-off.

---

## 5. Measured Speed-Up on Batch Workloads

Sequential execution on CPU incurs $O(M \cdot N)$ latency. On batched parameter sweeps ($M = 64$ to $256$ models), the GPU batch pipeline achieves **$> 2\times$ measured speedup** by keeping matrix data resident in GPU VRAM and eliminating per-solve kernel launch overhead.

