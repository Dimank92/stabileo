/** Atomic longitudinal patterns for an adjacent beam pair or line. */
import type { ProvidedReinforcement, RebarLayer } from '../../design-contract/reinforcement';
import type { MemberContext } from '../design/member-context';
import { STANDARD_LONG_DIAS } from '../design/objective';
import { BEAM_LIMITS, maxBarsPerRow } from '../design/candidate-enumerate-beam';

export interface BeamContinuityAlternative {
  layers: RebarLayer[];
  reinforcement: ProvidedReinforcement[];
}

/**
 * Reuse each existing pattern throughout the line. The generator may then
 * carry equal bars through their shared support. This is a proposal, not a continuity proof:
 * different sections, seats, demand and neighbours still require full detailing.
 * Explicit curtailment/groups and multi-element spans need a different operator.
 */
export function* beamContinuityAlternatives(
  contexts: readonly MemberContext[],
  current: readonly ProvidedReinforcement[],
  face: 'top' | 'bottom' = 'top',
): Generator<BeamContinuityAlternative> {
  if (contexts.length < 2 || contexts.length !== current.length) return;
  if (contexts.some(c => c.elementType !== 'beam' || c.span)) return;
  if (current.some(r => !r.regions || r.regions.continuity || (face === 'top'
    ? r.regions.topStartGroups || r.regions.topEndGroups
    : r.regions.bottomGroups && r.regions.bottomGroups.length !== 1))) return;
  const seen = new Set<string>();
  const regions = face === 'top' ? ['topStart', 'topEnd'] as const : ['bottomSpan'] as const;
  for (const r of current) for (const region of regions) {
    const group = r.regions![region];
    const layers = (face === 'bottom' ? r.regions!.bottomGroups?.[0].layers : undefined)
      ?? r.regions![`${region}Layers`] ?? (group ? [{ ...group, row: 0 }] : []);
    if (!layers.length || layers.length > BEAM_LIMITS.maxRows) continue;
    const ordered = [...layers].sort((a, b) => a.row - b.row);
    const dia = ordered[0].diameter;
    if (!Number.isFinite(dia) || dia <= 0 || ordered.some((l, i) => l.row !== i
      || l.diameter !== dia || !Number.isSafeInteger(l.count) || l.count < BEAM_LIMITS.minBarsPerRow)) continue;
    if (contexts.some((ctx, i) => {
      const stirrup = Math.max(ctx.material.stirrupDia,
        current[i].regions!.stirrupsSupport?.diameter ?? 0,
        current[i].regions!.stirrupsSpan?.diameter ?? 0, current[i].stirrups?.diameter ?? 0);
      return !(ctx.material.longDiametersMm ?? STANDARD_LONG_DIAS).includes(dia)
        || ordered.some(l => l.count > maxBarsPerRow(ctx.axes.bFlex, ctx.material.cover, stirrup, dia));
    })) continue;
    const key = JSON.stringify(ordered);
    if (seen.has(key)) continue;
    seen.add(key);
    const count = ordered.reduce((sum, l) => sum + l.count, 0);
    const copy = (r: ProvidedReinforcement): ProvidedReinforcement => ({ ...r, regions: face === 'top' ? {
      ...r.regions, topStart: { count, diameter: dia }, topEnd: { count, diameter: dia },
      topStartLayers: ordered.map(l => ({ ...l })), topEndLayers: ordered.map(l => ({ ...l })),
    } : {
      ...r.regions, bottomSpan: { count, diameter: dia }, bottomSpanLayers: ordered.map(l => ({ ...l })),
      preserveBottomRows: true,
      ...(r.regions!.bottomGroups ? { bottomGroups: [{ ...r.regions!.bottomGroups[0],
        layers: ordered.map(l => ({ ...l })) }] } : {}),
    } });
    yield { layers: ordered.map(l => ({ ...l })), reinforcement: current.map(copy) };
  }
}
