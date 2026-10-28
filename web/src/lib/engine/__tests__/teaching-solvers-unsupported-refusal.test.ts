import { describe, it, expect } from 'vitest';
import { solveDetailed } from '../solver-detailed';
import { solveDetailed3D } from '../solver-detailed-3d';
import { StepByStepScopeError } from '../step-by-step-scope';
import type { SolverInput } from '../types';
import type { SolverInput3D } from '../types-3d';

describe('teaching solvers refuse unsupported models explicitly', () => {
  it('solveDetailed refuses an empty model', () => {
    const empty: SolverInput = {
      nodes: new Map(),
      materials: new Map(),
      sections: new Map(),
      elements: new Map(),
      supports: new Map(),
      loads: [],
    };
    expect(() => solveDetailed(empty)).toThrow(StepByStepScopeError);
  });

  it('solveDetailed refuses a model with constraints', () => {
    const input: SolverInput = {
      nodes: new Map([[1, { id: 1, x: 0, z: 0 }], [2, { id: 2, x: 2, z: 0 }]]),
      materials: new Map([[1, { id: 1, e: 200_000, nu: 0.3 }]]),
      sections: new Map([[1, { id: 1, a: 0.01, iz: 1e-4 }]]),
      elements: new Map([[1, { id: 1, type: 'frame', nodeI: 1, nodeJ: 2, materialId: 1, sectionId: 1, hingeStart: false, hingeEnd: false }]]),
      supports: new Map([[1, { id: 1, nodeId: 1, type: 'fixed' }]]),
      loads: [],
      constraints: [{ type: 'rigidLink', masterNode: 1, slaveNode: 2 }],
    };
    expect(() => solveDetailed(input)).toThrow(StepByStepScopeError);
  });

  it('solveDetailed refuses a model with connectors', () => {
    const input: SolverInput = {
      nodes: new Map([[1, { id: 1, x: 0, z: 0 }], [2, { id: 2, x: 2, z: 0 }]]),
      materials: new Map([[1, { id: 1, e: 200_000, nu: 0.3 }]]),
      sections: new Map([[1, { id: 1, a: 0.01, iz: 1e-4 }]]),
      elements: new Map([[1, { id: 1, type: 'frame', nodeI: 1, nodeJ: 2, materialId: 1, sectionId: 1, hingeStart: false, hingeEnd: false }]]),
      supports: new Map([[1, { id: 1, nodeId: 1, type: 'fixed' }]]),
      loads: [],
      connectors: new Map([[1, { id: 1, nodeI: 1, nodeJ: 2, kAxial: 1000 }]]),
    };
    expect(() => solveDetailed(input)).toThrow(StepByStepScopeError);
  });

  it('solveDetailed3D refuses a model with shells (quads or plates)', () => {
    const input: SolverInput3D = {
      nodes: new Map([
        [1, { id: 1, x: 0, y: 0, z: 0 }],
        [2, { id: 2, x: 1, y: 0, z: 0 }],
        [3, { id: 3, x: 1, y: 1, z: 0 }],
        [4, { id: 4, x: 0, y: 1, z: 0 }],
      ]),
      materials: new Map([[1, { id: 1, e: 200_000, nu: 0.3 }]]),
      sections: new Map([[1, { id: 1, a: 0.01, iy: 1e-4, iz: 1e-4, j: 1e-4 }]]),
      elements: new Map([
        [1, { id: 1, type: 'frame', nodeI: 1, nodeJ: 2, materialId: 1, sectionId: 1, releaseMyStart: false, releaseMyEnd: false, releaseMzStart: false, releaseMzEnd: false, releaseTStart: false, releaseTEnd: false }],
      ]),
      supports: new Map([[1, { nodeId: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true }]]),
      loads: [],
      quads: new Map([[1, { id: 1, nodes: [1, 2, 3, 4], materialId: 1, thickness: 0.2 }]]),
    };
    expect(() => solveDetailed3D(input)).toThrow(StepByStepScopeError);
  });

  it('solveDetailed3D refuses a model with constraints', () => {
    const input: SolverInput3D = {
      nodes: new Map([
        [1, { id: 1, x: 0, y: 0, z: 0 }],
        [2, { id: 2, x: 2, y: 0, z: 0 }],
      ]),
      materials: new Map([[1, { id: 1, e: 200_000, nu: 0.3 }]]),
      sections: new Map([[1, { id: 1, a: 0.01, iy: 1e-4, iz: 1e-4, j: 1e-4 }]]),
      elements: new Map([
        [1, { id: 1, type: 'frame', nodeI: 1, nodeJ: 2, materialId: 1, sectionId: 1, releaseMyStart: false, releaseMyEnd: false, releaseMzStart: false, releaseMzEnd: false, releaseTStart: false, releaseTEnd: false }],
      ]),
      supports: new Map([[1, { nodeId: 1, rx: true, ry: true, rz: true, rrx: true, rry: true, rrz: true }]]),
      loads: [],
      constraints: [{ type: 'rigidLink', masterNode: 1, slaveNode: 2 }],
    };
    expect(() => solveDetailed3D(input)).toThrow(StepByStepScopeError);
  });
});
