import { afterEach, describe, expect, it, vi } from 'vitest';
import { Simulation, type Inputs } from '../simulation.svelte';
import type { WorkerOut } from '../springs.worker';

const input: Inputs = {
  network: { lattice: { kind: 'triangular', phi: 2, psi: 0.5 }, eta: 0, keep: 1, seed: 0, spring_law: 'hookean', k: 1, prestress: 0 },
  label: { family: 'ordered', eta: 0, keep: 1, seed: 0, strain: 0.01 },
  theta: 0, cells: 8, phi: 2, psi: 0.5,
};

it.each([1, 2, 3, 4])('clears every result when solve stage %i fails after success, then recovers', (stage) => {
  let calls = 0;
  let fail = 0;
  const solve = vi.fn(() => {
    if (++calls === fail) throw new Error('solve failed');
    return JSON.stringify({ marker: calls });
  });
  const glue = { default: async () => {}, springs_strip: solve, springs_periodic: solve };
  const sim = new Simulation();
  sim.run(glue, input);
  expect(sim.result?.strip).toEqual({ marker: 1 });
  expect(sim.result?.ordered?.eq).toEqual({ marker: 4 });
  const next = structuredClone(input);
  next.label.strain = 0.02;
  calls = 0;
  fail = stage;
  sim.run(glue, next);
  expect(sim.result).toBeNull();
  expect(sim.error).toBe('solve failed');
  calls = 0;
  fail = 0;
  sim.run(glue, next);
  expect(sim.error).toBeNull();
  expect(sim.result?.input.label.strain).toBe(0.02);
  next.label.strain = 0.03;
  expect(sim.result?.input.label.strain).toBe(0.02);
});

class FakeWorker {
  static instances: FakeWorker[] = [];
  onmessage: ((e: { data: WorkerOut }) => void) | null = null;
  onerror: ((e: { message: string }) => void) | null = null;
  terminate = vi.fn();
  postMessage = vi.fn();
  constructor() {
    FakeWorker.instances.push(this);
  }
  send(data: WorkerOut) {
    this.onmessage?.({ data });
  }
}

describe('Simulation worker execution', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    FakeWorker.instances = [];
  });

  it('solves in a worker and updates status to done with origin live', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    expect(sim.status).toBe('running');
    expect(sim.busy).toBe(true);
    expect(FakeWorker.instances).toHaveLength(1);

    const worker = FakeWorker.instances[0];
    const mockResult = { input, strip: { marker: 10 } as any, cell: {} as any, ordered: null };
    worker.send({ type: 'result', id: 2, result: mockResult, ms: 42 });

    expect(sim.status).toBe('done');
    expect(sim.busy).toBe(false);
    expect(sim.origin).toBe('live');
    expect(sim.ms).toBe(42);
    expect(sim.result).toEqual(mockResult);
  });

  it('terminates the active worker on Stop and sets status to stopped', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    expect(sim.status).toBe('running');
    const worker = FakeWorker.instances[0];

    sim.stop();
    expect(worker.terminate).toHaveBeenCalledOnce();
    expect(sim.status).toBe('stopped');
    expect(sim.busy).toBe(false);
  });

  it('terminates old worker and supersedes computation when new input arrives', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    const firstWorker = FakeWorker.instances[0];

    const next = structuredClone(input);
    next.label.strain = 0.05;
    sim.run(next);

    expect(firstWorker.terminate).toHaveBeenCalledOnce();
    expect(FakeWorker.instances).toHaveLength(2);
    expect(sim.status).toBe('running');

    // Stale result from first worker is rejected
    firstWorker.send({ type: 'result', id: 2, result: { input } as any, ms: 10 });
    expect(sim.result).toBeNull();

    // Valid result from second worker is accepted
    const secondWorker = FakeWorker.instances[1];
    const mockResult = { input: next, strip: { marker: 99 } as any, cell: {} as any, ordered: null };
    secondWorker.send({ type: 'result', id: 3, result: mockResult, ms: 25 });

    expect(sim.result).toEqual(mockResult);
    expect(sim.status).toBe('done');
  });

  it('restarts a computation on restart() by terminating and launching fresh worker', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    const firstWorker = FakeWorker.instances[0];

    sim.restart();
    expect(firstWorker.terminate).toHaveBeenCalledOnce();
    expect(FakeWorker.instances).toHaveLength(2);
    expect(sim.status).toBe('running');

    const secondWorker = FakeWorker.instances[1];
    const mockResult = { input, strip: { restarted: true } as any, cell: {} as any, ordered: null };
    secondWorker.send({ type: 'result', id: 3, result: mockResult, ms: 30 });

    expect(sim.result).toEqual(mockResult);
    expect(sim.status).toBe('done');
  });

  it('reuses cached results without creating a new worker unless forced', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    const worker = FakeWorker.instances[0];
    const mockResult = { input, strip: { cached: true } as any, cell: {} as any, ordered: null };
    worker.send({ type: 'result', id: 2, result: mockResult, ms: 50 });

    expect(sim.origin).toBe('live');

    // Run again with same input
    sim.run(input);
    expect(FakeWorker.instances).toHaveLength(1); // No new worker
    expect(sim.origin).toBe('cached');
    expect(sim.result).toEqual(mockResult);

    // Force recompute
    sim.run(input, true);
    expect(FakeWorker.instances).toHaveLength(2); // New worker created
    expect(sim.status).toBe('running');
  });

  it('handles worker error and clears results', () => {
    vi.stubGlobal('Worker', FakeWorker);
    const sim = new Simulation();
    sim.run(input);
    const worker = FakeWorker.instances[0];

    worker.send({ type: 'error', id: 2, error: 'WASM solver error: invalid input' });
    expect(sim.status).toBe('error');
    expect(sim.error).toBe('WASM solver error: invalid input');
    expect(sim.result).toBeNull();
  });
});
