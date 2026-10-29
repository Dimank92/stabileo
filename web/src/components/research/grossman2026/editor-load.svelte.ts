/** One automatic attempt per page; failures stay visible until an explicit retry. */
export class EditorLoad {
  attempted = $state(false);
  loading = $state(false);
  src = $state<string | null>(null);
  error = $state<string | null>(null);
  async load(encode: () => Promise<string>, isCurrent: () => boolean) {
    if (this.loading) return;
    this.attempted = true;
    this.loading = true;
    this.error = null;
    try {
      const url = await encode();
      if (isCurrent()) this.src = url;
      else {
        this.loading = false;
        this.error = 'The experiment changed while the editor was loading. Retry to load the current network.';
      }
    } catch (e) {
      this.error = `The Stabileo editor could not be loaded: ${e instanceof Error ? e.message : String(e)}`;
      this.loading = false;
    }
  }
}
