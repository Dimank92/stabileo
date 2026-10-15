/**
 * The building's envelope for the build timelapse: cladding and glass put on the façades once
 * the structure stands. It is a picture, not part of the model: nothing here is analysed.
 *
 * Pure. From the pieces the timelapse builds (scene space, Z up) it finds the floors and, on each,
 * the façade lines: the beams and slab edges that lie on the outline of that floor's plan (its
 * convex hull, so a re-entrant corner of an L-shaped plan is left open). Each façade line between
 * a floor and the one below is split at its columns into bays, and a bay a wall (a vertical
 * shell) already closes is left out. `envelopeSchedule` then puts the bays up floor by floor, each
 * floor going round the building.
 */
import type { BuildPiece, BuildStep, P3 } from './build-timelapse';

interface P2 { x: number; y: number }

export interface EnvelopeBay {
  key: string;
  /** The bay's ends on the façade line, in plan. */
  a: P2;
  b: P2;
  /** Its foot (the floor below, or the ground) and its head (its own floor). */
  z0: number;
  z1: number;
  /** Unit plan normal pointing out of the building. */
  out: P2;
  /** Floor index, from 1 at the first floor above the ground. */
  floor: number;
}

const cross2 = (o: P2, a: P2, b: P2) => (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

/** Counter-clockwise convex hull (Andrew's monotone chain). */
function hull(points: P2[]): P2[] {
  const pts = [...points].sort((p, q) => p.x - q.x || p.y - q.y);
  if (pts.length < 3) return pts;
  const lower: P2[] = [], upper: P2[] = [];
  for (const p of pts) {
    while (lower.length >= 2 && cross2(lower[lower.length - 2]!, lower[lower.length - 1]!, p) <= 1e-12) lower.pop();
    lower.push(p);
  }
  for (let i = pts.length - 1; i >= 0; i--) {
    const p = pts[i]!;
    while (upper.length >= 2 && cross2(upper[upper.length - 2]!, upper[upper.length - 1]!, p) <= 1e-12) upper.pop();
    upper.push(p);
  }
  return [...lower.slice(0, -1), ...upper.slice(0, -1)];
}

/** Where `p` falls along the edge a→b, as a length from a, and how far off its line it is. */
function onEdge(a: P2, b: P2, p: P2): { s: number; off: number; len: number } {
  const dx = b.x - a.x, dy = b.y - a.y, len = Math.hypot(dx, dy) || 1;
  const ux = dx / len, uy = dy / len;
  const rx = p.x - a.x, ry = p.y - a.y;
  return { s: rx * ux + ry * uy, off: Math.abs(-rx * uy + ry * ux), len };
}

/** The façade bays of the structure; none for a model with no floor above its foot. */
export function facadeBays(pieces: readonly BuildPiece[]): EnvelopeBay[] {
  if (pieces.length === 0) return [];
  let zMin = Infinity, span = 0;
  {
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity, z1 = -Infinity;
    for (const p of pieces) for (const q of p.kind === 'member' ? [p.a, p.b] : p.pts) {
      zMin = Math.min(zMin, q.z); z1 = Math.max(z1, q.z);
      x0 = Math.min(x0, q.x); x1 = Math.max(x1, q.x); y0 = Math.min(y0, q.y); y1 = Math.max(y1, q.y);
    }
    span = Math.max(x1 - x0, y1 - y0, z1 - zMin, 1e-6);
  }
  const tol = Math.max(0.05, span * 0.005);
  const band = (z: number) => Math.round(z / tol);

  // Horizontal lines (beams, slab edges) by floor; columns and walls to split and skip bays.
  const lines = new Map<number, Array<{ a: P2; b: P2; z: number }>>();
  const addLine = (a: P3, b: P3) => {
    const z = (a.z + b.z) / 2;
    if (z <= zMin + tol) return;
    const k = band(z);
    const list = lines.get(k) ?? [];
    list.push({ a: { x: a.x, y: a.y }, b: { x: b.x, y: b.y }, z });
    lines.set(k, list);
  };
  const columns: Array<{ p: P2; z0: number; z1: number }> = [];
  const walls: Array<{ a: P2; b: P2; z0: number; z1: number }> = [];
  for (const p of pieces) {
    if (p.kind === 'member') {
      const dz = Math.abs(p.b.z - p.a.z), L = Math.hypot(p.b.x - p.a.x, p.b.y - p.a.y, dz);
      if (L < 1e-9) continue;
      if (dz / L < 0.05) addLine(p.a, p.b);
      else if (dz / L > 0.95) columns.push({ p: { x: (p.a.x + p.b.x) / 2, y: (p.a.y + p.b.y) / 2 }, z0: Math.min(p.a.z, p.b.z), z1: Math.max(p.a.z, p.b.z) });
    } else {
      const zs = p.pts.map((q) => q.z);
      const lo = Math.min(...zs), hi = Math.max(...zs);
      if (hi - lo < tol) {
        for (let i = 0; i < p.pts.length; i++) addLine(p.pts[i]!, p.pts[(i + 1) % p.pts.length]!);
      } else {
        // A wall: its plan trace, the two points farthest apart.
        let best: [P3, P3] = [p.pts[0]!, p.pts[0]!], d = -1;
        for (const q of p.pts) for (const r of p.pts) {
          const e = Math.hypot(q.x - r.x, q.y - r.y);
          if (e > d) { d = e; best = [q, r]; }
        }
        if (d > tol) walls.push({ a: { x: best[0].x, y: best[0].y }, b: { x: best[1].x, y: best[1].y }, z0: lo, z1: hi });
      }
    }
  }

  const floors = [...lines.keys()].sort((a, b) => a - b);
  const bays: EnvelopeBay[] = [];
  floors.forEach((k, fi) => {
    const segs = lines.get(k)!;
    const z1 = segs.reduce((s, l) => s + l.z, 0) / segs.length;
    const z0 = fi === 0 ? zMin : lines.get(floors[fi - 1]!)!.reduce((s, l) => s + l.z, 0) / lines.get(floors[fi - 1]!)!.length;
    const ring = hull(segs.flatMap((l) => [l.a, l.b]));
    if (ring.length < 3) return;
    for (let e = 0; e < ring.length; e++) {
      const A = ring[e]!, B = ring[(e + 1) % ring.length]!;
      // The parts of this hull edge the floor's lines cover, and where its columns stand.
      const cover: Array<[number, number]> = [];
      const len = Math.hypot(B.x - A.x, B.y - A.y) || 1;
      for (const l of segs) {
        const p = onEdge(A, B, l.a), q = onEdge(A, B, l.b);
        if (p.off > tol || q.off > tol) continue;
        const s0 = Math.max(0, Math.min(p.s, q.s)), s1 = Math.min(p.len, Math.max(p.s, q.s));
        if (s1 - s0 > tol) cover.push([s0, s1]);
      }
      if (cover.length === 0) continue;
      cover.sort((u, v) => u[0] - v[0]);
      const merged: Array<[number, number]> = [];
      for (const c of cover) {
        const last = merged[merged.length - 1];
        if (last && c[0] <= last[1] + tol) last[1] = Math.max(last[1], c[1]);
        else merged.push([c[0], c[1]]);
      }
      const cuts: number[] = [];
      for (const c of columns) {
        if (c.z1 < z0 + tol || c.z0 > z1 - tol) continue;
        const r = onEdge(A, B, c.p);
        if (r.off <= tol) cuts.push(r.s);
      }
      const ux = (B.x - A.x) / len, uy = (B.y - A.y) / len;
      // Counter-clockwise ring: the outside is to the right of each edge.
      const out = { x: uy, y: -ux };
      for (const [m0, m1] of merged) {
        const stops = [m0, ...cuts.filter((s) => s > m0 + tol && s < m1 - tol).sort((u, v) => u - v), m1];
        // No column anywhere on the line (a flat slab's edge): bays of about 3 m.
        if (cuts.length === 0 && m1 - m0 > 4.5) {
          const n = Math.round((m1 - m0) / 3);
          stops.splice(1, 0, ...Array.from({ length: n - 1 }, (_, i) => m0 + ((i + 1) * (m1 - m0)) / n));
        }
        for (let i = 0; i + 1 < stops.length; i++) {
          const s0 = stops[i]!, s1 = stops[i + 1]!;
          const a = { x: A.x + ux * s0, y: A.y + uy * s0 }, b = { x: A.x + ux * s1, y: A.y + uy * s1 };
          const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }, zm = (z0 + z1) / 2;
          const closed = walls.some((w) => {
            const r = onEdge(w.a, w.b, mid);
            return r.off <= tol * 2 && r.s >= -tol && r.s <= r.len + tol && zm >= w.z0 - tol && zm <= w.z1 + tol;
          });
          if (!closed) bays.push({ key: `f${fi + 1}:${e}:${i}:${Math.round(s0 * 100)}`, a, b, z0, z1, out, floor: fi + 1 });
        }
      }
    }
  });
  return bays;
}

