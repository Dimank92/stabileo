/** Small, atomic alternatives for the top steel of two adjacent beam members. */
import type { ProvidedReinforcement, RebarLayer } from '../../design-contract/reinforcement';
import type { MemberContext } from '../design/member-context';
import { STANDARD_LONG_DIAS } from '../design/objective';
import { BEAM_LIMITS, maxBarsPerRow } from '../design/candidate-enumerate-beam';

export interface BeamContinuityAlternative {
  layers: RebarLayer[];
  reinforcement: [ProvidedReinforcement, ProvidedReinforcement];
}

/**
 * Reuse each existing top pattern on BOTH beams and at BOTH ends. The generator may then
 * carry equal bars through their shared support. This is a proposal, not a continuity proof:
 * different sections, seats, demand and neighbours still require full detailing.
 * Explicit curtailment/groups and multi-element spans need a different operator.
 */
export function* beamContinuityAlternatives(
  contexts: readonly [MemberContext, MemberContext],
  current: readonly [ProvidedReinforcement, ProvidedReinforcement],
): Generator<BeamContinuityAlternative> {
  if (contexts.some(c => c.elementType !== 'beam' || c.span)) return;
  if (current.some(r => !r.regions || r.regions.topStartGroups || r.regions.topEndGroups
    || r.regions.continuity)) return;
  const seen = new Set<string>();
  for (const r of current) for (const face of ['topStart', 'topEnd'] as const) {
    const group = r.regions![face];
    const layers = r.regions![`${face}Layers`] ?? (group ? [{ ...group, row: 0 }] : []);
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
    const copy = (r: ProvidedReinforcement): ProvidedReinforcement => ({ ...r, regions: {
      ...r.regions, topStart: { count, diameter: dia }, topEnd: { count, diameter: dia },
      topStartLayers: ordered.map(l => ({ ...l })), topEndLayers: ordered.map(l => ({ ...l })),
    } });
    yield { layers: ordered.map(l => ({ ...l })), reinforcement: [copy(current[0]), copy(current[1])] };
  }
}
