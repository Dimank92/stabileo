/**
 * The envelope the build timelapse puts on a building: its façade bays, found from the beams and
 * slab edges on each floor's outline, split at the columns, and left open where a wall closes them.
 */
import { describe, it, expect } from 'vitest';
import { facadeBays, envelopeSchedule } from '../build-envelope';
import type { BuildPiece, P3 } from '../build-timelapse';

/** A frame two bays by one, `floors` high (3 m each): columns at every grid point, beams round and across. */
function frame(floors: number, extra: BuildPiece[] = []): BuildPiece[] {
  const xs = [0, 5, 10], ys = [0, 6];
  const out: BuildPiece[] = [];
  let k = 0;
  const m = (a: P3, b: P3) => out.push({ key: `e${k++}`, kind: 'member', a, b, nodes: [k * 2, k * 2 + 1] });
  for (let f = 0; f < floors; f++) {
    const z0 = 3 * f, z1 = 3 * (f + 1);
    for (const x of xs) for (const y of ys) m({ x, y, z: z0 }, { x, y, z: z1 });
    for (const y of ys) for (let i = 0; i + 1 < xs.length; i++) m({ x: xs[i]!, y, z: z1 }, { x: xs[i + 1]!, y, z: z1 });
    for (const x of xs) m({ x, y: 0, z: z1 }, { x, y: 6, z: z1 });
  }
  return [...out, ...extra];
}

describe('façade bays', () => {
  it('every floor closes its outline, split at the columns; the inner frame stays open', () => {
    const bays = facadeBays(frame(2));
    // Per floor: two bays on each long side, one on each short side; the middle frame is inside.
    expect(bays).toHaveLength(12);
    expect(bays.filter((b) => b.floor === 1).every((b) => b.z0 === 0 && b.z1 === 3)).toBe(true);
    expect(bays.filter((b) => b.floor === 2).every((b) => b.z0 === 3 && b.z1 === 6)).toBe(true);
    // Each bay faces out of the plan.
    for (const b of bays) {
      const mid = { x: (b.a.x + b.b.x) / 2 - 5, y: (b.a.y + b.b.y) / 2 - 3 };
      expect(mid.x * b.out.x + mid.y * b.out.y).toBeGreaterThan(0);
    }
  });

  it('a wall on the façade closes its bay', () => {
    const wall: BuildPiece = { key: 'q1', kind: 'shell', nodes: [1, 2, 3, 4], pts: [{ x: 0, y: 0, z: 0 }, { x: 0, y: 6, z: 0 }, { x: 0, y: 6, z: 3 }, { x: 0, y: 0, z: 3 }] };
    expect(facadeBays(frame(1, [wall]))).toHaveLength(5);
  });

  it('a lone beam or a flat model has no façade', () => {
    expect(facadeBays([{ key: 'e1', kind: 'member', a: { x: 0, y: 0, z: 0 }, b: { x: 5, y: 0, z: 0 }, nodes: [1, 2] }])).toEqual([]);
  });

  it('bays go on floor by floor, inside the time given, glass after cladding', () => {
    const steps = envelopeSchedule(facadeBays(frame(2)), 10, 4);
    expect(steps).toHaveLength(12);
    const floors = steps.map((s) => s.bay.floor);
    expect(floors).toEqual([...floors].sort((a, b) => a - b));
    for (const s of steps) {
      expect(s.cladding.start).toBeGreaterThanOrEqual(10);
      expect(s.glass.start).toBeGreaterThan(s.cladding.start);
      expect(s.glass.end).toBeLessThanOrEqual(14 + 1e-9);
    }
  });
});
