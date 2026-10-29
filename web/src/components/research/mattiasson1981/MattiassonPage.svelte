<script lang="ts">
  import { onMount } from 'svelte';
  import { publicI18n } from '../../../lib/i18n/store.svelte';
  import ExperimentShell from '../shell/ExperimentShell.svelte';
  import PaneTabs from '../shell/PaneTabs.svelte';
  import { Playback } from '../shell/playback.svelte';
  import ScalarFieldViewer from '../../viewport/ScalarFieldViewer.svelte';
  import Curves from './Curves.svelte';
  import { cases, nativeOnly, runCases, viewerData, issue, tolerance, type Result } from './cases';
  import { editorURL } from './editor';
  import { strings } from './i18n';
  import { rs } from '../i18n';

  const playback = new Playback(350);
  let rows = $state<Result[]>([]);
  let busy = $state(false);
  let failure = $state('');
  let cancelled = false;
  let tab = $state('viewer');
  let pane: HTMLDivElement;
  let moment = $state(false);
  const forceCases = cases.filter(c => c.kind === 'force');
  const s = $derived(strings[publicI18n.locale as keyof typeof strings] ?? strings.en);
  const current = $derived(moment ? cases[cases.length - 1] : forceCases[playback.shown]);
  const result = $derived(rows.find(r => r.id === current.id));
  const view = $derived(viewerData(current, result));
  const checks = cases.filter(c => c.reference);
  const passed = $derived(rows.filter(r => checks.some(c => c.id === r.id) && r.pass).length);
  const fmt = (v: number | null | undefined) => v == null ? '—' : new Intl.NumberFormat(publicI18n.locale, { maximumSignificantDigits: 7 }).format(v);
  async function run() {
    if (busy) return;
    busy = true; failure = ''; rows = []; cancelled = false;
    try {
      await runCases(r => { rows = [...rows, r]; }, () => cancelled);
      if (cancelled) failure = s.interrupted;
      else {
        playback.setCount(forceCases.length);
        playback.startAutoplay({ reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches, narrow: matchMedia('(max-width: 700px)').matches });
      }
    } catch (e) { failure = String(e); }
    finally { busy = false; }
  }
  function stop() { cancelled = true; playback.pause(); }
  function open() { playback.interact(); window.open(editorURL(current, new URL(`${import.meta.env.BASE_URL}app`, location.origin).href), '_blank', 'noopener'); }
  onMount(() => {
    const interaction = () => playback.interact();
    window.addEventListener('pointerdown', interaction, true);
    window.addEventListener('keydown', interaction, true);
    window.addEventListener('wheel', interaction, { capture: true, passive: true });
    let inView = true;
    const observer = new IntersectionObserver(entries => { inView = entries[0].isIntersecting; playback.setVisible(inView && !document.hidden); });
    observer.observe(pane);
    const visibility = () => playback.setVisible(inView && !document.hidden);
    document.addEventListener('visibilitychange', visibility);
    void run();
    return () => { cancelled = true; playback.dispose(); observer.disconnect(); document.removeEventListener('visibilitychange', visibility); window.removeEventListener('pointerdown', interaction, true); window.removeEventListener('keydown', interaction, true); window.removeEventListener('wheel', interaction, true); };
  });
</script>

