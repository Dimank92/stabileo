<script lang="ts">
  /**
   * A small SVG chart: lines, points with optional vertical error bars, a highlighted current
   * point and reference rules. Axes fit every series.
   */
  export type PlotSeries = {
    name: string;
    color: string;
    points: { x: number; y: number; lo?: number; hi?: number }[];
    kind: 'line' | 'points' | 'band';
    dashed?: boolean;
    shape?: 'circle' | 'triangle' | 'square';
  };

  let {
    series,
    xLabel,
    yLabel,
    xDomain = null,
    yDomain = null,
    current = null,
    rules = [],
    height = 240,
    ariaLabel,
  }: {
    series: PlotSeries[];
    xLabel: string;
    yLabel: string;
    xDomain?: [number, number] | null;
    yDomain?: [number, number] | null;
    current?: { x: number; y: number; label: string } | null;
    rules?: { x?: number; y?: number; label: string }[];
    height?: number;
    ariaLabel: string;
  } = $props();

  let width = $state(480);
  const M = { l: 52, r: 12, t: 10, b: 38 };

  function ticks(lo: number, hi: number, n = 5): number[] {
    const raw = (hi - lo) / n;
    const p = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 2.5, 5, 10].map((k) => k * p).find((s) => s >= raw) ?? raw;
    const out: number[] = [];
    for (let v = Math.ceil(lo / step - 1e-9) * step; v <= hi + step * 1e-6; v += step) out.push(Number(v.toPrecision(10)));
    return out;
  }

  const ys = $derived(
    series.flatMap((s) => s.points.flatMap((p) => [p.y, p.lo ?? p.y, p.hi ?? p.y])).concat(current ? [current.y] : []).filter(Number.isFinite),
  );
  const xs = $derived(series.flatMap((s) => s.points.map((p) => p.x)).filter(Number.isFinite));
  const xd = $derived<[number, number]>(xDomain ?? [Math.min(...xs), Math.max(...xs)]);
  const yd = $derived.by<[number, number]>(() => {
    if (yDomain) return yDomain;
    const lo = Math.min(...ys);
    const hi = Math.max(...ys);
    const pad = 0.06 * (hi - lo || 1);
    return [lo - pad, hi + pad];
  });
  const sx = (x: number) => M.l + ((x - xd[0]) / (xd[1] - xd[0])) * (width - M.l - M.r);
  const sy = (y: number) => M.t + (1 - (y - yd[0]) / (yd[1] - yd[0])) * (height - M.t - M.b);
  const fmt = (v: number) => (Math.abs(v) < 1e-12 ? '0' : Number(v.toPrecision(3)).toString());
  const path = (pts: { x: number; y: number }[]) =>
    pts
      .filter((p) => Number.isFinite(p.y))
      .map((p, i) => `${i ? 'L' : 'M'}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`)
      .join('');
  const band = (pts: { x: number; lo?: number; hi?: number }[]) => {
    const up = pts.map((p) => `${sx(p.x).toFixed(1)},${sy(p.hi ?? 0).toFixed(1)}`);
    const down = [...pts].reverse().map((p) => `${sx(p.x).toFixed(1)},${sy(p.lo ?? 0).toFixed(1)}`);
    return `M${up.join('L')}L${down.join('L')}Z`;
  };
</script>

