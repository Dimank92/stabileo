import { solvePeriodic, solveStrip, type Glue, type NetworkSpec } from './springs';
import type { StripLabel } from './open-in-stabileo';
import type { WorkerIn, WorkerOut } from './springs.worker';

export type Inputs = {
  network: NetworkSpec;
  label: StripLabel;
  theta: number;
  cells: number;
  phi: number;
  psi: number;
};

export type SolveAllResult = {
  input: Inputs;
  strip: ReturnType<typeof solveStrip>;
  cell: ReturnType<typeof solvePeriodic>;
  ordered: { hook: ReturnType<typeof solvePeriodic>; eq: ReturnType<typeof solvePeriodic> } | null;
};

export type SolveStatus = 'idle' | 'running' | 'done' | 'stopped' | 'error';
export type Source = 'replay' | 'live' | 'cached';

export function solveAll(glue: Glue, input: Inputs): SolveAllResult {
  const { network, label, theta, cells } = input;
  const strip = solveStrip(glue, { network, width: 13, height: 52, theta, clamp_band: 0.5, strain: label.strain });
  const cell = solvePeriodic(glue, { network, n1: cells, n2: cells, strain: [0, 0, 0] });
  const ordered =
    label.family === 'ordered' && network.keep === 1 && network.prestress === 0
      ? {
          hook: solvePeriodic(glue, { network: { ...network, spring_law: 'hookean' }, n1: 4, n2: 4, strain: [0, 0, 0] }),
          eq: solvePeriodic(glue, { network: { ...network, spring_law: 'equal_weight' }, n1: 4, n2: 4, strain: [0, 0, 0] }),
        }
      : null;
  return { input, strip, cell, ordered };
}

export class Simulation {
  result = $state.raw<SolveAllResult | null>(null);
  status = $state<SolveStatus>('idle');
  origin = $state<Source>('replay');
  error = $state<string | null>(null);
  ms = $state(0);
  busy = $derived(this.status === 'running');

  private worker: Worker | null = null;
  private nextId = 1;
  private activeId = 0;
  private lastInput: Inputs | null = null;
  private cache = new Map<string, { result: SolveAllResult; ms: number }>();

  setReplay(result: SolveAllResult) {
    this.stop();
    this.result = result;
    this.origin = 'replay';
    this.status = 'done';
    this.error = null;
    this.ms = 0;
  }

  run(first: Glue | Inputs, second?: Inputs | boolean, third = false) {
    if (first && typeof (first as Glue).springs_strip === 'function') {
      const glue = first as Glue;
      const input = second as Inputs;
      const start = performance.now();
      try {
        this.result = solveAll(glue, structuredClone(input));
        this.error = null;
        this.status = 'done';
        this.origin = 'live';
      } catch (e) {
        this.result = null;
        this.error = e instanceof Error ? e.message : String(e);
        this.status = 'error';
      }
      this.ms = performance.now() - start;
      return;
    }

    const input = first as Inputs;
    const force = typeof second === 'boolean' ? second : third;
    this.lastInput = structuredClone(input);
    const key = JSON.stringify(input);

    if (!force) {
      const cached = this.cache.get(key);
      if (cached) {
        this.stop();
        this.result = cached.result;
        this.ms = cached.ms;
        this.status = 'done';
        this.origin = 'cached';
        this.error = null;
        return;
      }
    }

    this.stopWorker();
    this.status = 'running';
    this.error = null;
    const runId = ++this.nextId;
    this.activeId = runId;

    try {
      const w = this.createWorker();
      w.postMessage({ type: 'solve', id: runId, input: structuredClone(input) } satisfies WorkerIn);
    } catch (e) {
      this.status = 'error';
      this.error = e instanceof Error ? e.message : String(e);
      this.activeId = 0;
    }
  }

  stop() {
    this.stopWorker();
    this.status = 'stopped';
    this.activeId = 0;
  }

  prewarm() {
    if (typeof Worker === 'undefined') return;
    if (!this.worker) {
      const w = this.createWorker();
      w.postMessage({ type: 'init' } satisfies WorkerIn);
    }
  }

  restart() {
    this.stop();
    if (this.lastInput) {
      this.run(this.lastInput, true);
    }
  }

  dispose() {
    this.stopWorker();
    this.activeId = 0;
  }

  private stopWorker() {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
  }

  private createWorker(): Worker {
    const w = new Worker(new URL('./springs.worker.ts', import.meta.url), { type: 'module' });
    this.worker = w;
    w.onmessage = (e: MessageEvent<WorkerOut>) => {
      if (this.worker !== w) return;
      this.handleMessage(e.data);
    };
    w.onerror = (e) => {
      if (this.worker !== w) return;
      this.handleError(e.message || 'Worker execution error');
    };
    return w;
  }

  private handleMessage(msg: WorkerOut) {
    if (msg.type === 'ready') return;
    if (msg.id !== this.activeId) return;

    if (msg.type === 'result') {
      this.result = msg.result;
      this.ms = msg.ms;
      this.status = 'done';
      this.origin = 'live';
      this.error = null;
      const key = JSON.stringify(msg.result.input);
      this.cache.set(key, { result: msg.result, ms: msg.ms });
      while (this.cache.size > 20) this.cache.delete(this.cache.keys().next().value!);
    } else if (msg.type === 'error') {
      this.result = null;
      this.error = msg.error;
      this.status = 'error';
    }
  }

  private handleError(err: string) {
    this.stopWorker();
    this.status = 'error';
    this.error = err;
    this.result = null;
  }
}
