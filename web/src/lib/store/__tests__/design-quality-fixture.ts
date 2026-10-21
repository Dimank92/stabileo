import { modelStore, resultsStore, uiStore } from '..';
import { designRunStore } from '../design-run.svelte';
import { verificationStore } from '../verification.svelte';
import { solveCombinations3D, validateAndSolve3D } from '../../engine/solver-service';

/** Full production path with the example’s own lateral declaration, including final holds. */
export async function frameDesignQuality() {
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
  const verifiedIds: number[] = [];
  for (const id of verificationStore.contexts.keys()) {
    const status = verificationStore.memberVerdict(id).status;
    if (status === 'ok' || status === 'warn') verifiedIds.push(id);
    counts[status] = (counts[status] ?? 0) + 1;
  }
  // Independent of member holds: an omitted open condition must not inflate this metric.
  const physicallyBlocked = new Set(assemblies.flatMap(a => a.conflicts
    .filter(c => c.severity !== 'marginal').flatMap(c => c.elementIds)));
  const output = { verified: summary.verified, exhausted: summary.searchExhausted, provisional: summary.provisionalBiaxial,
    aborted: summary.aborted, notReached: summary.notReached, counts, assemblies: assemblies.length,
    passingWithBlockingConflicts: verifiedIds.filter(id => physicallyBlocked.has(id)),
    conflicts: assemblies.reduce((n, a) => n + a.conflicts.length, 0),
    blockingConflicts: assemblies.reduce((n, a) => n + a.conflicts.filter(c => c.severity !== 'marginal').length, 0),
    open: assemblies.flatMap(a => a.unsupported).reduce((n, u) => { n[u.key] = (n[u.key] ?? 0) + 1; return n; }, {} as Record<string, number>) };
  return { output, verifiedIds };
}
