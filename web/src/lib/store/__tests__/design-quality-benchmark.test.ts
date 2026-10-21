import { expect, it } from 'vitest';
import { modelStore, resultsStore, uiStore } from '..';
import { designRunStore } from '../design-run.svelte';
import { verificationStore } from '../verification.svelte';
import { solveCombinations3D, validateAndSolve3D } from '../../engine/solver-service';

// Opt-in end-to-end benchmark: STABILEO_DESIGN_BENCHMARK=1 npx vitest run --project unit src/lib/store/__tests__/design-quality-benchmark.test.ts
it.runIf(process.env.STABILEO_DESIGN_BENCHMARK === '1')('measures the designed and detailed frame', async () => {
  await modelStore.loadExample('rc-design-frame');
  uiStore.analysisMode = 'pro';
  const combo = solveCombinations3D(modelStore.model as never, modelStore.model.loadCases, modelStore.model.combinations, true, false);
  if (!combo || typeof combo === 'string') throw new Error(String(combo));
  resultsStore.setResults3D(validateAndSolve3D(modelStore.model as never, true, false) as never);
  resultsStore.setCombinationResults3D(combo.perCase, combo.perCombo, combo.envelope);
  designRunStore.designAll({ maxRunMs: 180_000 });
  const assemblies = modelStore.model.detailing?.assemblies ?? [];
  const summary = verificationStore.runSummary!;
  const counts: Record<string, number> = {};
  for (const id of verificationStore.contexts.keys()) {
    const status = verificationStore.memberVerdict(id).status;
    counts[status] = (counts[status] ?? 0) + 1;
  }
  const output = { verified: summary.verified, exhausted: summary.searchExhausted, provisional: summary.provisionalBiaxial,
    aborted: summary.aborted, notReached: summary.notReached, counts, assemblies: assemblies.length,
    conflicts: assemblies.reduce((n, a) => n + a.conflicts.length, 0),
    blockingConflicts: assemblies.reduce((n, a) => n + a.conflicts.filter(c => c.severity !== 'marginal').length, 0),
    open: assemblies.flatMap(a => a.unsupported).reduce((n, u) => { n[u.key] = (n[u.key] ?? 0) + 1; return n; }, {} as Record<string, number>) };
  expect(summary.aborted).toBe(false);
  expect(summary.notReached).toBe(0);
  expect(Object.values(counts).reduce((a, b) => a + b, 0)).toBe(408);
  expect(assemblies.length).toBeGreaterThan(0);
  console.log(JSON.stringify(output));
}, 600_000);
