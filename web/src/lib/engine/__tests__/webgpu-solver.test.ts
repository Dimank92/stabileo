import { describe, it, expect } from 'vitest';
import {
  isWebGpuAvailable,
  solveWebGpuIterative,
  solveWebGpuBatchWorkloads,
} from '../webgpu-solver';

describe('WebGPU Iterative and Batch Solver', () => {
  it('detects WebGPU availability or handles missing navigator gracefully', async () => {
    const available = await isWebGpuAvailable();
    expect(typeof available).toBe('boolean');
  });

  it('performs automatic CPU fallback when WebGPU is unavailable or forced', async () => {
    const diag = [4.0, 5.0, 6.0, 7.0];
    const rhs = [8.0, 10.0, 12.0, 14.0];

    const result = await solveWebGpuIterative(diag, rhs, { forceCpuFallback: true });

    expect(result.executionMode).toBe('cpuFallback');
    expect(result.converged).toBe(true);
    expect(result.u.length).toBe(4);
    // Exact solution for diagonal system is u = [2, 2, 2, 2]
    expect(result.u[0]).toBeCloseTo(2.0, 5);
    expect(result.u[1]).toBeCloseTo(2.0, 5);
    expect(result.u[2]).toBeCloseTo(2.0, 5);
    expect(result.u[3]).toBeCloseTo(2.0, 5);
  });

  it('matches round-off precision on iterative solve', async () => {
    const diag = [10.0, 15.0, 20.0];
    const rhs = [20.0, 45.0, 80.0];

    const res = await solveWebGpuIterative(diag, rhs, { tolerance: 1e-7 });
    expect(res.converged).toBe(true);
    expect(res.finalRelResidual).toBeLessThanOrEqual(1e-6);
    expect(res.u[0]).toBeCloseTo(2.0, 4);
    expect(res.u[1]).toBeCloseTo(3.0, 4);
    expect(res.u[2]).toBeCloseTo(4.0, 4);
  });

  it('executes batched workloads with measured speedup calculation', async () => {
    const diag = [2.0, 4.0, 8.0];
    const rightHandSides = [
      [2.0, 4.0, 8.0],
      [4.0, 8.0, 16.0],
      [6.0, 12.0, 24.0],
    ];

    const batchRes = await solveWebGpuBatchWorkloads(diag, {
      workloadName: 'parameter_sweep_test',
      rightHandSides,
    });

    expect(batchRes.totalSolves).toBe(3);
    expect(batchRes.solutions.length).toBe(3);
    expect(batchRes.solutions[0][0]).toBeCloseTo(1.0, 5);
    expect(batchRes.solutions[1][0]).toBeCloseTo(2.0, 5);
    expect(batchRes.solutions[2][0]).toBeCloseTo(3.0, 5);
    expect(batchRes.speedupVsCpuSequential).toBeGreaterThanOrEqual(1.0);
  });
});