<div class="plot" bind:clientWidth={width}>
  <svg {width} {height} role="img" aria-label={ariaLabel}>
    {#each ticks(yd[0], yd[1]) as t (t)}
      <line x1={M.l} x2={width - M.r} y1={sy(t)} y2={sy(t)} class="grid" />
      <text x={M.l - 6} y={sy(t) + 4} text-anchor="end" class="tick">{fmt(t)}</text>
    {/each}
    {#each ticks(xd[0], xd[1]) as t (t)}
      <text x={sx(t)} y={height - M.b + 15} text-anchor="middle" class="tick">{fmt(t)}</text>
    {/each}
    {#if yd[0] < 0 && yd[1] > 0}<line x1={M.l} x2={width - M.r} y1={sy(0)} y2={sy(0)} class="zero" />{/if}
    {#each rules as r (r.label)}
      {#if r.x !== undefined}
        <line x1={sx(r.x)} x2={sx(r.x)} y1={M.t} y2={height - M.b} class="rule" /><text x={sx(r.x) + 4} y={M.t + 11} class="rule-t">{r.label}</text>
      {:else if r.y !== undefined}
        <line x1={M.l} x2={width - M.r} y1={sy(r.y)} y2={sy(r.y)} class="rule" /><text x={width - M.r - 4} y={sy(r.y) - 4} text-anchor="end" class="rule-t">{r.label}</text>
      {/if}
    {/each}
    {#each series as s (s.name)}
      {#if s.kind === 'band'}
        <path d={band(s.points)} fill={s.color} opacity="0.18" />
      {:else if s.kind === 'line'}
        <path d={path(s.points)} fill="none" stroke={s.color} stroke-width="2" stroke-dasharray={s.dashed ? '5 4' : undefined} />
      {:else}
        {#each s.points as p, i (i)}
          {#if p.lo !== undefined && p.hi !== undefined}
            <line x1={sx(p.x)} x2={sx(p.x)} y1={sy(p.lo)} y2={sy(p.hi)} stroke={s.color} stroke-width="1.2" opacity="0.8" />
          {/if}
          {#if Number.isFinite(p.y)}
            {#if s.shape === 'triangle'}
              <path d="M{sx(p.x)},{sy(p.y) - 5}l4.5,8h-9z" fill={s.color} />
            {:else if s.shape === 'square'}
              <rect x={sx(p.x) - 4} y={sy(p.y) - 4} width="8" height="8" fill="none" stroke={s.color} stroke-width="1.6" />
            {:else}
              <circle cx={sx(p.x)} cy={sy(p.y)} r="3.2" fill={s.color} />
            {/if}
          {/if}
        {/each}
      {/if}
    {/each}
    {#if current && Number.isFinite(current.y)}
      <circle cx={sx(current.x)} cy={sy(current.y)} r="6" fill="none" stroke="var(--fg, #fff)" stroke-width="2" />
    {/if}
    <text x={(M.l + width - M.r) / 2} y={height - 4} text-anchor="middle" class="axis">{xLabel}</text>
    <text x="12" y={(M.t + height - M.b) / 2} text-anchor="middle" class="axis" transform="rotate(-90 12 {(M.t + height - M.b) / 2})">{yLabel}</text>
  </svg>
  <ul class="legend">
    {#each series.filter((s) => s.kind !== 'band') as s (s.name)}
      <li><span class="sw" class:dash={s.dashed} class:pt={s.kind === 'points'} style="--c: {s.color}"></span>{s.name}</li>
    {/each}
    {#if current}<li><span class="sw cur"></span>{current.label}</li>{/if}
  </ul>
</div>

<style>
  .plot {
    width: 100%;
  }
  svg {
    display: block;
    overflow: visible;
  }
  .grid {
    stroke: var(--hair, #333);
    stroke-width: 1;
  }
  .zero {
    stroke: var(--fg-2, #999);
    stroke-width: 1;
  }
  .rule {
    stroke: var(--fg-2, #999);
    stroke-dasharray: 2 3;
  }
  .rule-t,
  .tick {
    fill: var(--fg-2, #999);
    font: 11px var(--lp-mono, monospace);
  }
  .axis {
    fill: var(--fg, #ddd);
    font: 12px var(--lp-mono, monospace);
  }
  .legend {
    list-style: none;
    padding: 0;
    margin: 0.4rem 0 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem 1rem;
    font-size: 0.8rem;
    color: var(--fg-2, #aaa);
  }
  .sw {
    display: inline-block;
    width: 18px;
    height: 0;
    border-top: 2px solid var(--c);
    margin-right: 0.4rem;
    vertical-align: middle;
  }
  .sw.dash {
    border-top-style: dashed;
  }
  .sw.pt {
    width: 8px;
    height: 8px;
    border: none;
    border-radius: 50%;
    background: var(--c);
  }
  .sw.cur {
    width: 10px;
    height: 10px;
    border: 2px solid var(--fg, #fff);
    border-radius: 50%;
  }
</style>
