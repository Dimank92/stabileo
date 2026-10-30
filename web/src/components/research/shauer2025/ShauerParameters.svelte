<script lang="ts">
  import ParamRow from '../shell/ParamRow.svelte';
  import NumberField from './NumberField.svelte';
  import { solverStepCount } from './steps';
  import { num } from '../shell/format';
  import { MESHES, LIMITS, inRange, type MeshKey } from './parameters';
  import type { SimulationController, Frame } from './simulation.svelte';
  import type { ResearchStrings } from '../i18n';

  interface Props {
    tab: string;
    strings: ResearchStrings;
    e: number;
    nu: number;
    gc: number;
    l0: number;
    u: number;
    dt: number;
    meshKey: MeshKey;
    history: boolean;
    invalid: { e: boolean; nu: boolean; gc: boolean; l0: boolean; u: boolean };
    valid: boolean;
    isReference: boolean;
    sigmaC: number;
    hOverL: number;
    run: SimulationController['current'];
    sim: SimulationController;
    frames: Frame[];
    expectedSteps: number;
    onInteract?: () => void;
    onStart: (force: boolean) => void;
    onRestart: () => void;
    onReset: () => void;
    onBackToReplay: () => void;
  }

  let {
    tab,
    strings,
    e = $bindable(),
    nu = $bindable(),
    gc = $bindable(),
    l0 = $bindable(),
    u = $bindable(),
    dt = $bindable(),
    meshKey = $bindable(),
    history = $bindable(),
    invalid = $bindable(),
    valid,
    isReference,
    sigmaC,
    hOverL,
    run,
    sim,
    frames,
    expectedSteps,
    onInteract,
    onStart,
    onRestart,
    onReset,
    onBackToReplay,
  }: Props = $props();

  const fmt = (v: number, d = 4) => (Math.abs(v) >= 1e-3 || v === 0 ? num(v, { dp: d }) : num(v, { sig: 3 }));
</script>

<div
  class="rs-panel rs-params"
  id="rs-panel-parameters"
  role="tabpanel"
  aria-labelledby="rs-tab-parameters"
  hidden={tab !== 'parameters'}
  oninputcapture={() => onInteract?.()}
