/**
 * A beam with a slab cast on it bends in the slab's plane only with the slab: the in-plane moment
 * goes to the slab's diaphragm, and the beam is designed about its strong axis alone. In a
 * reconstructed waffle-slab building 601 beam pieces in the slabs' mesh were refused as «biaxial,
 * secondary axis unchecked» for the slab's in-plane action on them.
 */
import { describe, it, expect } from 'vitest';
import { resolveDesignAxes, withSlabDiaphragm } from '../design-axes';

const demands = (my: number, mz: number) => ({
  elementId: 1, length: 5,
  demands: [{ category: 'My+', absValue: my }, { category: 'Mz+', absValue: mz }],
}) as never;

describe('the in-plane axis of a beam under its slab', () => {
  it('is the slab\'s: strong axis governing, biaxial with no slab, not biaxial with one', () => {
    const axes = resolveDesignAxes('beam', { b: 0.3, h: 0.5 }, demands(100, 25));
    expect(axes.flexure).toBe('My');
    expect(axes.biaxial).toBe(true);
    expect(withSlabDiaphragm(axes, false)).toBe(axes);
    const slab = withSlabDiaphragm(axes, true);
    expect(slab.biaxial).toBe(false);
    expect(slab.secondaryCarriedBy).toBe('slab');
  });

  it('a beam bending sideways (its in-plane moment governing) keeps its biaxial check', () => {
    const axes = resolveDesignAxes('beam', { b: 0.3, h: 0.5 }, demands(10, 100));
    expect(axes.flexure).toBe('Mz');
    expect(withSlabDiaphragm(axes, true)).toBe(axes);
  });
});