/** Each bay's turn: its cladding, then its glass a little after. */
export interface EnvelopeStep {
  bay: EnvelopeBay;
  cladding: BuildStep;
  glass: BuildStep;
}

/**
 * When each bay goes on, from `start` over `duration` seconds: floor by floor from the bottom,
 * each floor round the building. A bay rises from its foot like a curtain.
 */
export function envelopeSchedule(bays: readonly EnvelopeBay[], start: number, duration: number): EnvelopeStep[] {
  if (bays.length === 0) return [];
  let cx = 0, cy = 0;
  for (const b of bays) { cx += (b.a.x + b.b.x) / 2; cy += (b.a.y + b.b.y) / 2; }
  cx /= bays.length; cy /= bays.length;
  const angle = (b: EnvelopeBay) => Math.atan2((b.a.y + b.b.y) / 2 - cy, (b.a.x + b.b.x) / 2 - cx);
  const order = [...bays].sort((p, q) => p.floor - q.floor || angle(p) - angle(q));
  const n = order.length;
  const grow = Math.min(0.8, Math.max(0.2, (duration / n) * 8), duration * 0.4);
  const spacing = n > 1 ? Math.max(0, duration - grow * 1.4) / (n - 1) : 0;
  return order.map((bay, i) => {
    const t = start + i * spacing;
    const foot = (z: number): P3 => ({ x: (bay.a.x + bay.b.x) / 2, y: (bay.a.y + bay.b.y) / 2, z });
    const rise = (t0: number, z: number): BuildStep => ({
      key: `${bay.key}`, start: t0, end: t0 + grow, origin: foot(z), dir: { x: 0, y: 0, z: 1 }, kind: 'member', top: bay.z1,
    });
    return { bay, cladding: rise(t, bay.z0), glass: rise(t + grow * 0.4, bay.z0) };
  });
}
