import { expect, it, vi } from 'vitest';
import { EditorLoad } from '../editor-load.svelte';

it('attempts a rejected encoder once and preserves the error until manual retry', async () => {
  const editor = new EditorLoad();
  const encode = vi.fn().mockRejectedValue(new Error('raw-deflate unavailable'));
  // Repeated evaluations of the page's automatic-load gate must not retry a rejection.
  for (let i = 0; i < 10; i++) if (!editor.attempted) await editor.load(encode, () => true);
  expect(encode).toHaveBeenCalledTimes(1);
  expect(editor.error).toContain('raw-deflate unavailable');
  expect(editor.loading).toBe(false);
  encode.mockResolvedValue('/editor');
  await editor.load(encode, () => true);
  expect(encode).toHaveBeenCalledTimes(2);
  expect(editor.error).toBeNull();
  expect(editor.src).toBe('/editor');
});
it('does not install a URL for an obsolete simulation', async () => {
  const editor = new EditorLoad();
  await editor.load(async () => '/obsolete', () => false);
  expect(editor.src).toBeNull();
  expect(editor.loading).toBe(false);
  expect(editor.attempted).toBe(true);
});
