/**
 * Run the plastic collapse analysis on the model as it stands.
 *
 * Delegated to the Rust WASM engine (`engine/src/solver/plastic.rs` -> `solvePlastic`).
 * Mp of each frame member from its section's own plastic modulus
 * (plastic-moments.ts); Np = fy·A for a truss bar, with fy from its material
 * (250 MPa if it has none, and said so by `plasticMoments`).
 */
import { modelStore, uiStore } from '../store';
import { plasticMoments, DEFAULT_FY, type SectionMp } from '../engine/plastic-moments';
import { advancedRefusal2D, variableRefusal2D } from '../engine/solver-service';
import { t } from '../i18n';
import { solvePlastic, isWasmReady, input2DToWireObject } from '../engine/wasm-solver';
import { EngineUnavailableError } from '../engine/engine-unavailable';
import { tagCertifyJob } from '../engine/certify-tags';
import type { SolverInput } from '../engine/types';
import type { PlasticCollapseResult, PlasticCollapseHinge } from '../engine/result-types';

export interface PlasticRun {
  result: PlasticCollapseResult;
  mps: SectionMp[];
}

/**
 * Throws, with the static solve's own message, on a model the static solve
 * refuses: the collapse analysis starts from that solve, and ran on a beam on
 * two rollers or a stray node as if it had one. A model nothing loads is
 * refused too: its λc = 0 read as "does not collapse".
 */
export function runPlasticCollapse(): PlasticRun | null {
  const input = modelStore.buildSolverInput(uiStore.includeSelfWeight);
  if (!input) return null;
  const refusal = variableRefusal2D(modelStore.model) ?? advancedRefusal2D(input);
  if (refusal) throw new Error(refusal);

  if (!isWasmReady()) {
    throw new EngineUnavailableError('plastic collapse');
  }

  const mps = plasticMoments(modelStore.sections, modelStore.materials, modelStore.elements);
  const mpOfSection = new Map(mps.map((m) => [m.sectionId, m.mp]));

  const sections = new Map<number, { a: number; iz: number; materialId: number }>();
  const mpOverrides = new Map<number, number>();
  for (const e of input.elements.values()) {
    const s = input.sections.get(e.sectionId);
    if (!s || sections.has(e.sectionId)) continue;
    const mp = mpOfSection.get(e.sectionId);
    if (mp !== undefined && Number.isFinite(mp) && mp > 0) {
      sections.set(e.sectionId, { a: s.a, iz: s.iz, materialId: e.materialId });
      mpOverrides.set(e.sectionId, mp);
    }
  }
  const materials = new Map<number, { fy?: number }>();
  for (const id of input.materials.keys()) {
    materials.set(id, { fy: modelStore.materials.get(id)?.fy || DEFAULT_FY });
  }

  let engineRes: any;
  try {
    engineRes = solvePlastic({
      solver: input,
      sections,
      materials,
      mpOverrides,
    });
  } catch (err: any) {
    if (err && typeof err.message === 'string' && err.message.includes('No mechanical load')) {
      throw new Error(t('advanced.plasticNoLoads'));
    }
    throw err;
  }

  // Map engine hinges to include x coordinate along member
  const mapHinge = (h: any): PlasticCollapseHinge => {
    const elem = input.elements.get(h.elementId);
    let l = 0;
    if (elem) {
      const ni = input.nodes.get(elem.nodeI);
      const nj = input.nodes.get(elem.nodeJ);
      if (ni && nj) {
        const dx = nj.x - ni.x;
        const dz = nj.z - ni.z;
        l = Math.sqrt(dx * dx + dz * dz);
      }
    }
    const pos = h.position !== undefined ? h.position : (h.end === 'end' ? 1 : 0);
    return {
      elementId: h.elementId,
      end: h.end,
      moment: h.moment,
      loadFactor: h.loadFactor,
      step: h.step,
      position: h.position,
      unloadedAt: h.unloadedAt,
      x: pos * l,
      kind: 'moment',
    };
  };

  const hinges: PlasticCollapseHinge[] = (engineRes.hinges || []).map(mapHinge);
  const steps = (engineRes.steps || []).map((st: any) => ({
    loadFactor: st.loadFactor,
    hingesFormed: (st.hingesFormed || []).map(mapHinge),
    results: st.results,
  }));

  const result: PlasticCollapseResult = {
    collapseFactor: engineRes.collapseFactor,
    steps,
    hinges,
    isMechanism: engineRes.isMechanism,
    redundancy: engineRes.redundancy,
    degree: engineRes.redundancy,
    unloaded: false,
    warnings: engineRes.warnings,
  };

  tagPlasticCertificate(result, input, mpOfSection);
  return { result, mps };
}

/**
 * Note how to certify the collapse factor (`certify_plastic_2d`, docs/verification/CERTIFY.md):
 * the same model and the same Mp per section, sent as the engine's plastic input. The certificate
 * is of the rigid–plastic model with bending-only yield; `PlasticCertificate.svelte` reads it
 * against λc only when no bar yielded axially. A section with no Mp (one this analysis takes as
 * never yielding) has no counterpart there, and the result is not certified.
 */
function tagPlasticCertificate(result: object, input: SolverInput, mpOfSection: Map<number, number>): void {
  const sections: Record<string, { a: number; iz: number; materialId: number }> = {};
  const mpOverrides: Record<string, number> = {};
  for (const e of input.elements.values()) {
    const s = input.sections.get(e.sectionId);
    if (!s || String(e.sectionId) in sections) continue;
    const mp = mpOfSection.get(e.sectionId);
    if (!(mp !== undefined && Number.isFinite(mp) && mp > 0)) return;
    sections[String(e.sectionId)] = { a: s.a, iz: s.iz, materialId: e.materialId };
    mpOverrides[String(e.sectionId)] = mp;
  }
  const materials = Object.fromEntries([...input.materials.keys()].map((id) => [String(id), { fy: modelStore.materials.get(id)?.fy || DEFAULT_FY }]));
  const json = JSON.stringify({ solver: input2DToWireObject(input), sections, materials, mpOverrides });
  tagCertifyJob(result, { name: 'certify_plastic_2d', args: [json] });
}
