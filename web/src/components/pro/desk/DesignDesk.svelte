<script lang="ts">
  import RecheckNotice from '../design/RecheckNotice.svelte';
  /**
   * The design desk: one place and one «Design» for the whole structure. The rail on the left
   * (the ways of working, then the navigator: levels, filters, the members, the way through the
   * failing ones), the canvas (the level's plan by status, the piece editors, the centres of mass
   * and rigidity, the final report, the geometry check) and the piece sheet on the right (the
   * member, its decision and its checks).
   *
   * ── Docked: the body of the Design stage ───────────────────────────
   *
   * On a desktop the desk is no longer a sheet over the whole window. It is mounted inside the
   * PRO body row (`App.svelte`) with `docked`, and its parts take the cells of that row's grid
   * around the model viewer, which stays where it is in the page: the rail on the left, the
   * viewer in the centre with the canvas under it, the piece sheet on the right. The viewer is
   * never remounted, so its camera is the model's camera on the way in and on the way out, and
   * a pick in it is a pick in design (`deskUi.selected` reads the model selection). On a phone
   * there is no room for the viewer beside it and the desk stays a full-screen sheet.
   */
  import { modelStore, resultsStore, uiStore } from '../../../lib/store';
  import { designHome } from '../../../lib/pro/design-home';
  import { DESIGN_FAMILIES } from '../../../lib/engine/design/design-families';
  import { t, tp } from '../../../lib/i18n';
  import { designRunStore } from '../../../lib/store/design-run.svelte';
  import { deskSlabColumnJoint, designEverything, finalReport, modelLevelCentres } from '../../../lib/store/design-desk';
  import { deskUi, deskLayout, DESK_GROUPS, groupOf, type DeskTab } from '../../../lib/store/design-desk-ui.svelte';
  import { detailingStore } from '../../../lib/store/detailing.svelte';
  import { rebarWorkspace } from '../../../lib/store/rebar-workspace.svelte';
  import { jointChecks } from '../../../lib/store/joint-checks';
  import DocumentsSection from '../design/DocumentsSection.svelte';
  import { deskRows } from './desk-rows';
  import { untrack } from 'svelte';
  import { verificationStore } from '../../../lib/store/verification.svelte';
  import { runSteelCheck } from '../../../lib/store/steel-check';
  import DeskNavigator from './DeskNavigator.svelte';
  import DeskPlan from './DeskPlan.svelte';
  import DeskInspector from './DeskInspector.svelte';
  import DeskLevels from './DeskLevels.svelte';
  import DeskReportView from './DeskReportView.svelte';
  import DeskGeometry from './DeskGeometry.svelte';
  import FrameEditor from './frame/FrameEditor.svelte';
  import ColumnEditor from './column/ColumnEditor.svelte';
  import WallEditor from './wall/WallEditor.svelte';
  import SlabEditor from './slab/SlabEditor.svelte';
  import JoistEditor from './joist/JoistEditor.svelte';
  import FootingEditor from './footing/FootingEditor.svelte';
  import SeismicEditor from './seismic/SeismicEditor.svelte';
  import JointDeskView from './joint/JointDeskView.svelte';
  import MaterialsEditor from './materials/MaterialsEditor.svelte';
  import SpecialFloorsEditor from './special/SpecialFloorsEditor.svelte';
  import StairsEditor from './stairs/StairsEditor.svelte';
  import PostTensionEditor from './pt/PostTensionEditor.svelte';
  import ElementsEditor from './elements/ElementsEditor.svelte';
  import FireEditor from './fire/FireEditor.svelte';
  import ForeignCodesEditor from './codes/ForeignCodesEditor.svelte';
  import BudgetEditor from './budget/BudgetEditor.svelte';
  import CollabEditor from './collab/CollabEditor.svelte';
  import ArchEditor from './arch/ArchEditor.svelte';
  import SiteEditor from './site/SiteEditor.svelte';
  import { materialCheckState, runMaterialChecks } from '../../../lib/store/material-check.svelte';
  import { equalizeAllFrames } from '../../../lib/store/frame-equalize';
  import { designDrafts } from '../../../lib/store/design-drafts.svelte';
  import { canOpenRebar3D } from '../../../lib/store/rebar-open';
  import { rcOpenRebar3D } from '../../../lib/flow/rc-commands';
  import { detailingAuthor } from '../../../lib/store/detailing-author.svelte';
  import { openClassic, isDesignEmpty } from '../../../lib/pro/design-navigation';
  import DeskPrepare from './DeskPrepare.svelte';

  /** Mounted in the PRO body row beside the viewer (desktop), or as a full-screen sheet (phone). */
  let { docked = false }: { docked?: boolean } = $props();

  const rows = $derived.by(() => { void modelStore.elements; void modelStore.designDecisions; void designRunStore.running; return deskOpen() ? deskRows() : []; });
  const deskOpen = () => deskUi.open;
  const centres = $derived(deskUi.open && (deskUi.tab === 'levels' || deskUi.tab === 'plan') ? modelLevelCentres() : []);
  const levels = $derived(deskUi.open && deskUi.tab === 'report' ? finalReport() : []);
  const solved = $derived(!!resultsStore.results3D);
  const group = $derived(groupOf(deskUi.tab));
  /** The editor for a member: a beam's frame, a column's stack. */
  const pieceTabOf = (id: number | null): DeskTab | null => {
    const k = id !== null ? verificationStore.contextFor(id)?.elementType : undefined;
    return k === 'beam' ? 'frame' : k === 'column' ? 'column' : null;
  };
  let lastPiece = $state<DeskTab>('frame');
  $effect(() => { if (group === 'piece') lastPiece = deskUi.tab; });
  // In the piece view, selecting a beam or a column opens its editor: the tool follows the selection.
  $effect(() => {
    const id = deskUi.selected;
    untrack(() => {
      if (deskUi.tab !== 'frame' && deskUi.tab !== 'column') return;
      const k = pieceTabOf(id);
      if (k && k !== deskUi.tab) deskUi.tab = k;
    });
  });
  function openPiece() { deskUi.tab = pieceTabOf(deskUi.selected) ?? lastPiece; }
  /** Everything the report lists: what fails, what could not be checked, what is out of date, and the joints. */
  const pendingCount = $derived.by(() => {
    if (!deskUi.open) return 0;
    void verificationStore.contexts; void designRunStore.running;
    const lv = finalReport();
    const members = lv.reduce((n, l) => n + l.failing.length + l.unchecked.length + l.stale.length, 0);
    const joints = jointChecks().filter((j) => j.check.status === 'FAIL' || j.check.status === 'UNSUPPORTED').length;
    return members + joints;
  });
  let menu = $state<'specialties' | 'project' | null>(null);
  function pick(k: DeskTab) { deskUi.tab = k; menu = null; }
  /** Nothing to design yet: the canvas offers the two ways in instead of an empty plan. */
  const empty = $derived(deskUi.open && isDesignEmpty(rows.length));
  const can3d = $derived(deskUi.open && canOpenRebar3D());
  let railNote = $state<string | null>(null);
  /** The 3-D reinforcement view, on top of design, on the member design is on. */
  function open3d() {
    railNote = null;
    const r = rcOpenRebar3D(detailingAuthor.resolve(t('detailing.doc.unnamedAuthor')), new Date().toISOString());
    if (!r.ok) railNote = t('detailing.doc.noCoordinated');
  }

  /** Drag the canvas's top edge, when docked: the viewer above takes what the canvas leaves. */
  function startSplit(e: PointerEvent) {
    const y0 = e.clientY, h0 = deskLayout.canvasH;
    const max = window.innerHeight - 220;
    const move = (ev: PointerEvent) => { deskLayout.canvasH = Math.min(max, h0 - (ev.clientY - y0)); };
    const up = () => {
      window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up);
      deskLayout.remember();
    };
    window.addEventListener('pointermove', move); window.addEventListener('pointerup', up);
  }
  const detailReady = $derived(detailingStore.readiness);
  /** Whether a floor's slab is joined to its columns at one node (`analysis-validity.ts`). */
  const joint = $derived(deskSlabColumnJoint());
  const detailBlockers = $derived(detailReady.prerequisites.map((p) => tp(p.key, { n: p.count, ids: p.elementIds.slice(0, 6).join(', ') })).join(' '));
  /** Detail what was designed and go to the documents, where the result is read. */
  function detail() { detailingStore.generate(); deskUi.tab = 'documents'; }

  let asking = $state(false);
  let edited = $state<'keep' | 'overwrite'>('keep');
  let steel = $state(true);
  /*
   * Messages kept as a key and its parameters, translated where they are shown: a text built
   * when the design ran stayed in that language after the app's changed.
   */
  let error = $state<{ key: string; params: Record<string, string | number> } | null>(null);
  let framesNote = $state<{ groups: number; n: number } | null>(null);
  const editedCount = $derived(designRunStore.manualOverrides.size);

  let onlySelection = $state(false);
  async function run() {
    asking = false; error = null;
    await new Promise((r) => requestAnimationFrame(() => r(null)));
    if (onlySelection && uiStore.selectedElements.size > 0) {
      // The members selected in the model, alone: the classic screen's «design the selection».
      designRunStore.autoDesign([...uiStore.selectedElements], { edited });
      deskUi.report = { concrete: null, steel: null, skipped: { ...designRunStore.lastSkipped }, levels: finalReport() };
    } else deskUi.report = designEverything({ edited, steel });
    if (designRunStore.lastError) error = { key: designRunStore.lastError.key, params: designRunStore.lastError.params ?? {} };
    deskUi.tab = 'report';
  }
  /*
   * A project opened with its bars: once there is an analysis, the desk checks them again on its
   * own, the concrete through its demands and the steel through CIRSOC 301. Results are not saved
   * with the project; decisions are, and this is where they are read back.
   */
  $effect(() => {
    if (!deskUi.open || !solved) return;
    untrack(() => {
      const hasBars = [...modelStore.elements.values()].some((e) => e.reinforcement);
      if (hasBars && verificationStore.contexts.size === 0 && !designRunStore.running) designRunStore.computeDemands();
      if (verificationStore.steelMap.size === 0) runSteelCheck();
      if (materialCheckState.results.size === 0) runMaterialChecks();
    });
  });

  /*
   * The desk is modal: closing it gives focus back to what opened it (the Design stage, the desk
   * command), as the 3-D rebar workspace does, rather than leaving it on the page body.
   */
  let opener: HTMLElement | null = null;
  $effect(() => {
    if (deskUi.open) { opener = document.activeElement instanceof HTMLElement ? document.activeElement : null; return; }
    const o = opener; opener = null;
    if (o && o.isConnected) queueMicrotask(() => o.focus());
  });

  /*
   * Escape closes the topmost surface only. The 3-D workspace opened from the desk sits above it,
   * and its own Escape handler closes it — read in the capture phase, before that handler runs,
   * so the same key does not then find no workspace and close the desk as well.
   */
  let workspaceAtKeydown = false;
  $effect(() => {
    const note = (): void => { workspaceAtKeydown = rebarWorkspace.open; };
    window.addEventListener('keydown', note, true);
    return () => window.removeEventListener('keydown', note, true);
  });
  function onKey(e: KeyboardEvent) { if (deskUi.open && e.key === 'Escape' && !asking && !workspaceAtKeydown) deskUi.open = false; }
