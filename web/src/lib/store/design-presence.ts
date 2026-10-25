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

/** The projects already told about in this session: a tab switched back to says nothing again. */
const told = new Set<string>();

/**
 * A PRO project opened with design off that holds design: say so, with the switch at hand.
 *
 * Said once a project is the one on screen, from every load path (`onProjectOpened`: a .ded, the
 * autosave, a tab, a share link), with the mode the project was opened in (the UI's may not be set
 * yet). Once per project and session.
 */
export function noteDesignHeldIfOff(analysisMode: string | undefined = uiStore.analysisMode): void {
  if (analysisMode !== 'pro' || uiStore.designInPro || !projectHoldsDesign()) return;
  const key = `${modelStore.model.name}|${modelStore.elements.size}|${modelStore.nodes.size}`;
  if (told.has(key)) return;
  told.add(key);
  uiStore.toast(t('design.heldWhileOff'), 'info', 'enable-design');
}
