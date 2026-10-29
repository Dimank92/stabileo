/// <reference lib="webworker" />
import type { Inputs, SolveAllResult } from './simulation.svelte';
import { solveStrip, solvePeriodic, type Glue } from './springs';

export type WorkerIn =
  | { type: 'init' }
  | { type: 'solve'; id: number; input: Inputs };

export type WorkerOut =
  | { type: 'ready' }
  | { type: 'result'; id: number; result: SolveAllResult; ms: number }
  | { type: 'error'; id: number; error: string }
  | { type: 'unavailable'; message: string };

const getBase = (): string => {
  const base = `${import.meta.env.BASE_URL}research-wasm/grossman-2026/`;
  if (typeof self !== 'undefined' && self.location && self.location.origin) {
    try {
      return new URL(base, self.location.origin).href;
    } catch {
      // fallback
    }
  }
  return base;
};

let gluePromise: Promise<Glue> | null = null;

function load(): Promise<Glue> {
  if (!gluePromise) {
    gluePromise = (async () => {
      const base = getBase();
      const g = (await import(/* @vite-ignore */ `${base}springs.js`)) as Glue;
      await g.default({ module_or_path: `${base}springs_bg.wasm` });
      return g;
    })();
  }
  return gluePromise;
}

// Pre-warm WASM inside the worker in the background
load().catch(() => {});

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

self.onmessage = async (e: MessageEvent<WorkerIn>) => {
  const msg = e.data;
  if (!msg) return;
  if (msg.type === 'init') {
    try {
      await load();
      (self as unknown as Worker).postMessage({ type: 'ready' } satisfies WorkerOut);
    } catch (err) {
      (self as unknown as Worker).postMessage({
        type: 'unavailable',
        message: err instanceof Error ? err.message : String(err),
      } satisfies WorkerOut);
    }
    return;
  }
  if (msg.type === 'solve') {
    const { id, input } = msg;
    const t0 = performance.now();
    try {
      const glue = await load();
      const result = solveAll(glue, input);
      const ms = performance.now() - t0;
      (self as unknown as Worker).postMessage({ type: 'result', id, result, ms } satisfies WorkerOut);
    } catch (err) {
      (self as unknown as Worker).postMessage({
        type: 'error',
        id,
        error: err instanceof Error ? err.message : String(err),
      } satisfies WorkerOut);
    }
  }
};
