### Performance
#### WebGPU compute shaders for iterative PCG and batched workloads (#481)

Provides GPU-accelerated preconditioned iterative solves (PCG with Jacobi
diagonal preconditioning) and batched parameter sweeps and time-history step
sweeps via WGSL compute shaders. Results are guaranteed to match the CPU
reference path to round-off precision, and an automatic CPU fallback ensures
zero interruptions when WebGPU is unavailable or disabled.
