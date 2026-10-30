<script lang="ts">
  import LineChart from './LineChart.svelte';
  import IterationsChart from './IterationsChart.svelte';
  import type { Frame } from './simulation.svelte';
  import type { Series } from './protocol';
  import type { compareReference } from './comparison';
  import type { Source } from '../shell/playback.svelte';
  import type { ResearchStrings } from '../i18n';

  type Ref = { step: number; pseudotime: number; reaction: number; iterations: number; damageMax: number };

  interface Props {
    tab: string;
    strings: ResearchStrings;
    forceH?: number;
    convH?: number;
    forceSeries: Series[];
    forceLabels: Array<{ x: number; y: number; anchor: 'start' | 'middle' | 'end'; text: string }>;
    residualSeries: Series[];
    showsReference: boolean;
    frame: Frame | null;
    comparison: ReturnType<typeof compareReference> | null;
    source: Source;
    refPeak: Ref;
    reference: Ref[];
    frames: Frame[];
    energySeries: Series[];
    energyLabels: Array<{ x: number; y: number; anchor: 'start' | 'middle' | 'end'; text: string }>;
  }

  let {
    tab,
    strings,
    forceH = $bindable(400),
    convH = $bindable(600),
    forceSeries,
    forceLabels,
    residualSeries,
    showsReference,
    frame,
    comparison,
    source,
    refPeak,
    reference,
    frames,
    energySeries,
    energyLabels,
  }: Props = $props();
</script>

<div class="rs-panel" id="rs-panel-force" role="tabpanel" aria-labelledby="rs-tab-force" hidden={tab !== 'force'} bind:clientHeight={forceH}>
  <figure>
    <figcaption>{strings.reactionCaption}</figcaption>
    <LineChart
      series={forceSeries}
      xLabel={strings.topDisplacementAxis}
      yLabel="F (kN)"
      marker={frame ? frame.step.displacement : null}
      markerLabel={frame ? `${strings.step} ${frame.step.step}` : ''}
      labels={forceLabels}
      height={Math.max(170, forceH - 190)}
    />
    {#if residualSeries.length}
      <LineChart
        series={residualSeries}
        xLabel={strings.topDisplacementAxis}
        yLabel="|F − F_ref| (kN)"
        log
        rules={[{ y: 1e-5, text: strings.forceComparisonTolerance }]}
        marker={frame ? frame.step.displacement : null}
        height={110}
      />
    {/if}
    {#if !showsReference}<p class="rs-note">{strings.referenceCurveNote}</p>{/if}
  </figure>
</div>

<div class="rs-panel" id="rs-panel-convergence" role="tabpanel" aria-labelledby="rs-tab-convergence" hidden={tab !== 'convergence'} bind:clientHeight={convH}>
  {#if comparison}
    <div class="rs-compare" id="rs-compare-detail">
      <h3>{source === 'replay' ? strings.thisReplay : strings.thisRun} {strings.againstReferenceCode}</h3>
      <ul>
        <li>{strings.peakForce} {comparison.peak.step.reaction.toFixed(6)} {strings.knAtStep} {comparison.peak.step.step}{strings.referenceSemicolon} {refPeak.reaction.toFixed(6)} {strings.knAtStep} {refPeak.step}.</li>
        <li>{strings.relativeForceDifference} {comparison.worstPre.toExponential(1)} {strings.relativeTestTolerance}</li>
        <li>{strings.absoluteForceDifference} {comparison.worstAll.toExponential(1)} {strings.absoluteTestTolerance}</li>
        <li>{strings.identicalIterations} {comparison.sameIterations} {strings.of} {reference.length} {strings.stepsPeriod}</li>
      </ul>
    </div>
  {/if}
  {#if frames.length}
    <figure>
      <figcaption>{strings.iterationsCaption}{showsReference ? strings.referenceDots : ''}</figcaption>
      <IterationsChart steps={frames.map((f) => f.step)} reference={showsReference ? reference.map((r) => r.iterations) : null} height={130} />
    </figure>
    <figure>
      <figcaption>{strings.energies}</figcaption>
      <LineChart series={energySeries} xLabel={strings.topDisplacementAxis} yLabel={strings.energyAxis} marker={frame ? frame.step.displacement : null} labels={energyLabels} height={Math.max(150, convH - (comparison ? 420 : 260))} />
    </figure>
  {/if}
</div>

<style>
  .rs-panel {
    min-width: 0;
  }
  .rs-panel[hidden] {
    display: none;
  }
  figure {
    margin: 0;
  }
  figcaption {
    color: var(--fg);
    font-size: 0.92rem;
    margin-bottom: 0.4rem;
  }
  .rs-note {
    font-size: 0.82rem;
    color: var(--fg-2);
    margin: 0.4rem 0;
  }
  .rs-compare {
    border: 1px solid var(--ok);
    border-radius: 3px;
    padding: 0.6rem 0.9rem;
    font-size: 0.88rem;
  }
  .rs-compare h3 {
    margin: 0 0 0.4rem;
    font-size: 0.95rem;
  }
  .rs-compare ul {
    margin: 0;
    padding-left: 1.1rem;
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
