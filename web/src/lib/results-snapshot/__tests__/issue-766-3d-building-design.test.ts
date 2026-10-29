import { beforeAll, describe, it, expect } from 'vitest';
import { initSolver } from '../../engine/wasm-solver';
import { capture } from '../capture';
import { concreteFc } from '../../model/design-data';
import { materialFamilyOf } from '../../model/material-family';

beforeAll(async () => {
  await initSolver();
});

describe('Issue #766: 3d-building in PRO with design on', () => {
  it('concreteFc and materialFamilyOf recognise H-30 with no explicit fc or fy', () => {
    const rawMaterial = { id: 2, name: 'H-30', e: 32000, nu: 0.2, rho: 25 };
    expect(concreteFc(rawMaterial)).toBe(30);
    expect(materialFamilyOf(rawMaterial).family).toBe('concrete');
  });

  it('recognises H-30 columns and beams as concrete members rather than reporting noConcreteMembers', async () => {
    const snap = await capture({
      name: 'design-3d-building',
      source: { example: '3d-building' },
      workspace: 'pro',
      design: true,
      group: 'design',
      covers: '3D building in PRO with design on',
    });

    expect(snap.values['design.codeCheck.ok']).toBe(true);
    expect(snap.values['design.codeCheck.refusal']).toBeUndefined();
  });
});
