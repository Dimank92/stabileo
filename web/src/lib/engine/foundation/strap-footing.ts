/**
 * An eccentric footing balanced by a strap beam to another footing.
 *
 * A footing at a property line is set off its column; on its own, its bearing carries the moment
 * of that offset and it is checked as eccentric. Tied by a strap beam (stiff enough to turn the
 * pair as one) to a footing further in, the moment goes into the strap, and the classical reading
 * of the pair holds. With P1 the column on the eccentric footing, e the offset of that footing's
 * centre along the strap and L the distance between the two columns, moments about the inner
 * column give the eccentric footing a centred reaction
 *
 *   R1 = P1 · L / (L − e),
 *
 * and the inner footing is relieved by what the strap lifts off it:
 *
 *   R2 = P2 − P1 · e / (L − e).
 *
 * A strip under a basement wall propped at its head by a floor balances its offset the same way,
 * the wall and the floor being the strap: P·e is a couple between the foot and the floor, and the
 * strip is checked centred across (`Footing.balanced`).
 *
 * The strap carries M = P1 · e at the eccentric column and a shear V = P1 · e / (L − e) along
 * its length. Both relations are linear in the loads, so they hold case by case and combination
 * by combination. The offset across the strap is not taken by it and stays with the footing.
 */
import type { Footing } from '../../model/footing';
import type { FootingNode, NodeReactions } from '../detailing/run-footing-design';

export interface StrapReading {
  footingId: number; partnerId: number;
  /** Between the columns, and the offset along the strap, m. */
  L: number; e: number;
  /** R1 / P1 = L / (L − e). */
  factor: number;
  /** The strap's governing moment and shear, kN·m and kN, and the combination they come from. */
  M: number; V: number; combinationName: string | null;
}

/** A footing whose offset across is taken by the structure above (`Footing.balanced`). */
export interface BalancedReading {
  footingId: number;
  /** The offset across that is no longer the soil's, m, and the footing's length along, m. */
  e: number; length: number;
  /** The couple: moment at the foot per metre, kN·m/m, and the prop's force per metre, kN/m (with a height). */
  m: number; h?: number; combinationName: string | null;
}

export interface StrapAdjusted {
  footings: Footing[];
  reactions: Map<number, NodeReactions>;
  straps: StrapReading[];
  balanced: BalancedReading[];
  /** Straps that cannot be read: the partner missing, or the offset not toward it. */
  refused: Array<{ footingId: number; reason: 'noPartner' | 'offsetNotTowardPartner' }>;
}

export function strapAdjust(footings: readonly Footing[], nodes: ReadonlyMap<number, FootingNode>, reactions: ReadonlyMap<number, NodeReactions>): StrapAdjusted {
  const out: StrapAdjusted = { footings: [...footings], reactions: new Map(reactions), straps: [], balanced: [], refused: [] };
  // The structure's couple: the offset across (local L, the short side of a strip) leaves the soil.
  for (const f of footings) {
    if (f.balanced?.by !== 'structure' || Math.abs(f.eccentricityL) < 1e-6) continue;
    const e = Math.abs(f.eccentricityL);
    out.footings = out.footings.map((x) => (x.id === f.id ? { ...x, eccentricityL: 0 } : x));
    const nodesOn = [f.nodeId, ...(f.supportedNodeIds ?? [])];
    let worst = 0, name: string | null = null;
    const combos = new Map<number, { n: string; p: number }>();
    for (const n of nodesOn) for (const c of reactions.get(n)?.factored ?? []) {
      const k = combos.get(c.combinationId) ?? { n: c.combinationName, p: 0 };
      k.p += Math.abs(c.fz); combos.set(c.combinationId, k);
    }
    for (const k of combos.values()) if (k.p > worst) { worst = k.p; name = k.n; }
    const m = (worst * e) / Math.max(f.B, 1e-6);
    out.balanced.push({ footingId: f.id, e, length: f.B, m, ...(f.balanced.height ? { h: m / f.balanced.height } : {}), combinationName: name });
  }
  for (const f of footings) {
    if (!f.strap) continue;
    const g = footings.find((x) => x.id === f.strap!.toFootingId);
    const a = nodes.get(f.nodeId), b = g ? nodes.get(g.nodeId) : undefined;
    if (!g || !a || !b) { out.refused.push({ footingId: f.id, reason: 'noPartner' }); continue; }
    const L = Math.hypot(b.x - a.x, b.y - a.y);
    const u = { x: (b.x - a.x) / L, y: (b.y - a.y) / L };
    // The footing's centre from its column, in plan, from its offset along its own axes.
    const t = (f.rotationDeg * Math.PI) / 180;
    const dx = f.eccentricityB * Math.cos(t) - f.eccentricityL * Math.sin(t);
    const dy = f.eccentricityB * Math.sin(t) + f.eccentricityL * Math.cos(t);
    const e = dx * u.x + dy * u.y;
    if (!(e > 1e-6) || e >= L) { out.refused.push({ footingId: f.id, reason: 'offsetNotTowardPartner' }); continue; }
    const factor = L / (L - e), relief = e / (L - e);
    // The footing centred along the strap; what lies across it stays.
    const rx = dx - e * u.x, ry = dy - e * u.y;
    out.footings = out.footings.map((x) => (x.id === f.id ? { ...x, eccentricityB: rx * Math.cos(t) + ry * Math.sin(t), eccentricityL: -rx * Math.sin(t) + ry * Math.cos(t) } : x));
    const r1 = out.reactions.get(f.nodeId), r2 = out.reactions.get(g.nodeId);
    if (r1) {
      out.reactions.set(f.nodeId, {
        factored: r1.factored.map((c) => ({ ...c, fz: c.fz * factor })),
        ...(r1.cases ? { cases: r1.cases.map((c) => ({ ...c, fz: c.fz * factor })) } : {}),
      });
      if (r2) {
        out.reactions.set(g.nodeId, {
          factored: r2.factored.map((c) => ({ ...c, fz: c.fz - (r1.factored.find((k) => k.combinationId === c.combinationId)?.fz ?? 0) * relief })),
          ...(r2.cases ? { cases: r2.cases.map((c) => ({ ...c, fz: c.fz - (r1.cases?.find((k) => k.caseId === c.caseId)?.fz ?? 0) * relief })) } : {}),
        });
      }
      const gov = r1.factored.reduce((m, c) => (Math.abs(c.fz) > Math.abs(m.fz) ? c : m), r1.factored[0] ?? { fz: 0, combinationName: null as never } as never);
      const P = Math.abs(gov?.fz ?? 0);
      out.straps.push({ footingId: f.id, partnerId: g.id, L, e, factor, M: P * e, V: P * relief, combinationName: gov?.combinationName ?? null });
    } else {
      out.straps.push({ footingId: f.id, partnerId: g.id, L, e, factor, M: 0, V: 0, combinationName: null });
    }
  }
  return out;
}
