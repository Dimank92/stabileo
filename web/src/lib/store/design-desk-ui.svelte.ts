/**
 * The design desk's view state: whether it is open, the level and the member it is on, the
 * canvas tab and the navigator's filters, and the last «Design» run's report. View state only;
 * none of it is saved with the project.
 *
 * ── One selection ───────────────────────────────────────────────────
 *
 * The member design is on is not a selection of its own. It is read from the model's selection
 * (`uiStore.selectedElements`) and picking a member in design selects it in the model, so the
 * viewer, the navigator, the plan and the 3-D view name the same member and a member the model
 * deletes leaves every one of them at once. What design keeps is only which one it shows when the
 * model has several selected (`focus`).
 *
 * ── Per project tab ─────────────────────────────────────────────────
 *
 * `capture` and `restore` travel with the tab (`tabs.svelte.ts`), and a new project starts from
 * the defaults (`reset`), so one tab's level, filter and editor never show up in another.
 */
import type { DeskReport } from './design-desk';
import { uiStore } from './ui.svelte';
import { modelStore } from './model.svelte';
import { sortedLevels } from '../model/grid';
import { designDrafts } from './design-drafts.svelte';
import { tp } from '../i18n';

export type DeskTab = 'prepare' | 'plan' | 'frame' | 'column' | 'wall' | 'slab' | 'joist' | 'footing' | 'seismic' | 'joint' | 'levels' | 'report' | 'geometry' | 'materials' | 'special' | 'stairs' | 'pt' | 'elements' | 'fire' | 'codes' | 'budget' | 'collab' | 'arch' | 'site' | 'documents';

/**
 * How the desk's views are grouped. Preparing the model comes first. Then three ways of working:
 * by floor (the plan and the levels), by the piece selected (its editor) and by what is pending
 * (the report and the geometry). The output of the work (documents) follows. Specialties that act
 * on the whole building and the project's own surfaces (budget, coordination, architecture,
 * works) are menus: they do not depend on what is selected.
 */
export const DESK_GROUPS = {
  prepare: ['prepare'],
  floor: ['plan', 'levels'],
  piece: ['frame', 'column', 'wall', 'slab', 'joist', 'special', 'pt', 'stairs', 'footing', 'joint', 'elements'],
  pending: ['report', 'geometry'],
  documents: ['documents'],
  specialties: ['seismic', 'fire', 'materials', 'codes'],
  project: ['budget', 'collab', 'arch', 'site'],
} as const satisfies Record<string, readonly DeskTab[]>;
export type DeskGroup = keyof typeof DESK_GROUPS;
export const groupOf = (t: DeskTab): DeskGroup => (Object.keys(DESK_GROUPS) as DeskGroup[]).find((g) => (DESK_GROUPS[g] as readonly DeskTab[]).includes(t)) ?? 'floor';
export type DeskFilter = 'all' | 'fail' | 'warn' | 'ok' | 'unavailable' | 'locked' | 'edited' | 'steel';

/** What a project tab keeps of design when another tab takes the screen. */
export interface DeskViewState {
  open: boolean;
  levelZ: number | null;
  focus: number | null;
  tab: DeskTab;
  filter: DeskFilter;
  /** The modelling level design last took over, to tell a new one from the same one. */
  modelLevelSeen: number;
}

const SAME_Z = 1e-3;

function createDesk() {
  let open = $state(false);
  let levelZ = $state<number | null>(null);
  let focus = $state<number | null>(null);
  let tab = $state<DeskTab>('plan');
  let filter = $state<DeskFilter>('all');
  let report = $state<DeskReport | null>(null);
  /*
   * Not reactive: read and written only on entering and leaving. The default is the default
   * modelling level, so a model nobody moved off the ground opens design on «all levels».
   */
  let modelLevelSeen = 0;

  /** The member design shows: the focused one while it is selected, else the first selected. */
  function current(): number | null {
    const sel = uiStore.selectedElements;
    if (sel.size === 0) return null;
    if (focus !== null && sel.has(focus)) return focus;
    let first = Infinity;
    for (const id of sel) if (id < first) first = id;
    return first;
  }

  /** Entering design: a level chosen in the model since the last visit becomes design's level. */
  function enter(): void {
    const z = uiStore.nodeCreateZ;
    if (Math.abs(z - modelLevelSeen) <= SAME_Z) return;
    modelLevelSeen = z;
    const level = sortedLevels(modelStore.grid).find((l) => Math.abs(l.z - z) < SAME_Z);
    if (level) levelZ = level.z;
  }

  /**
   * Leaving design: the level worked on in design is the model's active level when the person
   * comes back to the model, and the drafts still open are announced, not applied or dropped.
   */
  function leave(): void {
    if (levelZ !== null && Math.abs(levelZ - uiStore.nodeCreateZ) > SAME_Z) uiStore.nodeCreateZ = levelZ;
    modelLevelSeen = uiStore.nodeCreateZ;
    announceDrafts();
  }

  return {
    get open() { return open; },
    set open(v: boolean) {
      if (v === open) return;
      open = v;
      if (v) enter(); else leave();
    },
    get levelZ() { return levelZ; }, set levelZ(v: number | null) { levelZ = v; },
    /** Design's current member, read from the model selection. */
    get selected() { return current(); },
    /** Make `v` design's current member: it becomes the model's selection. */
    set selected(v: number | null) {
      focus = v;
      if (v === null) return;
      const sel = uiStore.selectedElements;
      if (!(sel.size === 1 && sel.has(v))) uiStore.selectElement(v);
    },
    get tab() { return tab; }, set tab(v: DeskTab) { tab = v; },
    get filter() { return filter; }, set filter(v: DeskFilter) { filter = v; },
    get report() { return report; }, set report(v: DeskReport | null) { report = v; },

    capture(): DeskViewState {
      return { open, levelZ, focus, tab, filter, modelLevelSeen };
    },
    /** Take a tab's design state back. `reopen` false keeps design closed (a session restored at boot). */
    restore(s: DeskViewState | undefined, reopen = true) {
      if (!s) { this.reset(); return; }
      levelZ = s.levelZ; focus = s.focus; tab = s.tab; filter = s.filter;
      modelLevelSeen = s.modelLevelSeen;
      report = null;
      open = reopen && s.open;
    },
    /** A new project: the defaults. Design stays open or closed as it is. */
    reset() {
      levelZ = null; focus = null; tab = 'plan'; filter = 'all'; report = null;
      modelLevelSeen = uiStore.nodeCreateZ;
    },
  };
}

/** Say that drafts are kept, when there are any. */
export function announceDrafts(): void {
  const n = designDrafts.count;
  if (n > 0) uiStore.toast(tp('desk.draftsKept', { n }), 'info');
}

export const deskUi = createDesk();

/*
 * The height of design's canvas under the viewer when docked, dragged from its top edge and kept
 * for this browser: a per-viewer convenience, so a private window without storage only starts
 * from the default.
 */
const CANVAS_KEY = 'stabileo-desk-canvas-h';
function createDeskLayout() {
  let canvasH = $state(((): number => {
    try { const v = Number(localStorage.getItem(CANVAS_KEY)); if (v > 0) return v; } catch { /* no storage */ }
    return typeof window === 'undefined' ? 360 : Math.round(window.innerHeight * 0.38);
  })());
  return {
    get canvasH() { return canvasH; },
    set canvasH(v: number) { canvasH = Math.max(120, Math.round(v)); },
    remember() { try { localStorage.setItem(CANVAS_KEY, String(canvasH)); } catch { /* no storage */ } },
  };
}
export const deskLayout = createDeskLayout();
export const openDesk = (): void => { deskUi.open = true; };
