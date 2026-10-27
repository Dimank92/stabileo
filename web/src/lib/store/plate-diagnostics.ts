/**
 * What the diagnostics list says about the physical plates (`store/physical-plates.ts`): a plate
 * with no mesh or an out-of-date one, an outline that no longer makes a plate, and meshes with
 * poor elements. Each names its plate, so a click selects it whole where its mesh is made.
 */
import { physicalPlates, plateStatus } from './physical-plates';
import { meshBeforeAnalysis } from './mesh-policy.svelte';
import type { SolverDiagnostic } from '../engine/types';

export function plateDiagnostics(t: (key: string) => string): SolverDiagnostic[] {
  const out: SolverDiagnostic[] = [];
  for (const p of physicalPlates()) {
    const s = plateStatus(p.id);
    const base = { source: 'model' as const, objects: [{ kind: 'plate' as const, id: p.id }], details: { plate: p.name } };
    // Without a mesh the analysis meshes it, or stops when told not to: a warning only then.
    if (s === 'unmeshed') out.push({ ...base, severity: meshBeforeAnalysis.value ? 'info' : 'warning', code: 'PLATE_UNMESHED', message: t('plates.diag.unmeshed') });
    else if (s === 'stale') out.push({ ...base, severity: meshBeforeAnalysis.value ? 'info' : 'warning', code: 'PLATE_MESH_STALE', message: t('plates.diag.stale') });
    else if (s === 'invalid') out.push({ ...base, severity: 'error', code: 'PLATE_INVALID', message: t('plates.diag.invalid') });
    const q = p.surface.meshed?.quality;
    if (s === 'current' && q && q.poor > 0) out.push({ ...base, severity: 'warning', code: 'PLATE_MESH_POOR', message: t('plates.diag.poor'), details: { plate: p.name, poor: q.poor, minAngle: Math.round(q.minAngle) } });
  }
  return out;
}
