<script lang="ts">
  import { onMount } from 'svelte';
  import ExperimentShell from '../shell/ExperimentShell.svelte';
  import StabileoViewer from '../shell/StabileoViewer.svelte';
  import { Playback, type Source } from '../shell/playback.svelte';
  import { num } from '../shell/format';
  import NumberField from '../shauer2025/NumberField.svelte';
  import LineChart from '../shauer2025/LineChart.svelte';
  import type { Series } from '../shauer2025/protocol';
  import { publicI18n } from '../../../lib/i18n/store.svelte';
  import { defaults, validConfig, validFrame, referenceMatches, mesh, decodeReplay, type Config, type Frame } from './model';
  import { text } from './i18n';
  import validation from '../../../../../research/optimization/andreassen-2011/VALIDATION.json';
  const s = $derived(text[publicI18n.locale as keyof typeof text] ?? text.en);
  const referenceChecks = validation.checks.filter(c => c.name.startsWith('ft'));
  let config = $state<Config>({ ...defaults });
  let active = $state<Config>({ ...defaults });
  let frames = $state<Frame[]>([]);
  let refs = $state<Record<number, Frame[]>>({});
  let source = $state<Source>('replay');
  let running = $state(false);
  let error = $state('');
  let tab = $state('density');
  let showMesh = $state(true);
  let invalid = $state([false, false, false, false, false]);
  let host: HTMLDivElement;
  let worker: Worker | null = null;
  let base = $state('');
  let interacted = false;
  let epoch = 0;
  const playback = new Playback();
  const frame = $derived(frames[playback.shown]);
  const geometry = $derived(mesh(active));
  const overlay = $derived(referenceMatches(active) ? refs[active.ft] ?? [] : []);
  const disabled = $derived(!validConfig(config) || invalid.some(Boolean));
  const fields: { key: 'nelx' | 'nely' | 'volfrac' | 'penal' | 'rmin'; min: number; max: number }[] = [
    { key: 'nelx', min: 2, max: 120 }, { key: 'nely', min: 2, max: 60 }, { key: 'volfrac', min: .05, max: .95 }, { key: 'penal', min: 1, max: 5 }, { key: 'rmin', min: 1, max: 10 },
  ];
  function series(key: 'compliance' | 'volume'): Series[] {
    return [{ name: s.reference, color: '#aaaaaa', points: overlay.map(f => [f.iteration, f[key]]), dashed: true },
      { name: 'Stabileo', color: '#db9d45', points: frames.map(f => [f.iteration, f[key]]) }];
  }
  function interact(event: Event) {
    interacted = true;
    if (!(event.target instanceof Element && event.target.closest('[data-play]')) && playback.auto) playback.interact();
  }
  function stop() {
    epoch++;
    worker?.terminate(); worker = null; running = false; playback.interact();
    if (source === 'live') source = 'cached';
  }
  async function replay(ft = config.ft, autoplay = false) {
    const mayAutoplay = autoplay && playback.auto;
    stop(); const id = epoch;
    error = '';
    try {
      const response = await fetch(`${base}ft${ft}.srpl`);
      if (!response.ok) throw Error(s.failed);
      const bytes = await new Response(response.body!.pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
      const recording = decodeReplay(bytes);
      if (id !== epoch) return;
      active = recording.config; config = { ...active }; frames = recording.frames; source = 'replay';
      playback.setCount(frames.length);
      if (mayAutoplay && !interacted) { playback.auto = true; playback.startAutoplay({ reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches, narrow: matchMedia('(max-width: 640px)').matches }); }
      else playback.seek(0);
    } catch (e) { if (id === epoch) error = `${s.failed}: ${e}`; }
  }
  function run() {
    if (disabled || !base) return;
    stop(); const id = epoch;
    active = { ...config }; frames = []; error = ''; source = 'live'; running = true;
    playback.follow = true; playback.setCount(0);
    worker = new Worker(new URL('./sim.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = event => {
      if (id !== epoch) return;
      const message = event.data;
      if (message.type === 'step') {
        if (!validFrame(message.frame, active, frames.length)) { error = s.failed; stop(); return; }
        frames = [...frames, message.frame]; playback.setCount(frames.length);
      } else if (message.type === 'done') { running = false; source = 'cached'; worker?.terminate(); worker = null; }
      else { error = String(message.message); stop(); }
    };
    worker.onerror = () => { error = s.failed; stop(); };
    worker.postMessage({ config: $state.snapshot(active), base });
  }
  onMount(() => {
    let disposed = false;
    showMesh = !matchMedia('(max-width: 640px)').matches;
    const observer = new IntersectionObserver(entries => playback.setVisible(entries[0].isIntersecting));
    observer.observe(host);
    (async () => {
      try {
        const root = `${import.meta.env.BASE_URL}research-wasm/andreassen-2011/`;
        const manifest = await fetch(`${root}build.json`).then(r => { if (!r.ok) throw Error(s.failed); return r.json(); });
        if (!/^[a-f0-9]{64}$/.test(manifest.build_id)) throw Error(s.failed);
        if (disposed) return;
        base = `${root}${manifest.build_id}/`;
        const reference = await Promise.all([1, 2].map(ft => fetch(`${base}ft${ft}.json`).then(r => { if (!r.ok) throw Error(s.failed); return r.json(); })));
        if (disposed) return;
        refs = { 1: reference[0], 2: reference[1] };
        await replay(1, true);
      } catch (e) { if (!disposed) error = `${s.failed}: ${e}`; }
    })();
    return () => { disposed = true; stop(); observer.disconnect(); playback.dispose(); };
  });
</script>

<ExperimentShell paperId="andreassen-2011" name="Topology optimization of an MBB beam"
  lead="An educational SIMP optimizer: redistribute a fixed amount of material to reduce compliance."
  sourcePath="research/optimization/andreassen-2011"
  verification={{ reference: { kind: 'checks', passed: referenceChecks.filter(c => c.passed).length, total: referenceChecks.length, href: '#checks' }, published: { kind: 'none', why: 'Figure 3 settings reproduced; no digitized figure comparison or published compliance values claimed.' } }}
  actions={{ goToStep: n => playback.seek(n - 1), setParams: p => { playback.interact(); config = { ...config, ...p } as Config; }, showTab: t => { playback.interact(); tab = t; } }}>
  {#snippet article()}
    <h2>The experiment</h2>
    <p>The authors’ abstract describes a compact successor to the 99-line code, adding density filtering and improving efficiency through preallocation and vectorization. Here we implement its main minimum-compliance method.</p>
    <p>Each square uses a bilinear plane-stress element. SIMP interpolates its stiffness as E = 10⁻⁹ + ρᵖ(1 − 10⁻⁹). Optimality criteria redistribute material, with a move limit of 0.2. The engine’s sparse Cholesky solves equilibrium at every iteration.</p>
    <h2>Reproduction</h2>
    <p>Defaults: 60 × 20 elements, volume fraction 0.5, penalization 3, filter radius 2.4. These are section 3.4 and Figure 3(a) of the authors’ manuscript (pages 6–7). Sensitivity and density filtering use the same settings. Physical densities are shown.</p>
    <p>Compliance is measured before each update; volume and density are recorded after it, exactly as in top88. Stop when the largest design change is at most 0.01. The reference overlay is shown only for matching default parameters.</p>
    <h2 id="checks">Evidence and limitations</h2>
    <p>The reference runner downloads the authors’ unmodified function, checks its SHA-256, and observes full-precision data at its plotting calls in GNU Octave. Rust tests compare iteration counts, every compliance, volume and change, and final densities, alongside independent element and filter checks.</p>
    <p>The accessible manuscript reports optimized images and timing tables, not final compliance or iteration counts for these cases. We do not claim a paper-figure comparison. Agreement between implementations is not experimental validation or engineering certification. Alternate boundary conditions are delegated to the 2001 paper; this experiment supports the MBB load case.</p>
    <p><a href="https://www.topopt.mek.dtu.dk/apps-and-software/efficient-topology-optimization-in-matlab">Abstract, manuscript and authors’ reference</a>. Journal volume 43 (2011), pages 1–16; Crossref records the online publication in 2010.</p>
  {/snippet}
  {#snippet app()}
    <div bind:this={host} onpointerdowncapture={interact} onkeydowncapture={interact} onwheelcapture={interact} role="group" aria-label="Topology optimization">
      <p aria-live="polite">{s[source]}{#if running} · {s.iteration} {num(frames.length)}{/if}</p>
      <div class="buttons"><button onclick={() => tab = 'density'}>{s.density}</button><button onclick={() => tab = 'convergence'}>{s.convergence}</button></div>
      {#if tab === 'density'}
        <StabileoViewer {...geometry} values={frame?.density ?? null} perCell={true} range={{ min: 0, max: 1 }} title={s.density} colourMap="sequential-grey" legendFooter={s.material} {showMesh} height="360px" />
      {:else}
        <p>{#if overlay.length}{s.reference} · {/if}Stabileo</p>
        <LineChart series={series('compliance')} xLabel={s.iteration} yLabel={s.compliance} marker={frame?.iteration} />
        <LineChart series={series('volume')} xLabel={s.iteration} yLabel={s.volume} marker={frame?.iteration} />
        {#if !overlay.length}<p>{s.noOverlay}</p>{/if}
      {/if}
      {#if frame}<p>{s.iteration}: {num(frame.iteration)} · {s.compliance}: {num(frame.compliance, { dp: 4 })} · {s.volume}: {num(frame.volume, { dp: 5 })}</p>{:else}<p>{s.loading}</p>{/if}
      <input type="range" aria-label={s.iteration} min="0" max={Math.max(0, frames.length - 1)} value={playback.shown} oninput={e => playback.seek(+e.currentTarget.value)} />
      <div class="buttons"><button data-play onclick={() => playback.playing ? playback.pause() : playback.play()}>{playback.playing ? s.pause : s.play}</button><button onclick={run} disabled={disabled || running || !base}>{s.run}</button><button onclick={stop}>{s.stop}</button><button onclick={() => replay()} disabled={!base}>{s.restart}</button></div>
      <div class="parameters">
        {#each fields as field, i}<label>{field.key === 'volfrac' ? s.volume : field.key === 'penal' ? s.penal : field.key === 'rmin' ? s.radius : field.key}<NumberField bind:value={config[field.key]} bind:invalid={invalid[i]} min={field.min} max={field.max} label={field.key} /></label>{/each}
        <label>{s.filter}<select bind:value={config.ft}><option value={1}>{s.sensitivity}</option><option value={2}>{s.density}</option></select></label>
        <label>{s.load}<select><option>MBB</option></select></label>
        <label><input type="checkbox" bind:checked={showMesh} />{s.mesh}</label>
      </div>
      {#if disabled}<p role="alert">{s.invalid}</p>{/if}
      {#if error}<p role="alert">{error}</p>{/if}
    </div>
  {/snippet}
</ExperimentShell>
<style>
  .buttons { display: flex; flex-wrap: wrap; gap: .5rem; margin: .75rem 0; }
  button, select { background: var(--card); color: var(--fg); border: 1px solid var(--hair); padding: .5rem; border-radius: 3px; }
  button:disabled { opacity: .5; }
  .parameters { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; }
  label { display: grid; gap: .25rem; }
  input[type=range] { width: 100%; }
</style>
