import { readFileSync, writeFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { frameDesignQuality, currentFrameDesignQuality } from './design-quality-fixture';
import { modelStore } from '..';
import { verificationStore } from '../verification.svelte';
import { detailingStore } from '../detailing.svelte';
import { detailableReinforcement, type MemberDesignOutcome } from '../../engine/design/outcome';

// Forensic continuation of a saved audit. Forces are solved again; only its reinforcement
// assignment is restored. Every comparison below uses a newly generated full detail.
it.runIf(!!process.env.STABILEO_REPAIR_SNAPSHOT)('reverifies a retained reinforcement assignment', async () => {
  const saved = JSON.parse(readFileSync(process.env.STABILEO_REPAIR_SNAPSHOT!, 'utf8'));
  await frameDesignQuality();
  const outcomes = new Map<number, MemberDesignOutcome>(saved.members.map((m: { id: number; outcome: MemberDesignOutcome }) => [m.id, m.outcome]));
  modelStore.reinforcementTransaction(api => {
    for (const [id, o] of outcomes) {
      const rebar = detailableReinforcement(o)?.rebar;
      // Refused members have no retained steel. Clear any seed supplied by the fresh model.
      api.setReinforcement(id, rebar);
    }
  });
  verificationStore.setDesignOutcomes({ ...verificationStore.runSummary!, outcomes });
  expect(detailingStore.generate(), detailingStore.lastError ?? '').not.toBeNull();
  const restored = currentFrameDesignQuality();
  if (process.env.STABILEO_REPAIR_CATALOG) expect(detailingStore.generate({ physicalSearch: {
    maxDetailingRuns: Number(process.env.STABILEO_REPAIR_CATALOG), maxMemberSearches: 512,
    operators: ['harmonizeTopCatalog'],
  } }), detailingStore.lastError ?? '').not.toBeNull();
  const after = currentFrameDesignQuality();
  const detail = detailingStore.lastRun!;
  const search = detailingStore.lastFeedbackLoop?.physicalSearch;
  const evidence = { before: saved.after, restored, after,
    search: search ? { trials: search.trials, stats: search.stats, termination: search.termination, changed: search.changed } : undefined,
    reverification: detail.reverification,
    assemblies: detail.assemblies, lapping: detail.lapping, layering: detail.layering,
    members: [...verificationStore.contexts].map(([id, ctx]) => ({ id, kind: ctx.elementType,
      section: ctx.section, material: ctx.material, verdict: verificationStore.memberVerdict(id),
      outcome: verificationStore.outcomeFor(id) })),
  };
  writeFileSync('/tmp/design-next-conflicts.json', JSON.stringify(evidence,
    (_k, v) => v instanceof Map ? Object.fromEntries(v) : v instanceof Set ? [...v] : v));
  console.log(JSON.stringify({ before: saved.after.output, after: after.output,
    lost: saved.after.verifiedIds.filter((id: number) => !after.verifiedIds.includes(id)) }));
  expect(after.verifiedIds).toEqual(expect.arrayContaining(saved.after.verifiedIds));
  expect(after.output.passingWithBlockingConflicts).toEqual([]);
  expect(after.output.blockingConflicts).toBeLessThanOrEqual(saved.after.output.blockingConflicts);
  expect(detailingStore.generate()).not.toBeNull();
  expect(currentFrameDesignQuality()).toEqual(after);
}, 600_000);
