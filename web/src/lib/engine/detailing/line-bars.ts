/**
 * A saved beam line's bars as the detailing lays them, member by member, exactly as drawn.
 *
 * ── What it replaces ───────────────────────────────────────────────
 *
 * The frame editor draws bars along a line: where each starts and ends, its face, row, count,
 * diameter and hooks, and the laps a person cuts or overlaps. The detailing used to receive only
 * the members' three regions and five cut points (`memberCuts`), lay its own bars from them, and
 * let the splice planner decide again where the bars met: a lap drawn mid-span vanished, a second
 * curtailment group fell back to one, an extra bar between two arbitrary points could not be said.
 *
 * Here each bar of the line becomes one piece per member it crosses, in the member's own sense:
 * - an end inside a member is `cut` there (hooked if the bar is);
 * - an end on a node where the bar stops is `support`: anchored in that support, by the plan the
 *   design verified, whether or not the line continues (no lap takes it);
 * - an end on a node the bar runs through is `continue`: the pieces either side are one bar, and
 *   the lap materialiser fuses them (`lap-materialize.ts`).
 * Laps are where two bars of one face and row overlap. They are measured on the LINE, where the
 * whole overlap is, against the splice length the bar needs (`lineLaps`), and every pair of bars
 * lapping inside a member is declared to the collision check as intended (`DrawnLap`).
 *
 * Pure: no store, no runes.
 */
import type { BeamLineBar, BeamLineReinforcement, BeamStirrupZone } from '../../design-contract/beam-line';
import { lineLaps, type LineLap } from '../design/beam-line';

export { lineLaps };

const EPS = 1e-6;

export type LineBarEnd = 'support' | 'continue' | 'cut';

export interface LineBarPiece {
  /** The line bar's own id: the bar's identity, kept across regenerations while the line stands. */
  lineBarId: string;
  face: 'top' | 'bottom';
  row: number;
  count: number;
  diameterMm: number;
  /** Along the member from its i node, m. */
  from: number;
  to: number;
  start: LineBarEnd;
  end: LineBarEnd;
  hookStart?: boolean;
  hookEnd?: boolean;
  /** The line bars this one laps with inside the member, by id. */
  lapsWith: string[];
}

export interface LineStirrupPiece { from: number; to: number; diameter: number; legs: number; spacing: number }

export type { LineLap };

/** The pieces of a line's bars and stirrups in each of its members, in each member's own sense. */
export function linePieces(
  line: BeamLineReinforcement, forward: readonly boolean[],
): Map<number, { bars: LineBarPiece[]; stirrups: LineStirrupPiece[] }> {
  const X: number[] = [];
  line.lengths.forEach((_, i) => X.push(i === 0 ? 0 : X[i - 1]! + line.lengths[i - 1]!));
  const laps = lineLaps(line);
  const out = new Map<number, { bars: LineBarPiece[]; stirrups: LineStirrupPiece[] }>();
  line.members.forEach((id, k) => {
    const a = X[k]!, L = line.lengths[k]!, b = a + L, fwd = forward[k] !== false;
    const bars: LineBarPiece[] = [];
    for (const bar of line.bars) {
      const s = Math.max(a, bar.from), e = Math.min(b, bar.to);
      if (e - s < EPS) continue;
      const startKind: LineBarEnd = s > a + EPS ? 'cut' : bar.from < a - EPS ? 'continue' : 'support';
      const endKind: LineBarEnd = e < b - EPS ? 'cut' : bar.to > b + EPS ? 'continue' : 'support';
      const lapsWith = laps.filter((l) => (l.a === bar.id || l.b === bar.id) && l.to > a + EPS && l.from < b - EPS).map((l) => (l.a === bar.id ? l.b : l.a));
      const piece: LineBarPiece = {
        lineBarId: bar.id, face: bar.face, row: bar.row, count: bar.count, diameterMm: bar.diameter,
        from: s - a, to: e - a, start: startKind, end: endKind,
        ...(startKind !== 'continue' && bar.hookStart ? { hookStart: true } : {}),
        ...(endKind !== 'continue' && bar.hookEnd ? { hookEnd: true } : {}),
        lapsWith,
      };
      bars.push(fwd ? piece : mirrorPiece(piece, L));
    }
    const stirrups: LineStirrupPiece[] = [];
    for (const z of line.stirrups) {
      const s = Math.max(a, z.from), e = Math.min(b, z.to);
      if (e - s < EPS) continue;
      const p = { from: s - a, to: e - a, diameter: z.diameter, legs: z.legs, spacing: z.spacing };
      stirrups.push(fwd ? p : { ...p, from: L - p.to, to: L - p.from });
    }
    out.set(id, { bars, stirrups: stirrups.sort((p, q) => p.from - q.from) });
  });
  return out;
}

function mirrorPiece(p: LineBarPiece, L: number): LineBarPiece {
  const { hookStart, hookEnd, ...rest } = p;
  return {
    ...rest, from: L - p.to, to: L - p.from, start: p.end, end: p.start,
    ...(hookEnd ? { hookStart: true } : {}), ...(hookStart ? { hookEnd: true } : {}),
  };
}

/** The stirrup zones of a line, as `BeamStirrupZone`s re-exported for callers that only read pieces. */
export type { BeamLineBar, BeamStirrupZone };
