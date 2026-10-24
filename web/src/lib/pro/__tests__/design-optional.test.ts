/**
 * The normative design, off by default and turned on from PRO's settings (`uiStore.designInPro`).
 *
 * Off, PRO offers no Design stage, no place of the desk opens, a design view named from anywhere
 * lands on the model, and a project opened with design in it says so with the switch at hand and
 * keeps it. On, everything is where it was.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { buildProStages } from '../stages';
import { openDeskAt, syncDeskWithTab } from '../design-navigation';
import { deskUi } from '../../store/design-desk-ui.svelte';
import { modelStore, uiStore, historyStore } from '../../store';
import { projectHoldsDesign, noteDesignHeldIfOff } from '../../store/design-presence';

const ctx = {
  solved: true, canSolve: true, canReport: true, onSolve: () => {}, onReport: () => {},
  onRebar3D: () => {}, canRebar3D: () => true, rebar3DMissingSteps: () => [],
};

describe('normative design behind PRO settings', () => {
  beforeEach(() => {
    uiStore.analysisMode = 'pro';
    uiStore.designInPro = false;
    deskUi.open = false;
    uiStore.proActiveTab = 'nodes';
    modelStore.clear(); historyStore.clear();
  });

  // Off as a fresh browser has it (no stored preference); kept per browser, as the e2e checks.
  it('offers only modelling and analysis while off, the Design stage once turned on', () => {
    expect(buildProStages(ctx).map((s) => s.id)).toEqual(['model', 'analyse']);
    uiStore.designInPro = true;
    expect(buildProStages(ctx).map((s) => s.id)).toContain('design');
  });

  it('opens no place of the desk while off, and a design view lands on the model', () => {
    openDeskAt('sheets');
    expect(deskUi.open).toBe(false);
    uiStore.proActiveTab = 'design';
    syncDeskWithTab('design');
    expect(deskUi.open).toBe(false);
    expect(uiStore.proActiveTab).toBe('nodes');
    uiStore.designInPro = true;
    openDeskAt('sheets');
    expect(deskUi.open).toBe(true);
  });

  it('says a project holds design, with the switch at hand, and changes nothing of it', () => {
    const a = modelStore.addNode(0, 0, 0), b = modelStore.addNode(5, 0, 0);
    const e = modelStore.addElement(a, b);
    expect(projectHoldsDesign()).toBe(false);
    modelStore.updateElement(e, { reinforcement: { kind: 'beam' } } as never);
    expect(projectHoldsDesign()).toBe(true);
    const before = uiStore.toasts.length;
    noteDesignHeldIfOff();
    const last = uiStore.toasts.at(-1)!;
    expect(uiStore.toasts.length).toBe(before + 1);
    expect(last.actionId).toBe('enable-design');
    expect(uiStore.designInPro).toBe(false);
    expect((modelStore.elements.get(e) as { reinforcement?: unknown }).reinforcement).toBeTruthy();
  });
});
