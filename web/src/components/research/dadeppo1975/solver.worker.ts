import { initSolver, solveArcLength } from '../../../lib/engine/wasm-solver';
import { benchmark, frameFor, sampleSteps, type Run } from './case';

self.onmessage = async () => {
  try {
    await initSolver();
    for (const step of sampleSteps) {
      const run = solveArcLength({ ...benchmark.input, maxSteps: step }) as Run;
      self.postMessage({ frame: frameFor(run, step), steps: run.steps });
    }
    self.postMessage({ done: true });
  } catch (error) {
    self.postMessage({ error: String(error) });
  }
};
