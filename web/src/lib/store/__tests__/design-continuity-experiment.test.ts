import { writeFileSync } from 'node:fs';
import { expect, it, vi } from 'vitest';
import { currentFrameDesignQuality, frameDesignQuality } from './design-quality-fixture';
import { detailingStore } from '../detailing.svelte';
import * as feedback from '../../engine/detailing/design-feedback-loop';

// STABILEO_CONTINUITY_EXPERIMENT=8 npx vitest run --project unit src/lib/store/__tests__/design-continuity-experiment.test.ts
it.runIf(!!process.env.STABILEO_CONTINUITY_EXPERIMENT)('repairs paired continuity and preserves physical approvals', async () => {
  const run = feedback.runDesignFeedbackLoop;
  const spy = vi.spyOn(feedback, 'runDesignFeedbackLoop').mockImplementation(i =>
    run({ ...i, physicalSearch: i.physicalSearch ?? { maxDetailingRuns: 8, maxMemberSearches: 64 } }));
  try {
    await frameDesignQuality();
    expect(detailingStore.generate({ physicalSearch: {
      maxDetailingRuns: 8, maxMemberSearches: 64, operators: ['redistributeLayers'],
    } })).not.toBeNull();
  } finally { spy.mockRestore(); }
  const before = currentFrameDesignQuality();
  const start = performance.now();
  expect(detailingStore.generate({ physicalSearch: {
    maxDetailingRuns: Number(process.env.STABILEO_CONTINUITY_EXPERIMENT), maxMemberSearches: 64,
    operators: ['harmonizeTop'],
  } }), detailingStore.lastError ?? '').not.toBeNull();
  const elapsedMs = performance.now() - start;
  const after = currentFrameDesignQuality();
  const search = detailingStore.lastFeedbackLoop!.physicalSearch!;
  const evidence = { before, after, elapsedMs, trials: search.trials, stats: search.stats,
    termination: search.termination, changed: search.changed };
  writeFileSync('/tmp/design-continuity-search.json', JSON.stringify(evidence));
  console.log(JSON.stringify(evidence));
  expect(after.verifiedIds).toEqual(expect.arrayContaining(before.verifiedIds));
  expect(after.output.passingWithBlockingConflicts).toEqual([]);
  expect(after.output.exhausted).toBe(before.output.exhausted);
  expect(after.output.provisional).toBe(before.output.provisional);
  expect(after.output.blockingConflicts).toBeLessThanOrEqual(before.output.blockingConflicts);
  for (const [key, count] of Object.entries(after.output.open)) {
    expect(count, key).toBeLessThanOrEqual(before.output.open[key] ?? 0);
  }
  expect(search.stats.structuralSolves).toBe(0);
  expect(detailingStore.generate()).not.toBeNull();
  const replay = currentFrameDesignQuality();
  expect(replay.verifiedIds).toEqual(after.verifiedIds);
  expect(replay.output).toEqual(after.output);
}, 600_000);
