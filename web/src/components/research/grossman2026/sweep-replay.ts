import type { SolveAllResult } from './simulation.svelte';

let cachedFrames: SolveAllResult[] | null = null;

export async function loadSweepFrames(base: string): Promise<SolveAllResult[]> {
  if (cachedFrames) return cachedFrames;
  const url = `${base.replace(/\/?$/, '/')}sweep-frames.json`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`HTTP ${res.status} fetching sweep frames from ${url}`);
  cachedFrames = (await res.json()) as SolveAllResult[];
  return cachedFrames;
}
