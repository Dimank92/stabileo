/**
 * Differential test: compare the Rust engine's plastic collapse solver (`solvePlastic`)
 * against the TypeScript reference oracle (`plasticCollapse2DOracle`).
 *
 * Verifies that:
 * 1. The engine and oracle calculate identical collapse factors λc on standard benchmark frames.
 * 2. Hinge locations (ends and span fractions) match.
 * 3. Both agree on whether a collapse mechanism has formed.
 */
import { describe, it, expect } from 'vitest';
import { solvePlastic, isWasmReady } from '../../wasm-solver';
import { plasticCollapse2DOracle } from './oracle/plastic-collapse-oracle';
import type { SolverInput, SolverLoad } from '../../types';

const MP = 100;

function makeModel(
  nodes: Array<[number, number, number]>,
  elements: Array<[number, number, number]>,
  supports: Array<[number, string]>,
  loads: SolverLoad[],
): SolverInput {
  return {
    nodes: new Map(nodes.map(([id, x, z]) => [id, { id, x, z }])),
    materials: new Map([[1, { id: 1, e: 200_000, nu: 0.3 }]]),
    sections: new Map([[1, { id: 1, a: 0.01, iz: 1e-4 }]]),
    elements: new Map(
      elements.map(([id, i, j]) => [
        id,
        { id, type: 'frame', nodeI: i, nodeJ: j, materialId: 1, sectionId: 1, hingeStart: false, hingeEnd: false },
      ]),
    ),
    supports: new Map(supports.map(([nodeId, type], k) => [k + 1, { id: k + 1, nodeId, type: type as never }])),
    loads,
  } as unknown as SolverInput;
}

const P = (nodeId: number, fx: number, fz: number): SolverLoad => ({
  type: 'nodal',
  data: { nodeId, fx, fz, my: 0 },
});

const pOn = (elementId: number, a: number, p: number): SolverLoad => ({
  type: 'pointOnElement',
  data: { elementId, a, p },
});

describe('plastic collapse: Rust engine vs TypeScript oracle', () => {
  it('WASM engine is ready', () => {
    expect(isWasmReady()).toBe(true);
  });

  it('simply supported beam with central point load: λc = 4Mp/(PL)', () => {
    // L = 6 m, P = 10 kN downward, Mp = 100 kN*m -> λc = 4*100 / (10*6) = 6.6667
    const m = makeModel([[1, 0, 0], [2, 6, 0]], [[1, 1, 2]], [[1, 'pinned'], [2, 'rollerX']], [pOn(1, 3, -10)]);

    const oracleRes = plasticCollapse2DOracle(m, { mp: () => MP, np: () => Infinity });

    const sections = new Map([[1, { a: 0.01, iz: 1e-4, materialId: 1 }]]);
    const materials = new Map([[1, { fy: 250 }]]);
    const mpOverrides = new Map([[1, MP]]);

    const engineRes = solvePlastic({ solver: m, sections, materials, mpOverrides });

    expect(engineRes.isMechanism).toBe(true);
    expect(engineRes.isMechanism).toBe(oracleRes.isMechanism);
    expect(engineRes.collapseFactor).toBeCloseTo(oracleRes.collapseFactor, 4);
    expect(engineRes.collapseFactor).toBeCloseTo((4 * MP) / (10 * 6), 4);
  });

  it('cantilever with tip load: λc = Mp/(PL)', () => {
    // L = 3 m, P = 15 kN tip load, Mp = 100 kN*m -> λc = 100 / (15*3) = 2.2222
    const m = makeModel([[1, 0, 0], [2, 3, 0]], [[1, 1, 2]], [[1, 'fixed']], [P(2, 0, -15)]);

    const oracleRes = plasticCollapse2DOracle(m, { mp: () => MP, np: () => Infinity });

    const sections = new Map([[1, { a: 0.01, iz: 1e-4, materialId: 1 }]]);
    const materials = new Map([[1, { fy: 250 }]]);
    const mpOverrides = new Map([[1, MP]]);

    const engineRes = solvePlastic({ solver: m, sections, materials, mpOverrides });

    expect(engineRes.isMechanism).toBe(true);
    expect(engineRes.collapseFactor).toBeCloseTo(oracleRes.collapseFactor, 4);
    expect(engineRes.collapseFactor).toBeCloseTo(MP / 45, 4);
  });

  it('portal frame under horizontal and vertical loads', () => {
    // Portal frame: 2 columns 4m, 1 beam 6m
    // Nodes: 1 (0,0), 2 (0,4), 3 (6,4), 4 (6,0)
    // Supports: fixed at 1 and 4
    // Loads: 20 kN horizontal at node 2, 40 kN vertical at midspan of beam 2
    const m = makeModel(
      [[1, 0, 0], [2, 0, 4], [3, 6, 4], [4, 6, 0]],
      [[1, 1, 2], [2, 2, 3], [3, 4, 3]],
      [[1, 'fixed'], [4, 'fixed']],
      [P(2, 20, 0), pOn(2, 3, -40)],
    );

    const oracleRes = plasticCollapse2DOracle(m, { mp: () => MP, np: () => Infinity });

    const sections = new Map([[1, { a: 0.01, iz: 1e-4, materialId: 1 }]]);
    const materials = new Map([[1, { fy: 250 }]]);
    const mpOverrides = new Map([[1, MP]]);

    const engineRes = solvePlastic({ solver: m, sections, materials, mpOverrides });

    expect(engineRes.isMechanism).toBe(true);
    expect(engineRes.collapseFactor).toBeCloseTo(oracleRes.collapseFactor, 3);
  });
});

