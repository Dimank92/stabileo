/**
 * The moves between design and the model, in one place, so every button that makes one makes
 * the same move: «Show in the model», a diagnostics row, a detailing conflict, a rail entry that
 * opens a classic screen, a «Prepare» entry that opens a modelling tool.
 *
 * ── Framing ─────────────────────────────────────────────────────────
 *
 * Both viewers already frame what is selected on `stabileo-zoom-to-selection`, keeping the
 * camera's direction and fitting its distance to the selection's nodes. A move that changes the
 * layout (design closing gives the viewer the whole body row back) is framed two frames later,
 * once the viewer's ResizeObserver has given the canvas its new size: framed in the same tick,
 * the fit measures the size the canvas is about to stop being.
 */
import { uiStore, modelStore } from '../store';
import { deskUi } from '../store/design-desk-ui.svelte';

/** Ask the model viewer to frame the current selection, once the layout has settled. */
export function frameSelection(): void {
  if (typeof window === 'undefined') return;
  const fire = () => window.dispatchEvent(new CustomEvent('stabileo-zoom-to-selection'));
  if (typeof requestAnimationFrame === 'undefined') { fire(); return; }
  requestAnimationFrame(() => requestAnimationFrame(fire));
}

/** Select these members in the model and frame them where the viewer is now. */
export function selectAndFrame(ids: readonly number[]): void {
  if (ids.length === 0) return;
  uiStore.setSelection(new Set(), new Set(ids), true);
  frameSelection();
}

/**
 * «Show in the model»: the members selected, design left for the model stage (on its members'
 * table, which lists them selected), and the viewer framing them from where it looks.
 */
export function showInModel(ids: readonly number[]): void {
  if (ids.length > 0) uiStore.setSelection(new Set(), new Set(ids), true);
  deskUi.open = false;
  uiStore.proActiveTab = 'elements';
  uiStore.proPanelVisible = true;
  frameSelection();
}

/** A classic design screen the desk does not hold yet, beside the viewer. */
export function openClassic(tab: string): void {
  uiStore.proActiveTab = tab;
  uiStore.proPanelVisible = true;
  deskUi.open = false;
}

/** A modelling tool opened from design's «Prepare»: the model stage, on that tool's panel. */
export function openModelTool(tab: string): void {
  deskUi.open = false;
  uiStore.proActiveTab = tab;
  uiStore.proPanelVisible = true;
  uiStore.currentTool = 'select';
}

/**
 * Whether design has nothing to work on yet: no member it reads (`rows`) and no bars anywhere in
 * the model. Then it offers its two ways in rather than an empty plan.
 */
export function isDesignEmpty(rowCount: number): boolean {
  if (rowCount > 0) return false;
  for (const e of modelStore.elements.values()) if (e.reinforcement) return false;
  return true;
}
