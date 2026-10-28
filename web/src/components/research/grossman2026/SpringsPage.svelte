<script lang="ts">
  /**
   * /research/metamaterials/grossman-2026: spring networks after Grossman & Boudaoud (2026), run in
   * the reader's browser by research/metamaterials/grossman-2026 (WASM), shown in the shared
   * Stabileo viewer, compared with the paper's published numbers.
   *
   * Layout: the article (left) and a sticky app pane (right: viewer, controls, plots). Links in
   * the article call `apply`, which drives the app.
   */
  import { onDestroy, onMount } from 'svelte';
  import validation from '../../../../../research/metamaterials/grossman-2026/VALIDATION.json';
  import results from '../../../../../research/metamaterials/grossman-2026/results/stabileo_results.json';
  import fig5Csv from '../../../../../research/metamaterials/grossman-2026/reference-results/fig5_simulation_digitized.csv?raw';
  import { REPO_URL } from '../../landing/landing-utils';
  import ExperimentShell from '../shell/ExperimentShell.svelte';
  import ExperimentLink from '../shell/ExperimentLink.svelte';
  import StabileoViewer from '../shell/StabileoViewer.svelte';
  import PaneTabs from '../shell/PaneTabs.svelte';
  import { publicI18n } from '../../../lib/i18n/store.svelte';
  import { Playback, SOURCE_LABEL } from '../shell/playback.svelte';
  import PlotXY, { type PlotSeries } from './PlotXY.svelte';
  import { loadSprings, solvePeriodic, solveStrip, type LatticeKind, type NetworkSpec, type PeriodicResult, type StripResult } from './springs';
  import { embeddedEditorUrl, fullEditorUrl, type StripLabel } from './open-in-stabileo';

  const SOURCE = `${REPO_URL}/tree/main/research/metamaterials/grossman-2026`;
  const DOI = 'https://doi.org/10.1103/17sg-gzhl';
  // Hue is the source (ours blue, the paper orange); dashes mark a variant within a source.
  const OURS = '#3987e5';
  const PAPER = '#d95926';

  type Family = 'foam' | 'ordered' | 'honeycomb' | 'square';
  let family = $state<Family>('foam');
  let eta = $state(0.3);
  let phi = $state(2);
  let psi = $state(0.5);
  let theta = $state(0);
  let ell = $state(0.25);
  let diagonals = $state(false);
  let keep = $state(1);
  let prestress = $state(0);
  let strain = $state(0.01);
  let seed = $state(0);
  let law = $state<'hookean' | 'equal_weight'>('hookean');
  let deform = $state(10);

  const locale = $derived(publicI18n.locale);
  const nf = (v: number | null | undefined, d = 3) =>
    v === null || v === undefined || !Number.isFinite(v) ? '—' : new Intl.NumberFormat(locale, { maximumFractionDigits: d, minimumFractionDigits: d }).format(v);

  const lattice = $derived<LatticeKind>(
    family === 'honeycomb'
      ? { kind: 'honeycomb', l: ell, k_radial: 1e-3 }
      : family === 'square'
        ? { kind: 'square', diagonals }
        : family === 'ordered'
          ? { kind: 'triangular', phi, psi }
          : { kind: 'triangular', phi: 1, psi: 1 },
  );
  const network = $derived<NetworkSpec>({
    lattice,
    eta: family === 'foam' ? eta : 0,
    keep,
    seed,
    spring_law: family === 'ordered' ? law : 'hookean',
    k: 1,
    prestress,
  });
  // The paper pulls along y. The honeycomb's Eq. (23) is for a pull along the 2l axis, so its
  // strip is turned by 90 degrees.
  const stripTheta = $derived(family === 'ordered' ? theta : family === 'honeycomb' ? Math.PI / 2 : 0);
  const cells = $derived(family === 'honeycomb' ? 6 : family === 'foam' ? 26 : 8);

  let glue = $state.raw<Awaited<ReturnType<typeof loadSprings>> | null>(null);
  let loadError = $state<string | null>(null);
  let strip = $state.raw<StripResult | null>(null);
  let cell = $state.raw<PeriodicResult | null>(null);
  let ordered = $state.raw<{ hook: PeriodicResult; eq: PeriodicResult } | null>(null);
  let solveError = $state<string | null>(null);
  let solveMs = $state(0);

  const playback = new Playback(140, 1800);
  const SWEEP = Array.from({ length: 26 }, (_, i) => i * 0.02);
  let autoplaying = $state(false);

  function solve() {
    if (!glue) return;
    const t0 = performance.now();
    try {
      const spec = $state.snapshot(network) as NetworkSpec;
      strip = solveStrip(glue, { network: spec, width: 13, height: 52, theta: stripTheta, clamp_band: 0.5, strain });
      cell = solvePeriodic(glue, { network: spec, n1: cells, n2: cells, strain: [0, 0, 0] });
      if (family === 'ordered' && keep === 1 && prestress === 0) {
        const base = { ...spec, keep: 1, prestress: 0 };
        ordered = {
          hook: solvePeriodic(glue, { network: { ...base, spring_law: 'hookean' }, n1: 4, n2: 4, strain: [0, 0, 0] }),
          eq: solvePeriodic(glue, { network: { ...base, spring_law: 'equal_weight' }, n1: 4, n2: 4, strain: [0, 0, 0] }),
        };
      } else ordered = null;
      solveError = null;
    } catch (e) {
      solveError = e instanceof Error ? e.message : String(e);
    }
    solveMs = performance.now() - t0;
  }

  $effect(() => {
    // Re-solve whenever an input changes (reads every input through `network`, `strain`, ...).
    void network;
    void strain;
    void stripTheta;
    void cells;
    if (glue) solve();
  });

  $effect(() => {
    if (!autoplaying) return;
    eta = SWEEP[playback.shown];
  });

  let pane: HTMLElement;
  onMount(() => {
    // Deep links: #edit opens the editor tab; ?eta=… (foam) sets the disorder and skips the sweep
    // (also how the gallery thumbnail is rendered: scripts/research-thumbs.mjs with thumbnailQuery).
    const q = new URLSearchParams(window.location.search);
    const qEta = Number(q.get('eta'));
    const deepLinked = q.has('eta') && Number.isFinite(qEta) && qEta >= 0 && qEta <= 0.5;
    if (deepLinked) eta = qEta;
    thumbMode = q.has('thumb');
    loadSprings()
      .then((g) => {
        glue = g;
        playback.setCount(SWEEP.length);
        if (window.location.hash === '#edit') {
          showTab('edit');
          return;
        }
        if (deepLinked) return;
        const reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
        const narrow = window.innerWidth < 640;
        if (reducedMotion) return;
        autoplaying = true;
        playback.startAutoplay({ reducedMotion, narrow });
      })
      .catch((e) => (loadError = e instanceof Error ? e.message : String(e)));
    const io = new IntersectionObserver((entries) => playback.setVisible(entries.some((x) => x.isIntersecting)));
    if (pane) io.observe(pane);
    return () => io.disconnect();
  });
  onDestroy(() => playback.dispose());

  $effect(() => {
    if (autoplaying && !playback.playing && !playback.auto) autoplaying = false;
  });

  /** Any control: the sweep ends for good. */
  function touch() {
    if (autoplaying) {
      autoplaying = false;
      playback.pause();
    }
  }

  /** What the article's links ask of the app pane (ExperimentLink). */
  const actions = {
    goToStep(i: number) {
      touch();
      eta = SWEEP[Math.max(0, Math.min(SWEEP.length - 1, i))];
    },
    setParams(p: Record<string, number | string | boolean>) {
      apply(p as Parameters<typeof apply>[0]);
    },
    showTab(t: string) {
      if (t === 'experiment' || t === 'edit') showTab(t);
    },
  };
  const passed = validation.checks.filter((c) => c.passed).length;

  /** Links in the article set the app's state. */
  function apply(p: Partial<{ family: Family; eta: number; phi: number; psi: number; theta: number; ell: number; keep: number; prestress: number; diagonals: boolean; law: 'hookean' | 'equal_weight'; strain: number }>) {
    touch();
    if (p.family !== undefined) family = p.family;
    if (p.eta !== undefined) eta = p.eta;
    if (p.phi !== undefined) phi = p.phi;
    if (p.psi !== undefined) psi = p.psi;
    if (p.theta !== undefined) theta = p.theta;
    if (p.ell !== undefined) ell = p.ell;
    if (p.diagonals !== undefined) diagonals = p.diagonals;
    if (p.law !== undefined) law = p.law;
    keep = p.keep ?? 1;
    prestress = p.prestress ?? 0;
    if (p.strain !== undefined) strain = p.strain;
  }

  // ---- 2x2 algebra on C (exx, eyy, exy; tensor shear) for directional moduli -------------
  function inv3(m: number[][]): number[][] | null {
    const det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    const s = Math.max(...m.flat().map(Math.abs));
    if (!(Math.abs(det) > 1e-20 * s * s * s)) return null;
    const r = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
    for (let i = 0; i < 3; i++)
      for (let j = 0; j < 3; j++) {
        const [a, b, c, d] = [(j + 1) % 3, (j + 2) % 3, (i + 1) % 3, (i + 2) % 3];
        r[i][j] = (m[a][c] * m[b][d] - m[a][d] * m[b][c]) / det;
      }
    return r;
  }
  /** Young's modulus and Poisson's ratio under uniaxial stress along direction `a`. */
  function uniaxial(c: number[][], a: number): [number, number] | null {
    const [co, si] = [Math.cos(a), Math.sin(a)];
    // Strain along the load direction n and across it t, as linear forms of the stress state.
    const m = c.map((row, i) => row.map((v) => (i === 2 ? v / 2 : v)));
    const s = inv3(m);
    if (!s) return null;
    // Stress = n n^T: (sxx, syy, sxy) = (co^2, si^2, co si).
    const sig = [co * co, si * si, co * si];
    const e = [0, 1, 2].map((i) => s[i][0] * sig[0] + s[i][1] * sig[1] + s[i][2] * sig[2]);
    const enn = e[0] * co * co + e[1] * si * si + 2 * e[2] * co * si;
    const ett = e[0] * si * si + e[1] * co * co - 2 * e[2] * co * si;
    return enn > 0 ? [1 / enn, -ett / enn] : null;
  }
  /** Supplemental Eq. (9) of the paper, as printed. */
  function eq9(f: number, p: number, t: number): number {
    const s3 = Math.sqrt(3);
    const a = f * f - 2 * f;
    const [p2, p4, p6] = [p * p, p ** 4, p ** 6];
    const P =
      Math.cos(4 * t) * (27 * p6 - (f - 2) ** 2 * f * f * (f * f - 2 * f + 4) + 9 * p4 * a - 3 * p2 * (a * (a - 20) - 16)) +
      Math.sin(4 * t) * (2 * s3 * p * (f - 1) * (9 * p4 + 6 * p2 * (a - 2) + a * (a + 8))) -
      48 * p2 +
      (a - 3 * p2) * (9 * p4 + 6 * p2 * a + a * (a + 4));
    const [S, C] = [Math.sin(t), Math.cos(t)];
    const Q =
      36 * p4 * S ** 4 +
      2 * C ** 4 * (6 * p2 * a * a + a * a * (a + 4) + 9 * p4 * (a + 2)) +
      6 * p2 * S * S * C * C * (9 * p4 + f ** 4) +
      3 * p2 * Math.sin(2 * t) ** 2 * (3 * p2 * (a + 2) - 2 * (f - 2) * (f * f - 4 * f + 2)) -
      4 * s3 * p * (f - 1) * S * C ** 3 * (9 * p4 + 6 * p2 * a + a * (a + 8)) -
      48 * s3 * p ** 3 * (f - 1) * S ** 3 * C;
    return -P / (4 * Q);
  }
  const eq23 = (l: number) => (-(2 - 4 * l) * (l + 0.5)) / (3 + 4 * l - 4 * l * l);

  // ---- what the viewer shows -------------------------------------------------------------
  // The strip is drawn lying down (the paper's y axis to the right) so it fills the wide pane:
  // drawn (X, Y) = (y, 13 - x).
  const xy = $derived(strip ? Float64Array.from(strip.nodes.flatMap((p) => [p[1], 13 - p[0]])) : new Float64Array());
  const bars = $derived(strip ? Uint32Array.from(strip.bonds.flat()) : new Uint32Array());
  const disp = $derived(strip ? Float64Array.from(strip.displacement.flatMap((u) => [u[1], -u[0]])) : null);
  // Forces per unit imposed strain, N / (k ε), so the colours do not depend on the strain's size.
  const forceScale = $derived(Math.abs(strain) > 0 ? 1 / Math.abs(strain) : 1);
  const forces = $derived(strip ? strip.force.map((f) => f * forceScale) : null);
  const forceRange = $derived.by(() => {
    if (!forces) return { min: -1, max: 1 };
    const m = Math.max(1e-12, ...forces.map(Math.abs));
    return { min: -m, max: m };
  });
  // Room under the strip for the legend, except when rendering the gallery thumbnail (?thumb).
  let thumbMode = $state(false);
  const VIEW = $derived<[number, number, number, number]>([-1, thumbMode ? -5 : -9, 53 * (1 + Math.max(0, deform * strain)) + 1, thumbMode ? 18 : 14]);
  // The paper's convention for the pulled direction, in the periodic cell's frame.
  const pullAngle = $derived(family === 'honeycomb' ? 0 : Math.PI / 2 - stripTheta);
  const cellAlong = $derived(cell ? uniaxial(cell.c, pullAngle) : null);

  // ---- plots ---------------------------------------------------------------------------------
  type MS = { mean: number; std: number };
  const foamP = results.foam.periodic as { eta: number; nu: MS; bulk: MS; shear: MS; young_over_young0: MS; nonaffinity_shear: MS }[];
  const foamS = results.foam.strip_13x52 as { eta: number; nu: MS }[];
  const fig5 = fig5Csv
    .trim()
    .split(/\r?\n/)
    .slice(1)
    .map((l) => l.split(','))
    .map((c) => ({ x: Number(c[0]), y: c[1] === '' ? NaN : Number(c[1]), lo: Number(c[2]), hi: Number(c[3]) }));
  const foamSeries = $derived<PlotSeries[]>([
    { name: `Ours, periodic ${results.foam.cells}² cells, ±1 s.d. of ${results.foam.seeds} samples`, color: OURS, kind: 'band', points: foamP.map((r) => ({ x: r.eta, y: r.nu.mean, lo: r.nu.mean - r.nu.std, hi: r.nu.mean + r.nu.std })) },
    { name: `Ours, periodic (exact moduli), mean of ${results.foam.seeds}`, color: OURS, kind: 'line', points: foamP.map((r) => ({ x: r.eta, y: r.nu.mean })) },
    { name: 'Ours, the paper’s 13 × 52 strip protocol', color: OURS, kind: 'line', dashed: true, points: foamS.map((r) => ({ x: r.eta, y: r.nu.mean })) },
    { name: 'Paper, Fig. 5 simulations (digitised, with error bars)', color: PAPER, kind: 'points', shape: 'triangle', points: fig5.filter((p) => Number.isFinite(p.y)) },
    { name: 'Paper, text: ν = 0 at η ≃ 0.46, −0.1 as η → 0.5', color: PAPER, kind: 'points', shape: 'square', points: [{ x: 0.46, y: 0 }, { x: 0.5, y: -0.1 }] },
  ]);
  type Dil = { z: number; bulk: number; shear: number };
  const dil = results.dilution as Dil[];
  const dilSeries: PlotSeries[] = [
    { name: 'Bulk modulus B / k', color: OURS, kind: 'line', points: dil.map((r) => ({ x: r.z, y: r.bulk })) },
    { name: 'Shear modulus G / k', color: OURS, kind: 'line', dashed: true, points: dil.map((r) => ({ x: r.z, y: r.shear })) },
  ];
  type Honey = { l: number; eq23: number; 'nu_k1e-3': number };
  const honeySeries: PlotSeries[] = [
    { name: 'Paper, Eq. (23)', color: PAPER, kind: 'line', points: Array.from({ length: 91 }, (_, i) => 0.05 + i * 0.01).map((l) => ({ x: l, y: eq23(l) })) },
    { name: 'Ours, periodic network, k_radial = 10⁻³', color: OURS, kind: 'points', points: (results.honeycomb as Honey[]).map((r) => ({ x: r.l, y: r['nu_k1e-3'] })) },
  ];
  const orderedSeries = $derived.by<PlotSeries[]>(() => {
    if (!ordered) return [];
    const ts = Array.from({ length: 91 }, (_, i) => (Math.PI * i) / 90);
    const pick = (r: PeriodicResult) => ts.map((t) => ({ x: t, y: uniaxial(r.c, Math.PI / 2 - t)?.[1] ?? NaN }));
    return [
      { name: 'Paper, Supplemental Eq. (9)', color: PAPER, kind: 'line', points: ts.map((t) => ({ x: t, y: eq9(phi, psi, t) })) },
      { name: 'Ours, equal-weight springs (k ∝ 1/ℓ²)', color: OURS, kind: 'line', dashed: true, points: pick(ordered.eq) },
      { name: 'Ours, equal Hookean springs (Eq. 3)', color: OURS, kind: 'line', points: pick(ordered.hook) },
    ];
  });

  const fmtSci = (v: number) => (v === 0 ? '0' : Math.abs(v) >= 0.01 && Math.abs(v) < 1000 ? v.toPrecision(3) : v.toExponential(1));

  // ---- the Stabileo editor, embedded (loaded only when its tab is opened) -------------------
  let tab = $state<'experiment' | 'edit'>('experiment');
  // PaneTabs binds a plain string; any switch counts as the visitor taking over.
  let tabId = $state<string>('experiment');
  $effect(() => {
    if (tabId !== tab) showTab(tabId === 'edit' ? 'edit' : 'experiment');
  });
  let editorSrc = $state<string | null>(null);
  let editorLoading = $state(false);
  let editorStrip = $state.raw<StripResult | null>(null);
  let editorError = $state<string | null>(null);
  const labelParams = (): StripLabel => ({ family, eta, keep, seed, strain });
  const e2e = typeof window !== 'undefined' && new URLSearchParams(window.location.search).get('e2e') === '1';

  async function loadEditor() {
    if (!strip) return;
    editorError = null;
    editorLoading = true;
    editorStrip = strip;
    // Test builds with ?e2e=1: the strip handed to the editor, so a spec can compare the
    // production engine's bar forces with ours.
    if (e2e) (window as unknown as { __springsEditorStrip?: StripResult }).__springsEditorStrip = strip;
    try {
      editorSrc = await embeddedEditorUrl(strip, labelParams(), e2e);
    } catch (e) {
      editorError = e instanceof Error ? e.message : String(e);
      editorLoading = false;
    }
  }
  function showTab(t: 'experiment' | 'edit') {
    touch();
    tab = t;
    tabId = t;
  }
  // The embedded editor needs the page's width (the app switches to its phone layout below 768 px):
  // while it is shown, the shell's grid is one column with the app first.
  $effect(() => {
    const grid = pane?.closest('.rs-shell-grid');
    grid?.classList.toggle('rs-editing', tab === 'edit');
    return () => grid?.classList.remove('rs-editing');
  });
  // The editor loads the first time its tab is shown (once a strip exists).
  $effect(() => {
    if (tab === 'edit' && strip && !editorSrc && !editorLoading) void loadEditor();
  });
  async function openFullEditor() {
    if (!strip) return;
    const w = window.open('about:blank', '_blank');
    const url = await fullEditorUrl(strip, labelParams());
    if (w) {
      w.opener = null;
      w.location.href = url;
    } else window.location.assign(url);
  }