>
  <div class="rs-params-list">
    <ParamRow id="p-e" name={strings.youngModulus} unit="kN/mm²" hint="10–400" invalid={invalid.e || !inRange(e, LIMITS.e)}>
      {#snippet symbol()}<var>E</var>{/snippet}
      <NumberField id="p-e" label={strings.youngModulusLabel} min={LIMITS.e[0]} max={LIMITS.e[1]} bind:value={e} bind:invalid={invalid.e} />
    </ParamRow>
    <ParamRow id="p-nu" name={strings.poissonRatio} hint="0–0.45" invalid={invalid.nu || !inRange(nu, LIMITS.nu)}>
      {#snippet symbol()}<var>ν</var>{/snippet}
      <NumberField id="p-nu" label={strings.poissonRatioLabel} min={LIMITS.nu[0]} max={LIMITS.nu[1]} bind:value={nu} bind:invalid={invalid.nu} />
    </ParamRow>
    <ParamRow id="p-gc" name={strings.fractureToughness} unit="kN/mm" hint="0.0001–0.01" invalid={invalid.gc || !inRange(gc, LIMITS.gc)}>
      {#snippet symbol()}<var>G</var><sub>c</sub>{/snippet}
      <NumberField id="p-gc" label={strings.fractureToughnessLabel} min={LIMITS.gc[0]} max={LIMITS.gc[1]} bind:value={gc} bind:invalid={invalid.gc} />
    </ParamRow>
    <ParamRow id="p-l0" name={strings.lengthScale} unit="mm" hint="0.003–0.03" invalid={invalid.l0 || !inRange(l0, LIMITS.l0)}>
      {#snippet symbol()}<var>ℓ</var>{/snippet}
      <NumberField id="p-l0" label={strings.lengthScaleLabel} min={LIMITS.l0[0]} max={LIMITS.l0[1]} bind:value={l0} bind:invalid={invalid.l0} />
    </ParamRow>
    <ParamRow id="p-u" name={strings.topDisplacement} unit="mm" hint="0.004–0.02" invalid={invalid.u || !inRange(u, LIMITS.u)}>
      {#snippet symbol()}<var>u</var>{/snippet}
      <NumberField id="p-u" label={strings.topDisplacementLabel} min={LIMITS.u[0]} max={LIMITS.u[1]} bind:value={u} bind:invalid={invalid.u} />
    </ParamRow>
    <ParamRow id="p-dt" name={strings.loadStep}>
      {#snippet symbol()}Δ<var>t</var>{/snippet}
      <select id="p-dt" bind:value={dt}>
        {#each [0.005, 0.01, 0.02, 0.03, 0.06] as v (v)}
          <option value={v}>{v} ({solverStepCount(v, 0.8)} {strings.stepsParen}</option>
        {/each}
      </select>
    </ParamRow>
    <ParamRow id="p-mesh" name={strings.mesh}>
      <select id="p-mesh" bind:value={meshKey}>
        {#each Object.entries(MESHES) as [k, m] (k)}
          <option value={k}>{strings[m.label]}</option>
        {/each}
      </select>
    </ParamRow>
    <label class="rs-check">
      <input type="checkbox" bind:checked={history} /> {strings.historyField}
    </label>
  </div>
  <p class="rs-derived">
    {strings.planeStrainLoaded}{fmt(0.8 * u)} mm). σ<sub>c</sub> = {fmt(sigmaC, 3)} {strings.refinedRatio} {hOverL.toFixed(2)}{#if hOverL > 0.5}<strong class="warnt"> {strings.underResolved}</strong>{/if}.
  </p>
  <div class="rs-buttons">
    {#if run?.status === 'running'}
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => sim.stop()}>{strings.stop}</button>
      <button class="btn btn-ghost btn-sm" type="button" onclick={onRestart}>{strings.restart}</button>
    {:else}
      <button class="btn btn-primary btn-sm" type="button" disabled={!valid} onclick={() => onStart(false)}>{strings.run}</button>
      <button class="btn btn-ghost btn-sm" type="button" disabled={!valid} onclick={() => onStart(true)}>{strings.solveAgain}</button>
    {/if}
    {#if !isReference}
      <button class="btn btn-ghost btn-sm" type="button" onclick={onReset}>{strings.resetParameters}</button>
    {/if}
    {#if run}
      <button class="btn btn-ghost btn-sm" type="button" onclick={onBackToReplay}>{strings.backToReplay}</button>
    {/if}
  </div>
  <p class="rs-status" aria-live="polite">
    {#if sim.available === false}
      <span class="warnt">{strings.solverUnavailable}</span>
    {:else if run?.status === 'running'}
      {strings.solvingStep} {frames.length + 1} {strings.of} {expectedSteps}…
    {:else if run?.status === 'done'}
      {strings.done} {frames.length} {strings.steps}{run.seconds ? ` ${strings.inTime} ${run.seconds.toFixed(1)} s` : ''}{strings.completedRunsKept}
    {:else if run?.status === 'stopped'}
      {strings.stoppedAfter} {frames.length} {strings.stepsPeriod}
    {:else if run?.status === 'error'}
      <span class="warnt">{strings.solverRefused} {run.error}</span>
    {:else if isReference}
      {strings.runReuseNote}
    {:else}
      {strings.runBrowserNote}
    {/if}
  </p>
</div>

<style>
  .rs-panel {
    min-width: 0;
  }
  .rs-panel[hidden] {
    display: none;
  }
  .rs-params-list {
    display: grid;
    gap: 0.15rem;
  }
  .rs-params-list :global(select) {
    height: var(--rs-control-h);
    background: var(--st-surface-3);
    color: var(--fg);
    border: 1px solid var(--rs-control-border);
    border-radius: var(--st-radius, 3px);
    font: 0.9375rem var(--st-sans);
  }
  .rs-check input {
    accent-color: var(--st-accent);
    width: 1.125rem;
    height: 1.125rem;
  }
  .rs-check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2rem;
    font-size: var(--rs-ui);
    justify-content: flex-start;
  }
  .rs-derived,
  .rs-status {
    font-size: 0.82rem;
    color: var(--fg-2);
    margin: 0.4rem 0;
  }
  .warnt {
    color: #d9a441;
  }
  .rs-buttons {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  @media (min-width: 70rem) {
    .rs-panel {
      flex: 1 1 auto;
      min-height: 0;
      overflow-y: auto;
      scrollbar-width: auto;
      scrollbar-gutter: stable;
      scrollbar-color: var(--fg-2) var(--hair);
    }
    .rs-panel::-webkit-scrollbar { width: 10px; }
    .rs-panel::-webkit-scrollbar-track { background: var(--hair); }
    .rs-panel::-webkit-scrollbar-thumb { background: var(--fg-2); border-radius: 5px; }
  }
</style>
