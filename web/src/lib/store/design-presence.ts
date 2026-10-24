/**
 * Whether a project holds work of the normative design: bars on a member, detailed assemblies or
 * decisions kept. Opened with design off in PRO's settings, such a project is offered to turn it
 * on (`noteDesignHeldIfOff`); it is never turned on for the reader, and nothing of it is dropped.
 */
import { modelStore } from './model.svelte';
import { uiStore } from './ui.svelte';
import { t } from '../i18n';

export function projectHoldsDesign(): boolean {
  const m = modelStore.model;
  for (const e of m.elements.values()) if ((e as { reinforcement?: unknown }).reinforcement) return true;
  if ((m.detailing?.assemblies.length ?? 0) > 0) return true;
  const d = m.designDecisions;
  return !!d && (d.locked.length > 0 || Object.keys(d.proposals).length > 0 || d.copies.length > 0);
}

/** A PRO project opened with design off that holds design: say so, with the switch at hand. */
export function noteDesignHeldIfOff(): void {
  if (uiStore.analysisMode !== 'pro' || uiStore.designInPro || !projectHoldsDesign()) return;
  uiStore.toast(t('design.heldWhileOff'), 'info', 'enable-design');
}