</script>
<ExperimentShell
  paperId="grossman-2026"
  name="Auxetic spring networks"
  lead="Pull a strip of Hookean springs and read how much it narrows. Shake the nodes of the triangular lattice and it starts to widen instead: it becomes auxetic. The network on the right is solved in your browser as a truss on Stabileo’s engine, and the same strip opens in the real Stabileo editor. No reference code is public (“available at request”), so we compare with the paper’s published numbers, closed forms and figures, and with exact results."
  verification={{
    reference: { kind: 'none', why: 'None is public: the authors’ code is “available at request”' },
    published: { kind: 'checks', passed, total: validation.checks.length, href: '#checks' },
  }}
  {actions}
  sourcePath="research/metamaterials/grossman-2026"
>
  {#snippet article()}
      <h2>What the paper shows</h2>
      <p>
        Many materials, from metamaterial lattices to cell membranes, are modelled as networks of Hookean springs. Grossman and Boudaoud derive the elastic continuum
        that such a network behaves like, using only its geometry and topology, and they compute the <em>nonaffine</em> displacements: the local deviations from a uniform
        stretch that make disordered networks softer than the uniform-strain estimate. They check the theory against simulations of a stretched strip for three families:
        ordered triangular lattices, a honeycomb that turns re-entrant, and a “foam” made by randomly displacing the nodes of a triangular lattice, which becomes
        <strong>auxetic</strong> (negative Poisson’s ratio: it widens when pulled) as the disorder grows.
      </p>
      <p class="sp-try">
        Try it: <ExperimentLink params={{ family: 'foam', eta: 0 }}>perfect lattice, η = 0</ExperimentLink> ·
        <ExperimentLink params={{ family: 'foam', eta: 0.3 }}>η = 0.3</ExperimentLink> ·
        <ExperimentLink params={{ family: 'foam', eta: 0.5 }}>η = 0.5, auxetic</ExperimentLink>
      </p>

      <h2>The model we solve</h2>
      <p>
        Springs <i>e</i> join nodes; each has stiffness <i>k<sub>e</sub></i>, rest length ℓ̄<sub>e</sub> and current length ℓ<sub>e</sub>, and the energy is
        <span class="eq"><var>E</var> = ½ Σ<sub>e</sub> <var>k</var><sub>e</sub> (<var>ℓ</var><sub>e</sub> − <var>ℓ̄</var><sub>e</sub>)²</span> (the paper’s Eq. 3). A spring network is a pin-jointed truss, so the right pane
        shows it in the Stabileo viewer: each bar coloured by its axial force, on the deformed shape. We compute two things.
      </p>
      <ul>
        <li>
          <b>The paper’s strip test.</b> A 13 × 52 strip (about 800 nodes), its top and bottom rows held, the top moved up by the imposed strain; the Poisson’s ratio is read
          from the width change between the two edge nodes nearest mid-height, as in the paper’s Sec. V A. Solved as a linear truss on the Stabileo engine’s sparse Cholesky
          factorisation; the same model solved by the production engine’s 2D truss analysis agrees to 10⁻¹⁴.
        </li>
        <li>
          <b>Exact effective moduli.</b> A periodic cell under an imposed macroscopic strain, with every node free to relax: the stiffness that remains is the network’s elastic
          tensor C, including the nonaffine softening, with no boundary effects. The uniform-strain (affine) tensor is the same sum without relaxation.
        </li>
        <li>
          <b>Prestress</b> (an addition, not in the paper’s computations): rest lengths shortened by a fraction, the network relaxed to equilibrium by Newton’s method, and the
          linear response taken about that state, tension stiffening included.
        </li>
      </ul>

      <h2>Comparison with the paper</h2>
      <h3>Foam: the auxetic transition</h3>
      <p>
        The paper reports ν falling with the disorder η, crossing zero at η ≃ 0.46 and reaching −0.1 as η → 0.5. Our exact periodic moduli (10 samples of 26 × 26 cells for each
        η) cross zero at η = 0.446 and give ν = −0.148 at η = 0.5. The mean falls inside every error bar of the paper’s simulations in Fig. 5 for η ≤ 0.45. The paper’s marker at
        η = 0.49 is an arrow below its plot (about −0.14); ours there is −0.117.
      </p>
      <PlotXY series={foamSeries} xLabel="disorder η" yLabel="Poisson’s ratio ν" xDomain={[0, 0.5]} current={family === 'foam' && cell ? { x: eta, y: cell.isotropic.poisson, label: 'This network (periodic)' } : null} ariaLabel="Poisson's ratio against disorder: ours and the paper's" />
      <p class="sp-note">
        Our implementation of the strip protocol (dashed) reads 0.371 for the perfect lattice, where the exact value is 1/3 and the paper’s Fig. 5 shows 0.331. The excess is a
        free-edge effect of order 1/width: strips of 26 and 52 widths read 0.354 and 0.345, extrapolating to 0.336. The paper’s strip does not show it, and its text does not say
        enough about the measurement for us to find out why.
      </p>

      <h3>Honeycomb: Eq. (23)</h3>
      <p>
        A honeycomb of unit edges whose opposing corners are 2ℓ apart is regular at ℓ = 1 and re-entrant below ℓ = ½. The paper triangulates each hexagon with a centre node and
        springs 10⁻³ as stiff as the edges, and compares with the closed form ν(ℓ) = −(2 − 4ℓ)(ℓ + ½)/(3 + 4ℓ − 4ℓ²). Our periodic network agrees to 3.4 × 10⁻³ with those springs and
        to 3.4 × 10⁻⁶ with springs 10⁻⁶ as stiff: the difference is the soft springs’ own stiffness.
        <ExperimentLink params={{ family: 'honeycomb', ell: 0.25 }}>Show ℓ = 0.25 (re-entrant)</ExperimentLink> ·
        <ExperimentLink params={{ family: 'honeycomb', ell: 0.9 }}>ℓ = 0.9</ExperimentLink>
      </p>
      <PlotXY series={honeySeries} xLabel="ℓ" yLabel="Poisson’s ratio ν" xDomain={[0, 1]} current={family === 'honeycomb' && cellAlong ? { x: ell, y: cellAlong[1], label: 'This network' } : null} ariaLabel="Honeycomb Poisson's ratio: Eq. 23 and our network" />

      <h3>Ordered lattices: an observation about spring constants</h3>
      <p>
        For ordered triangular lattices the paper gives a closed form, Supplemental Eq. (9), for ν as a function of the cell shape (φ, ψ) and orientation θ, plotted in its Fig. 4.
        A Bravais lattice has no nonaffine displacements, so the exact answer is the uniform-strain one, which we compute independently. Supplemental Eq. (9) and the
        calculation line of Fig. 7 (Young’s modulus of the (3, 2) lattice, read from the PDF’s vector data, times the area of one triangle) coincide with that affine result
        for spring constants k<sub>e</sub> ∝ 1/ℓ̄<sub>e</sub>², to 10⁻¹⁴ and 6 × 10⁻⁴. With equal spring constants we obtain ν = 1/6 at (φ, ψ, θ) = (1, 0.5, 0), where Eq. (9)
        gives 8/27. This may reflect a convention we have not identified; it is on our list of questions for the authors.
        <ExperimentLink params={{ family: 'ordered', phi: 1, psi: 0.5, theta: 0, law: 'hookean' }}>Show (1, 0.5)</ExperimentLink> ·
        <ExperimentLink params={{ family: 'ordered', phi: 2, psi: 0.5, theta: 0.7, law: 'equal_weight' }}>(2, 0.5) at θ = 0.7, equal weight</ExperimentLink>
      </p>
      {#if family === 'ordered' && orderedSeries.length}
        <PlotXY series={orderedSeries} xLabel="lattice orientation θ, rad" yLabel="Poisson’s ratio ν" xDomain={[0, Math.PI]} current={cellAlong ? { x: theta, y: cellAlong[1], label: 'This network' } : null} ariaLabel="Ordered lattice Poisson's ratio against orientation" />
      {:else}
        <p class="sp-note">Select an ordered lattice (without dilution or prestress) to plot Eq. (9) against both spring laws.</p>
      {/if}
      <p>
        We could not reproduce the simulation dots of Fig. 7: the paper does not say how its strip simulation measures Young’s modulus, and our reading (vertical force through
        mid-height over width and strain) differs from them by up to 0.5.
      </p>

      <h3>Rigidity and connectivity (an independent check)</h3>
      <p>
        Not in the paper: removing bonds at random from the triangular lattice lowers the mean coordination z from 6. Maxwell’s count (2 degrees of freedom per node against z/2
        bonds per node) says a generic network cannot be rigid below z = 4. Our moduli vanish to rounding at z = 3 and stay finite above about z = 3.9, the rigidity
        percolation threshold (e.g. Jacobs and Thorpe, <a href="https://doi.org/10.1103/PhysRevLett.75.4051" rel="noreferrer" target="_blank">PRL 75, 4051, 1995</a>).
        <ExperimentLink params={{ family: 'foam', eta: 0, keep: 0.62 }}>Dilute to z ≈ 3.7</ExperimentLink> ·
        <ExperimentLink params={{ family: 'foam', eta: 0, keep: 0.62, prestress: 0.03 }}>…and put it in tension</ExperimentLink>
      </p>
      <PlotXY series={dilSeries} xLabel="mean coordination z" yLabel="modulus / k" xDomain={[2.4, 6]} rules={[{ x: 4, label: 'z = 4' }]} current={family === 'foam' && eta === 0 && cell ? { x: cell.coordination, y: cell.isotropic.shear, label: 'This network: G' } : null} ariaLabel="Moduli against coordination" />

      <h3>All checks</h3>
      <table class="sp-checks" id="checks">
        <caption>{validation.checks.length} validation checks run in CI, each with its tolerance</caption>
        <thead><tr><th scope="col">Check</th><th scope="col">Observed</th><th scope="col">Tolerance</th><th scope="col">Result</th></tr></thead>
        <tbody>
          {#each validation.checks as c (c.name)}
            <tr><td>{c.name}<br /><span class="m">{c.metric}</span></td><td>{fmtSci(c.observed)}</td><td>{fmtSci(c.tolerance)}</td><td>{c.passed ? 'pass' : 'FAIL'}</td></tr>
          {/each}
        </tbody>
      </table>

      <h2>Limitations</h2>
      <ul>
        <li>No reference code is public. Comparisons rest on the paper’s text, its closed forms, the vector data of Figs. 7–9 and a digitisation of the raster Fig. 5 (±0.002).</li>
        <li>We have not implemented the paper’s own semi-analytical method (the nonaffine tensors W of its Eq. 20 in the mean-field approximation); we compare with its results.</li>
        <li>Linear response only (small strain). The paper minimised the nonlinear energy at 1% strain; at that strain the difference is of order 1%.</li>
        <li>The strip’s free edges bias the measured ν by O(1/width); the periodic moduli have no such bias.</li>
        <li>Our ensemble (10 samples per η) estimates the mean to about ±0.005 at η ≤ 0.3 and ±0.01 near 0.5.</li>
        <li>Agreement with a paper is not validation against experiments, and nothing here is an engineering design result.</li>
      </ul>

      <h2>Code and references</h2>
      <ul>
        <li>Implementation, tests, reference values and their provenance: <a href={SOURCE} rel="noreferrer" target="_blank">research/metamaterials/grossman-2026</a> (AGPL-3.0).</li>
        <li>Article and Supplemental Material: <a href={DOI} rel="noreferrer" target="_blank">doi:10.1103/17sg-gzhl</a>, CC BY 4.0. Figures on this page are drawn from our own runs; the paper’s points are its published data.</li>
        <li>Foam construction after J. Liu, Y. Nie, H. Tong and N. Xu, <a href="https://doi.org/10.1103/PhysRevMaterials.3.055607" rel="noreferrer" target="_blank">Phys. Rev. Materials 3, 055607 (2019)</a>, the paper’s Ref. 27.</li>
      </ul>
      {/snippet}

  {#snippet app()}
      <div class="sp-app" bind:this={pane}>
        <div class="sp-tabbar">
          <PaneTabs tabs={[{ id: 'experiment', label: 'Experiment' }, { id: 'edit', label: 'Edit in Stabileo' }]} bind:active={tabId} />
          <button class="sp-full" type="button" onclick={openFullEditor} disabled={!strip}>Open full editor ↗</button>
        </div>
          <div class="sp-editor" role="tabpanel" id="rs-panel-edit" aria-labelledby="rs-tab-edit" hidden={tab !== 'edit'}>
            <p class="sp-source">
              Stabileo editor (production engine): this strip as an ordinary truss, end rows pinned, the far end moved by the imposed strain. Edit and press Solve. Bar forces are
              1000 × the experiment’s (E·A/L = 1000 k kN/m).{#if prestress > 0} Prestress is not carried over.{/if}
              {#if editorStrip && strip !== editorStrip}<button class="link" type="button" onclick={loadEditor}>Load the experiment’s current network</button>{/if}
            </p>
            {#if editorError}<p class="warnt">{editorError}</p>{/if}
            <div class="sp-frame">
              {#if editorLoading}<div class="sp-frame-loading">Loading the Stabileo editor…</div>{/if}
              {#if editorSrc}
                <iframe src={editorSrc} title="Stabileo editor (production engine)" onload={() => (editorLoading = false)}></iframe>
              {/if}
            </div>
          </div>
        <div role="tabpanel" id="rs-panel-experiment" aria-labelledby="rs-tab-experiment" hidden={tab !== 'experiment'}>
        <p class="sp-source" aria-live="polite">
          {#if loadError}
            <span class="warnt">The simulation bundle is not part of this build ({loadError}). Build it with research/metamaterials/grossman-2026/wasm/build.sh.</span>
          {:else if !glue}
            Loading the solver…
          {:else if autoplaying}
            Live sweep of η in your browser (our solver) <button class="sp-pause" type="button" onclick={touch}>Pause sweep</button>
          {:else}
            {SOURCE_LABEL.live} · {nf(solveMs, 0)} ms
          {/if}
        </p>
        <div class="sp-viewer" data-nodes={strip?.nodes.length ?? 0}>
          {#if strip}
                          <StabileoViewer
                xy={xy}
                cells={bars}
                cellSize={2}
                lineWidth={2}
                values={forces}
                perCell={true}
                range={forceRange}
                title={Math.abs(strain) > 0 ? 'Axial force N / (k ε)' : 'Axial force N / k'}
                unit=""
                legendFooter="tension positive · strip pulled left to right, end rows held"
                view={VIEW}
                displacement={disp}
                deformScale={deform}
                showMesh={false}
                height="min(26rem, 55vh)"
                ariaLabel="Stabileo viewer: the spring network coloured by axial force on its deformed shape"
                status={[{ label: 'nodes', value: String(strip.nodes.length) }, { label: 'springs', value: String(strip.bonds.length) }]}
                editorButton={false}
              />
            
          {:else}
            <div class="sp-placeholder">The network appears here.</div>
          {/if}
        </div>
        {#if solveError}<p class="warnt">The solver refused the input: {solveError}</p>{/if}
        <dl class="sp-readout">
          <div><dt>ν, strip (paper’s protocol)</dt><dd>{nf(strip?.poisson)}</dd></div>
          <div><dt>ν along the pull, periodic (exact)</dt><dd>{nf(cellAlong?.[1])}</dd></div>
          <div><dt>B / k · G / k (affine)</dt><dd>{nf(cell?.isotropic.bulk)} · {nf(cell?.isotropic.shear)} <span class="m">({nf(cell?.isotropic_affine.bulk)} · {nf(cell?.isotropic_affine.shear)})</span></dd></div>
          <div><dt>coordination z · nonaffinity</dt><dd>{nf(cell?.coordination, 2)} · {nf(cell?.nonaffinity_shear, 4)}</dd></div>
        </dl>
        <form class="sp-controls" oninput={touch} onsubmit={(e) => e.preventDefault()}>
          <label>Network
            <select bind:value={family} aria-label="Network">
              <option value="foam">Foam (random node shifts η)</option>
              <option value="ordered">Ordered triangular lattice (φ, ψ, θ)</option>
              <option value="honeycomb">Honeycomb (ℓ)</option>
              <option value="square">Square lattice</option>
            </select>
          </label>
          {#if family === 'foam'}
            <label>Disorder η <input type="range" min="0" max="0.5" step="0.01" bind:value={eta} aria-label="Disorder eta" /><span class="v">{nf(eta, 2)}</span></label>
          {:else if family === 'ordered'}
            <label>Shear φ <input type="range" min="0.5" max="3" step="0.05" bind:value={phi} aria-label="phi" /><span class="v">{nf(phi, 2)}</span></label>
            <label>Elongation ψ <input type="range" min="0.4" max="2" step="0.05" bind:value={psi} aria-label="psi" /><span class="v">{nf(psi, 2)}</span></label>
            <label>Orientation θ <input type="range" min="0" max="3.14" step="0.02" bind:value={theta} aria-label="theta" /><span class="v">{nf(theta, 2)}</span></label>
            <label>Springs
              <select bind:value={law} aria-label="Spring law"><option value="hookean">equal k (Eq. 3)</option><option value="equal_weight">k ∝ 1/ℓ² (as Eq. 9)</option></select>
            </label>
          {:else if family === 'honeycomb'}
            <label>Corner distance ℓ <input type="range" min="0.05" max="0.95" step="0.01" bind:value={ell} aria-label="ell" /><span class="v">{nf(ell, 2)}</span></label>
          {:else}
            <label class="chk"><input type="checkbox" bind:checked={diagonals} /> Diagonals</label>
          {/if}
          <label>Bonds kept <input type="range" min="0.4" max="1" step="0.01" bind:value={keep} aria-label="Fraction of bonds kept" /><span class="v">{nf(keep, 2)}</span></label>
          <label>Prestress (rest lengths shortened) <input type="range" min="0" max="0.1" step="0.005" bind:value={prestress} aria-label="Prestress: rest lengths shortened by" /><span class="v">{nf(prestress * 100, 1)} %</span></label>
          <label>Imposed strain <input type="range" min="-0.05" max="0.05" step="0.005" bind:value={strain} aria-label="Imposed strain" /><span class="v">{nf(strain * 100, 1)} %</span></label>
          <label>Deformation × <input type="range" min="0" max="60" step="1" bind:value={deform} aria-label="Deformation scale" /><span class="v">{deform}</span></label>
          <label>Seed <input class="seed" type="number" min="0" max="999" step="1" bind:value={seed} aria-label="Random seed" /></label>
        </form>
        </div>
      </div>
      {/snippet}
</ExperimentShell>

<style>
  .eq {
    white-space: nowrap;
  }
  .eq var {
    font-style: italic;
  }
  button.link {
    background: none;
    border: none;
    padding: 0;
    color: var(--rs-link, #ff7a55);
    text-decoration: underline dotted;
    text-underline-offset: 3px;
    cursor: pointer;
    font: inherit;
  }
  .sp-try {
    font-size: 0.95rem;
  }
  .sp-note {
    font-size: 0.9375rem;
    color: var(--fg-2);
  }
  .warnt {
    color: #e0b341;
  }
  .m {
    color: var(--fg-2);
    font-size: 0.8rem;
  }
  .sp-checks {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }
  .sp-checks td,
  .sp-checks th {
    border-bottom: 1px solid var(--hair);
    padding: 0.35rem 0.4rem;
    text-align: left;
    vertical-align: top;
  }
  .sp-checks td:nth-child(2),
  .sp-checks td:nth-child(3) {
    font-family: var(--lp-mono);
    white-space: nowrap;
  }
  .sp-source {
    font: 12px var(--lp-mono);
    color: var(--fg-2);
    margin: 0 0 0.5rem;
  }
  .sp-viewer {
    min-height: 12rem;
  }
  .sp-placeholder {
    border: 1px dashed var(--hair);
    border-radius: 3px;
    padding: 4rem 1rem;
    text-align: center;
    color: var(--fg-2);
  }
  .sp-readout {
    min-height: 5.5rem;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.4rem 1rem;
    margin: 0.75rem 0;
    font-size: 0.85rem;
  }
  .sp-readout dt {
    color: var(--fg-2);
    font-size: 0.75rem;
  }
  .sp-readout dd {
    margin: 0;
    font-family: var(--lp-mono);
    color: var(--fg);
  }
  .sp-controls label {
    display: grid;
    grid-template-columns: 9rem 1fr 3.5rem;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--fg);
    margin: 0.3rem 0;
  }
  .sp-controls label.chk {
    display: flex;
  }
  .sp-controls select,
  .sp-controls input[type='number'] {
    grid-column: 2 / 4;
    background: var(--card);
    color: var(--fg);
    border: 1px solid var(--hair);
    border-radius: 3px;
    padding: 0.25rem 0.4rem;
    font: 13px var(--lp-mono);
    min-width: 0;
  }
  .sp-controls input[type='range'] {
    accent-color: var(--lp-accent, #e5482a);
    height: 24px;
    min-width: 0;
  }
  .sp-controls input.seed {
    grid-column: 2 / 3;
    width: 6rem;
  }
  .sp-pause {
    margin-left: 0.5rem;
    background: none;
    border: 1px solid var(--fg-2);
    border-radius: 3px;
    color: var(--fg);
    padding: 0.15rem 0.5rem;
    font: 12px var(--lp-sans, sans-serif);
    cursor: pointer;
    min-height: 24px;
  }
  .sp-tabbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-bottom: 1px solid var(--hair);
    margin-bottom: 0.6rem;
  }
  .sp-checks caption {
    text-align: left;
    color: var(--fg-2);
    padding-bottom: 0.4rem;
  }
  .sp-controls .v {
    font: 12px var(--lp-mono);
    color: var(--fg-2);
    text-align: right;
  }
  .sp-full {
    margin-left: auto;
    background: none;
    border: 1px solid var(--hair);
    border-radius: 3px;
    color: var(--fg);
    padding: 0.3rem 0.6rem;
    font-size: 0.82rem;
    cursor: pointer;
  }
  .sp-full:hover {
    border-color: var(--fg-2);
  }
  .sp-frame {
    position: relative;
    height: min(46rem, 80vh);
    border: 1px solid var(--hair);
    border-radius: 3px;
    overflow: hidden;
  }
  .sp-frame iframe {
    width: 100%;
    height: 100%;
    border: 0;
    display: block;
  }
  .sp-frame-loading {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: var(--fg-2);
    font: 13px var(--lp-mono);
  }
  /* While the embedded editor is shown: the shell's grid becomes one column, app first. */
  :global(.rs-shell-grid.rs-editing) {
    grid-template-columns: minmax(0, 1fr) !important;
  }
  :global(.rs-shell-grid.rs-editing .rs-shell-app) {
    order: -1;
    position: static !important;
    max-height: none !important;
    overflow: visible !important;
  }
</style>