<ExperimentShell paperId="mattiasson-elastica" name="Mattiasson (1981): cantilever elastica"
  lead="A cantilever under increasing tip force, traced by Stabileo’s production corotational solver. Reference transcriptions are provisional; failures remain visible."
  sourcePath="engine/tests/benchmarks"
  actions={{ goToStep: i => playback.seek(i), setParams: () => {}, showTab: t => { tab = t; } }}
  verification={{ reference: { kind: 'none', why: 'Production engine only; no independently reproduced reference solver.' }, published: { kind: 'none', why: 'The numerical transcription has not been checked against the paper text. Status: proposed.' } }}>
  {#snippet article()}
    <h2>Model and load path</h2>
    <p>A fixed-root cantilever has L = 2 m, 16 frame elements, E = 200000 MPa, A = 0.01 m² and I = 10⁻⁴ m⁴. The engine converts E to kN/m². Downward tip force P is parametrized by PL²/EI; u/L = −ux/L is shortening and v/L = −uz/L is downward displacement. The viewer uses actual nodal displacements at scale 1.</p>
    <p>Twenty independent incremental solves trace force parameters 0.5 through 10. Each uses the production <code>solveCorotational2D</code> wrapper with 50 iterations, residual tolerance 10⁻⁶ and 10 increments. Playback loops through these live results and stops on interaction. It is not an exact reference shape or an interpolated solver path.</p>
    <h2>References and accuracy</h2>
    <p>The four reference pairs are copied unchanged from <a href="https://github.com/lambdaclass/stabileo/blob/main/engine/tests/BENCHMARK_REFERENCES.md#mattiasson-elastica--cantilever-large-deflection">BENCHMARK_REFERENCES.md</a>, which attributes them to Mattiasson’s Table 1 and explicitly says they need verification. Neither the table/page locator nor these numbers have been checked against the paper text. They must not be treated as verified exact elliptic-integral values.</p>
    <p>The exploratory accuracy target is relative error strictly below {fmt(tolerance * 100)}% for both signed components. This new page target is separate from every existing native assertion, tolerance and regression bound. Intermediate steps have no accuracy reference. Nonconvergence or incomplete loading is a failure, never an accuracy pass.</p>
    <h2>Tip moment</h2>
    <p>The separate moment case uses M = EI/L = 10000 kN·m, matching the native end-moment test exactly. That test checks convergence and significant rotation. It does not establish full-circle rolling or elliptic-integral accuracy; the page makes neither claim. Under our downward-positive convention its upward displacement gives a negative v/L.</p>
    <h2>One case file and findings</h2>
    <p><a href="https://github.com/lambdaclass/stabileo/blob/main/engine/tests/benchmarks/mattiasson_elastica.json">The shared engine-owned JSON</a> supplies the native end-moment test, native comparison runner and live page. Research CI compares fresh native and WASM results, including failures. No research solver is used.</p>
    <p><a href={issue}>Findings ledger and unresolved provenance</a>: disagreement may involve transcription, model assumptions or solver accuracy. Solver-owner triage and the original numerical table are needed before attributing a defect. No runtime issue identifier is invented.</p>
    <h2>{s.native}</h2>
    <ul>{#each nativeOnly as c}<li><code>{c.nativeTest}</code>: {c.reason}</li>{/each}</ul>
    <p>The corotational API is available in production WASM. The native-only designation above applies to regression assertions, not to an unavailable solve.</p>
  {/snippet}
  {#snippet app()}
    <div class="benchmark-pane" bind:this={pane}>
      <p role="status" data-testid="mattiasson-summary">{#if busy}{s.running}{:else}{s.summary}: {passed}/{checks.length}{/if}</p>
      <p><strong>{rs().disclaimer}</strong></p>
      <p>{s.warning}</p>
      <p>{s.complete}: {rows.filter(r => !r.failure).length}/{cases.length} · {s.fail}: {rows.filter(r => r.failure).length}</p>
      <p>{busy ? s.live : s.cached}</p>
      <button onclick={() => { playback.interact(); void run(); }} disabled={busy}>{s.run}</button>
      <button onclick={stop}>{s.stop}</button>
      <button onclick={() => { moment = false; playback.play(); }} disabled={busy || rows.length !== cases.length}>{s.play}</button>
      <button onclick={() => { moment = false; playback.seek(0); playback.play(); }} disabled={busy || rows.length !== cases.length}>{s.restart}</button>
      {#if failure}<p role="alert">{failure}</p>{/if}
      <PaneTabs tabs={[{ id: 'viewer', label: s.viewer }, { id: 'results', label: s.results }]} bind:active={tab} />
      <div id="rs-panel-viewer" role="tabpanel" aria-labelledby="rs-tab-viewer" hidden={tab !== 'viewer'}>
        <label><input type="checkbox" bind:checked={moment} /> {s.moment}</label>
        <label>{s.select}: {fmt(current.parameter)} <input aria-label={s.select} type="range" min="0" max={forceCases.length - 1} value={playback.shown} disabled={moment} oninput={e => playback.seek(Number(e.currentTarget.value))} /></label>
        <p data-testid="mattiasson-step">{moment ? s.moment : s.force}: {current.id}</p>
        <ScalarFieldViewer {...view} cellSize={2} deformScale={1} showMesh={true} title={moment ? s.moment : s.force} unit="m" height="23rem" onOpenEditor={open} />
        {#if result?.failure}<p role="alert">{result.failure}</p>{:else if !result}<p>{s.pending}</p>{/if}
        <button onclick={open}>{s.open}</button><p>{s.handoff}</p>
      </div>
      <div id="rs-panel-results" role="tabpanel" aria-labelledby="rs-tab-results" hidden={tab !== 'results'}>
        <Curves {rows} referenceLabel={s.reference} locale={publicI18n.locale} />
        <div class="table-scroll" id="checks"><table><thead><tr><th>PL²/EI</th><th>{s.reference} u/L, v/L</th><th>Stabileo u/L, v/L</th><th>{s.error} %</th><th></th></tr></thead><tbody>
          {#each cases as c}
            {@const r = rows.find(r => r.id === c.id)}
            <tr data-testid={c.id}><th>{c.kind === 'moment' ? s.moment : fmt(c.parameter)}</th><td>{c.reference ? c.reference.map(fmt).join(', ') : '—'}</td><td>{fmt(r?.u)}, {fmt(r?.v)}</td><td>{r?.errors ? r.errors.map(e => fmt(e * 100)).join(', ') : '—'}</td><td>{#if !r}{s.pending}{:else if r.pass === true}{s.pass}{:else if r.pass === false}<a href={issue}>{s.fail}</a>{#if r.failure}<p>{r.failure}</p>{/if}{:else}{s.noReference}{/if}</td></tr>
          {/each}
        </tbody></table></div>
      </div>
    </div>
  {/snippet}
</ExperimentShell>
<style>
  .benchmark-pane { height: 100%; overflow: auto; }
  button, input { font: inherit; } button { padding: .45rem; }
  label { display: block; margin: .65rem 0; } p { font-size: .875rem; }
  .table-scroll { overflow-x: auto; } table { border-collapse: collapse; width: 100%; font-size: .8rem; }
  th, td { text-align: left; padding: .5rem; border-bottom: 1px solid var(--hair); } a { color: inherit; }
</style>
