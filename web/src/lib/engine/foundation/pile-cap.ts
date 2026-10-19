/**
 * A pile cap under a column, checked on its piles' reactions.
 *
 *   Piles      R_i = N/n + My·x_i/Σx² + Mx·y_i/Σy² (rigid cap), service with the cap's weight
 *              against the pile's capacity, none in tension.
 *   Flexure    at each column face, the factored reactions of the piles beyond it times their
 *              lever arm; As = Mu/(φ·0,9·d·fy) on the cap's width, never under 0,0018·b·h.
 *   Shear      one-way at d from each face: a pile whose centre is d_p/2 or more past the section
 *              counts whole, d_p/2 or more before it not at all, linearly between (§13.4.2.5).
 *   Punching   the column, less the piles inside its critical perimeter; and the most loaded
 *              corner pile on its own perimeter, cut by the cap's two edges.
 *
 * Local axes as `rigid-footing.ts`, piles and column centre about the cap's centroid.
 */
import { checkPunchingShear, type PunchingCheck } from '../detailing/punching-shear';

export interface PileCapInput {
  B: number; L: number; thickness: number; cover: number; fc: number; fy: number;
  piles: ReadonlyArray<{ x: number; y: number }>;
  pileDiameter: number; pileCapacity: number;
  column: { x: number; y: number; cx: number; cy: number };
  service: { N: number; Mx: number; My: number };
  factored: { N: number; Mx: number; My: number };
}

export interface PileCapResult {
  reactions: { service: number[]; factored: number[] };
  piles: { max: number; min: number; capacity: number; utilization: number; tension: boolean };
  x: { mu: number; as: number; shear: { vu: number; phiVc: number; utilization: number } };
  y: { mu: number; as: number; shear: { vu: number; phiVc: number; utilization: number } };
  punching: { column: number; cornerPile: number; checks: PunchingCheck[] };
  utilization: number;
}

export function pileReactions(piles: ReadonlyArray<{ x: number; y: number }>, N: number, My: number, Mx: number): number[] {
  const n = piles.length;
  const cx = piles.reduce((s, p) => s + p.x, 0) / n, cy = piles.reduce((s, p) => s + p.y, 0) / n;
  const sx = piles.reduce((s, p) => s + (p.x - cx) ** 2, 0), sy = piles.reduce((s, p) => s + (p.y - cy) ** 2, 0);
  return piles.map((p) => N / n + (sx > 0 ? (My * (p.x - cx)) / sx : 0) + (sy > 0 ? (Mx * (p.y - cy)) / sy : 0));
}

/** Share of a pile's reaction past a section at s (toward +), by §13.4.2.5. */
const share = (pos: number, s: number, dp: number) => Math.max(0, Math.min(1, (pos - s) / dp + 0.5));

export function checkPileCap(c: PileCapInput): PileCapResult {
  const d = c.thickness - c.cover - 0.016;
  const weight = 25 * c.B * c.L * c.thickness;
  const Mcol = (a: { N: number; Mx: number; My: number }) => ({ My: a.My + a.N * c.column.x, Mx: a.Mx + a.N * c.column.y });
  const sm = Mcol(c.service), fm = Mcol(c.factored);
  const service = pileReactions(c.piles, c.service.N + weight, sm.My, sm.Mx);
  const factored = pileReactions(c.piles, c.factored.N, fm.My, fm.Mx);
  const max = Math.max(...service), min = Math.min(...service);
  const piles = { max, min, capacity: c.pileCapacity, utilization: max / c.pileCapacity, tension: min < -1e-9 };

  const dir = (axis: 'x' | 'y') => {
    const u = (p: { x: number; y: number }) => (axis === 'x' ? p.x : p.y);
    const half = (axis === 'x' ? c.column.cx : c.column.cy) / 2, at = u(c.column), W = axis === 'x' ? c.L : c.B;
    let mu = 0, vu = 0;
    for (const sign of [1, -1]) {
      const face = at + sign * half;
      let m = 0, v = 0;
      c.piles.forEach((p, i) => {
        const beyond = sign * (u(p) - face);
        if (beyond > 0) m += factored[i]! * beyond;
        v += factored[i]! * share(sign * u(p), sign * (face + sign * d), c.pileDiameter);
      });
      mu = Math.max(mu, m); vu = Math.max(vu, v);
    }
    const phiVc = 0.75 * 0.17 * Math.sqrt(Math.min(c.fc, 70)) * W * d * 1000;
    return { mu, as: Math.max((mu * 1000) / (0.9 * 0.9 * d * c.fy * 1e6), 0.0018 * W * c.thickness), shear: { vu, phiVc, utilization: vu / phiVc } };
  };
  const x = dir('x'), y = dir('y');

  // Column punching: the piles whose centres fall inside its critical perimeter carry their load there.
  const hx = (c.column.cx + d) / 2, hy = (c.column.cy + d) / 2;
  const inside = c.piles.reduce((s, p, i) => s + (Math.abs(p.x - c.column.x) <= hx && Math.abs(p.y - c.column.y) <= hy ? factored[i]! : 0), 0);
  const col = checkPunchingShear({
    fc: c.fc, columnB: c.column.cx, columnH: c.column.cy, d, position: 'interior',
    demand: { supportReaction: c.factored.N, loadInsidePerimeter: inside / ((c.column.cx + d) * (c.column.cy + d)), unbalancedMomentX: c.factored.Mx, unbalancedMomentY: c.factored.My },
  });
  // The most loaded pile nearest a corner, on a perimeter cut by the cap's two edges.
  const k = factored.indexOf(Math.max(...factored));
  const corner = checkPunchingShear({ fc: c.fc, columnB: c.pileDiameter, columnH: c.pileDiameter, d, position: 'corner', shape: 'circular', demand: { supportReaction: Math.max(0, factored[k]!), loadInsidePerimeter: 0 } });

  const utilization = Math.max(piles.tension ? Infinity : 0, piles.utilization, x.shear.utilization, y.shear.utilization, col.utilization, corner.utilization);
  return { reactions: { service, factored }, piles, x, y, punching: { column: col.utilization, cornerPile: corner.utilization, checks: [col, corner] }, utilization };
}