</script>

<svelte:window onkeydown={onKey} />

{#if deskUi.open}
  <div class="dd" class:docked role={docked ? 'region' : 'dialog'} aria-modal={docked ? undefined : 'true'} aria-label={t('desk.title')} data-testid="design-desk">
    <header class="dd-head">
      <h3>{t('desk.title')}</h3>
      <div class="dd-flow" role="group" aria-label={t('desk.flow')}>
        <button class="dd-primary" disabled={!solved || designRunStore.running} onclick={() => (asking = true)} data-testid="desk-design">{t('desk.design')}</button>
        <span aria-hidden="true">›</span>
        <button disabled={!detailReady.ready || detailingStore.generating || designRunStore.running} title={detailReady.ready ? '' : detailBlockers} onclick={detail} data-testid="desk-detail">{detailingStore.generating ? t('detailing.cmd.generating') : t('desk.detail')}</button>
        <span aria-hidden="true">›</span>
        <button class:on={deskUi.tab === 'documents'} onclick={() => (deskUi.tab = 'documents')} data-testid="desk-tab-documents">{t('desk.tab.documents')}</button>
      </div>
      <button disabled={designRunStore.running} onclick={() => { const r = equalizeAllFrames(); framesNote = { groups: r.groups, n: r.members }; }} title={t('desk.equalizeFramesHint')} data-testid="desk-equalize-frames">{t('desk.equalizeFrames')}</button>
      {#if framesNote}<span class="dd-progress" role="status" data-testid="desk-frames-note">{tp('desk.framesEqualized', framesNote)}</span>{/if}
      {#if designRunStore.running}<span class="dd-progress" data-testid="desk-progress">{tp('desk.progress', { done: designRunStore.progress?.done ?? 0, total: designRunStore.progress?.total ?? 0 })}</span>{/if}
      {#if !solved}<span class="dd-warn">{t('desk.needSolve')}</span><RecheckNotice />{/if}
      {#if error}<span class="dd-warn" role="alert" data-testid="desk-error">{tp(error.key, error.params)}</span>{/if}
      {#if joint.verdict === 'pointJoint'}<span class="dd-warn" role="status" data-testid="desk-slab-column-point-joint">{tp('slabColumnJoint.desk', { n: joint.pointJoints.length })}</span>{/if}
      <!-- Drafts kept by the editors (`design-drafts.svelte.ts`): said here until saved or discarded. -->
      {#if designDrafts.count > 0}
        <span class="dd-drafts" role="status" data-testid="desk-drafts">{tp('desk.drafts', { n: designDrafts.count })}
          {#each designDrafts.frames as d (d.key)}<button onclick={() => { deskUi.selected = d.members[0]!; deskUi.tab = 'frame'; }} data-testid="desk-draft-{d.key}">{d.label}</button>{/each}
        </span>
      {/if}
      <!-- What the desk does not hold yet (the steel optimiser and assumptions, the member list with
           its detailing state) is in the classic screens; this is the way to them. -->
      <button class="dd-classic" onclick={() => openClassic(designHome(modelStore.elements.values(), modelStore.materials))} title={t('desk.classicHint')} data-testid="desk-classic">{t('desk.classic')}</button>
      <button class="dd-close" onclick={() => (deskUi.open = false)} aria-label={t('desk.close')} data-testid="desk-close">✕</button>
    </header>

    <!--
      The rail: preparing the model, then concrete and steel, the other codes, what is pending and
      the documents. An entry marked ↗ opens a classic screen the desk does not hold yet.
    -->
    <aside class="dd-rail">
      <nav class="dd-modes" aria-label={t('desk.views')}>
        <button class:on={group === 'prepare' || (empty && deskUi.tab === 'plan')} onclick={() => (deskUi.tab = 'prepare')} data-testid="desk-rail-prepare">{t('desk.rail.prepare')}</button>
        <p class="dd-sec">{t('desk.rail.concrete')}</p>
        <button class:on={deskUi.tab === 'plan' && !empty} onclick={() => (deskUi.tab = 'plan')} data-testid="desk-tab-plan">{t('desk.tab.plan')}</button>
        <button class:on={deskUi.tab === 'levels'} onclick={() => (deskUi.tab = 'levels')} data-testid="desk-tab-levels">{t('desk.tab.levels')}</button>
        <button class:on={group === 'piece'} onclick={openPiece} data-testid="desk-mode-piece">{t('desk.mode.piece')}</button>
        {#if group === 'piece'}
          <div class="dd-sub" role="group" aria-label={t('desk.mode.piece')}>
            {#each DESK_GROUPS.piece as k (k)}<button class:on={deskUi.tab === k} onclick={() => (deskUi.tab = k)} data-testid="desk-tab-{k}">{t(`desk.tab.${k}`)}</button>{/each}
          </div>
        {/if}
        <button onclick={() => openClassic('design')} title={t('desk.rail.classicHint')} data-testid="desk-rail-list">{t('desk.rail.list')} ↗</button>
        <button disabled={!can3d} onclick={open3d} title={can3d ? t('desk.rail.rebar3dHint') : t('detailing.doc.noCoordinated')} data-testid="desk-rail-3d">{t('desk.rail.rebar3d')}</button>
        <p class="dd-sec">{t('desk.rail.steel')}</p>
        <button onclick={() => openClassic('steel')} title={t('desk.rail.classicHint')} data-testid="desk-rail-steel">{t('desk.rail.profiles')} ↗</button>
        <button onclick={() => openClassic('connections')} title={t('desk.rail.classicHint')} data-testid="desk-rail-connections">{t('desk.rail.connections')} ↗</button>
        <p class="dd-sec" aria-hidden="true"></p>
        <button class:on={deskUi.tab === 'codes'} onclick={() => (deskUi.tab = 'codes')} data-testid="desk-rail-codes">{t('desk.rail.otherCodes')}</button>
        <button class:on={group === 'pending'} onclick={() => (deskUi.tab = 'report')} data-testid="desk-mode-pending">{tp('desk.mode.pending', { n: pendingCount })}</button>
        {#if group === 'pending'}
          <div class="dd-sub" role="group" aria-label={tp('desk.mode.pending', { n: pendingCount })}>
            {#each DESK_GROUPS.pending as k (k)}<button class:on={deskUi.tab === k} onclick={() => (deskUi.tab = k)} data-testid="desk-tab-{k}">{t(`desk.tab.${k}`)}</button>{/each}
          </div>
        {/if}
        <button class:on={deskUi.tab === 'documents'} onclick={() => (deskUi.tab = 'documents')} data-testid="desk-rail-documents">{t('desk.tab.documents')}</button>
        <div class="dd-menus">
          {#each ['specialties', 'project'] as const as g (g)}
            <div class="dd-menu">
              <button class:on={group === g} aria-expanded={menu === g} onclick={() => (menu = menu === g ? null : g)} data-testid="desk-menu-{g}">{t(`desk.menu.${g}`)} ▾</button>
              {#if menu === g}
                <div class="dd-menu-list" role="menu">
                  {#each DESK_GROUPS[g] as k (k)}<button role="menuitem" class:on={deskUi.tab === k} onclick={() => pick(k)} data-testid="desk-tab-{k}">{t(`desk.tab.${k}`)}</button>{/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
        {#if railNote}<p class="dd-rail-note" role="status">{railNote}</p>{/if}
      </nav>
      <DeskNavigator {rows} />
    </aside>

    <main class="dd-canvas" data-testid="desk-canvas">
      {#if docked}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="dd-split" role="separator" aria-orientation="horizontal" aria-label={t('desk.split')} onpointerdown={startSplit} data-testid="desk-split"></div>
      {/if}
      <div class="dd-canvas-body">
        {#if deskUi.tab === 'prepare' || (empty && deskUi.tab === 'plan')}<DeskPrepare {empty} />
        {:else if deskUi.tab === 'plan'}<DeskPlan {rows} {centres} />
        {:else if deskUi.tab === 'column'}<ColumnEditor memberId={deskUi.selected} />
        {:else if deskUi.tab === 'wall'}<WallEditor />
        {:else if deskUi.tab === 'slab'}<SlabEditor />
        {:else if deskUi.tab === 'joist'}<JoistEditor />
        {:else if deskUi.tab === 'footing'}<FootingEditor />
        {:else if deskUi.tab === 'seismic'}<SeismicEditor />
        {:else if deskUi.tab === 'joint'}<JointDeskView />
        {:else if deskUi.tab === 'materials'}<MaterialsEditor />
        {:else if deskUi.tab === 'special'}<SpecialFloorsEditor />
        {:else if deskUi.tab === 'stairs'}<StairsEditor />
        {:else if deskUi.tab === 'pt'}<PostTensionEditor />
        {:else if deskUi.tab === 'elements'}<ElementsEditor />
        {:else if deskUi.tab === 'fire'}<FireEditor />
        {:else if deskUi.tab === 'codes'}<ForeignCodesEditor />
        {:else if deskUi.tab === 'budget'}<BudgetEditor />
        {:else if deskUi.tab === 'collab'}<CollabEditor />
        {:else if deskUi.tab === 'arch'}<ArchEditor />
        {:else if deskUi.tab === 'site'}<SiteEditor />
        {:else if deskUi.tab === 'frame'}{#if deskUi.selected !== null}<FrameEditor memberId={deskUi.selected} />{:else}<p class="dd-hint">{t('frameEd.pick')}</p>{/if}
        {:else if deskUi.tab === 'levels'}<DeskLevels {centres} />
        {:else if deskUi.tab === 'report'}<DeskReportView {levels} />
        {:else if deskUi.tab === 'documents'}<div class="dd-docs"><DocumentsSection recheck={false} /></div>
        {:else}<DeskGeometry />{/if}
      </div>
    </main>
    <div class="dd-sheet"><DeskInspector {rows} /></div>

    {#if asking}
      <div class="dd-ask" role="dialog" aria-label={t('desk.design')} data-testid="desk-run-options">
        <h4>{t('desk.design')}</h4>
        <p>{t('desk.designHint')}</p>
        {#if editedCount > 0}
          <fieldset>
            <legend>{tp('desk.editedQuestion', { n: editedCount })}</legend>
            <label><input type="radio" bind:group={edited} value="keep" data-testid="desk-keep-edited" /> {t('desk.keepEdited')}</label>
            <label><input type="radio" bind:group={edited} value="overwrite" data-testid="desk-overwrite-edited" /> {t('desk.overwriteEdited')}</label>
          </fieldset>
        {/if}
        <!-- What the classic screen's three design buttons chose: the families, or only what is selected. -->
        <fieldset data-testid="desk-run-families">
          <legend>{t('desk.scope')}</legend>
          {#each DESIGN_FAMILIES as f (f)}
            <label><input type="checkbox" checked={designRunStore.familySelection.includes(f)} disabled={onlySelection}
              onchange={(e) => designRunStore.setFamilySelection(e.currentTarget.checked ? [...designRunStore.familySelection, f] : designRunStore.familySelection.filter((x) => x !== f))}
              data-testid="desk-run-family-{f}" /> {t(`design.families.${f}`)}</label>
          {/each}
          <label><input type="checkbox" bind:checked={onlySelection} disabled={uiStore.selectedElements.size === 0} data-testid="desk-run-selection" /> {tp('desk.onlySelection', { n: uiStore.selectedElements.size })}</label>
        </fieldset>
        <label><input type="checkbox" bind:checked={steel} /> {t('desk.withSteel')}</label>
        <div class="dd-ask-actions">
          <button onclick={() => (asking = false)}>{t('desk.cancel')}</button>
          <button class="dd-primary" onclick={run} data-testid="desk-run">{t('desk.run')}</button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  /*
   * As a sheet (phone): the whole window, head over rail, canvas and piece sheet. Docked
   * (desktop): the box disappears (`display: contents`) and its parts take the cells the PRO body
   * row names for them around the viewer (`App.svelte`, `.desk-docked`).
   */
  .dd { position: fixed; inset: 0; z-index: 900; background: var(--st-bg, var(--st-surface)); color: var(--st-text);
    display: grid; grid-template-columns: auto minmax(0, 1fr) auto; grid-template-rows: auto minmax(0, 1fr);
    grid-template-areas: 'head head head' 'rail canvas sheet'; }
  .dd.docked { display: contents; }
  .dd-head { grid-area: head; display: flex; align-items: center; gap: 10px; padding: 6px 12px; border-bottom: 1px solid var(--st-hair); flex-wrap: wrap; background: var(--st-surface); color: var(--st-text); }
  .dd-rail { grid-area: rail; display: flex; flex-direction: column; min-height: 0; overflow: hidden; border-right: 1px solid var(--st-hair); background: var(--st-surface); color: var(--st-text); }
  .dd-rail > :global(.dn) { flex: 1; min-height: 0; border-right: none; border-top: 1px solid var(--st-hair); }
  .dd-canvas { grid-area: canvas; min-width: 0; min-height: 0; display: flex; flex-direction: column; overflow: hidden; background: var(--st-bg, var(--st-surface)); color: var(--st-text); }
  .dd.docked .dd-canvas { border-top: 1px solid var(--st-hair-strong); }
  /* What is scrolled into view lands clear of the edges, not flush under the viewer's border. */
  .dd-canvas-body { flex: 1; min-height: 0; display: flex; flex-direction: column; overflow: auto; scroll-padding: 24px; }
  .dd-split { flex: none; height: 6px; cursor: row-resize; background: var(--st-surface-2, var(--st-surface)); border-bottom: 1px solid var(--st-hair); }
  .dd-split:hover { background: var(--st-surface-3); }
  .dd-sheet { grid-area: sheet; min-height: 0; display: flex; flex-direction: column; overflow: hidden; background: var(--st-surface); color: var(--st-text); }
  .dd-sheet > :global(.di) { flex: 1; min-height: 0; }
  .dd-head h3 { margin: 0; font-size: 0.9rem; }
  .dd-hint { padding: 12px; font-size: 0.74rem; color: var(--st-text-3); }
  .dd-head > button:not(.dd-primary):not(.dd-close) { font-size: 0.72rem; padding: 3px 10px; border-radius: 4px; border: 1px solid var(--st-hair-strong); background: var(--st-surface); color: var(--st-text); cursor: pointer; }
  .dd-primary { font-size: 0.74rem; padding: 3px 12px; border-radius: 4px; border: 1px solid var(--st-accent); background: var(--st-accent); color: var(--st-on-accent, #fff); cursor: pointer; }
  .dd-primary:disabled { opacity: 0.5; cursor: default; }
  .dd-progress, .dd-warn { font-size: 0.7rem; color: var(--st-text-2); }
  .dd-warn { color: var(--st-warn); }
  .dd-drafts { display: inline-flex; flex-wrap: wrap; align-items: center; gap: 4px; font-size: 0.7rem; color: var(--st-text); border-left: 2px solid var(--st-warn); padding-left: 6px; }
  .dd-drafts button { font-size: 0.68rem; padding: 1px 6px; border-radius: 4px; border: 1px solid var(--st-hair-strong); background: var(--st-surface); color: var(--st-text); cursor: pointer; }
  .dd-flow { display: flex; align-items: center; gap: 6px; color: var(--st-text-3); }
  .dd-flow button:not(.dd-primary) { font-size: 0.72rem; padding: 3px 10px; border-radius: 4px; border: 1px solid var(--st-hair-strong); background: var(--st-surface); color: var(--st-text); cursor: pointer; }
  .dd-flow button:disabled { opacity: 0.5; cursor: default; }
  .dd-flow .on { border-color: var(--st-accent); }
  /* The ways of working, one per line, the specialties and the project as menus at the end. */
  .dd-modes { display: flex; flex-direction: column; gap: 1px; padding: 6px 8px; width: 17rem; box-sizing: border-box; overflow: auto; max-height: 55%; flex: none; }
  .dd-sub { display: flex; flex-wrap: wrap; gap: 2px; padding: 2px 0 4px 10px; }
  .dd-modes button { text-align: left; white-space: nowrap; font-size: 0.72rem; padding: 2px 8px; border: 1px solid transparent; background: none; color: var(--st-text-2); border-radius: 4px; cursor: pointer; }
  .dd-modes button:hover:not(:disabled) { background: var(--st-surface-3); color: var(--st-text); }
  .dd-modes button:disabled { opacity: 0.45; cursor: default; }
  .dd-sub button { font-size: 0.68rem; padding: 1px 6px; border-color: var(--st-hair); }
  .dd-modes .on { border-color: var(--st-accent); color: var(--st-text); }
  .dd-sec { margin: 6px 0 1px; font-family: var(--st-mono); font-size: 0.6rem; letter-spacing: 0.1em; text-transform: uppercase; color: var(--st-text-3); }
  .dd-sec:empty { margin: 3px 0 0; border-top: 1px solid var(--st-hair); }
  .dd-menus { display: flex; gap: 4px; margin-top: 6px; }
  .dd-menus .dd-menu > button { border-color: var(--st-hair-strong); }
  .dd-rail-note { margin: 4px 0 0; font-size: 0.68rem; color: var(--st-text-2); border-left: 2px solid var(--st-accent); padding-left: 6px; }
  .dd-menu { position: relative; }
  .dd-menu-list { position: absolute; left: 0; top: calc(100% + 2px); z-index: 5; display: flex; flex-direction: column; gap: 2px; padding: 4px; min-width: 12rem; background: var(--st-surface); border: 1px solid var(--st-hair-strong); border-radius: 4px; box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25); }
  .dd-menu-list button { text-align: left; }
  .dd-docs { padding: 10px; }
  .dd-classic { margin-left: auto; font-size: 0.7rem; padding: 2px 8px; border-radius: 4px; border: 1px solid var(--st-hair); background: none; color: var(--st-text-2); cursor: pointer; }
  .dd-close { background: none; border: none; color: var(--st-text-2); font-size: 1rem; cursor: pointer; }
  .dd-ask { position: fixed; z-index: 905; top: 110px; left: 50%; transform: translateX(-50%); width: min(28rem, 92vw); background: var(--st-surface); color: var(--st-text); border: 1px solid var(--st-hair-strong); border-radius: 6px; padding: 12px; box-shadow: 0 6px 24px rgba(0, 0, 0, 0.25); display: flex; flex-direction: column; gap: 8px; font-size: 0.74rem; }
  .dd-ask h4 { margin: 0; }
  .dd-ask p { margin: 0; color: var(--st-text-2); }
  .dd-ask fieldset { border: 1px solid var(--st-hair); border-radius: 4px; display: flex; flex-direction: column; gap: 4px; }
  .dd-ask-actions { display: flex; justify-content: flex-end; gap: 6px; }
  .dd-ask-actions button { font-size: 0.72rem; padding: 3px 10px; border-radius: 4px; border: 1px solid var(--st-hair-strong); background: var(--st-surface); color: var(--st-text); cursor: pointer; }
</style>
