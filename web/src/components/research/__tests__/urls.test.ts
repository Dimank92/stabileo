import { expect, it } from 'vitest';
import { isHttpUrl } from '../urls';

it('only permits absolute HTTP(S) gallery links', () => {
  for (const url of ['https://github.com/example', 'http://example.com', 'HTTPS://gitlab.com/example']) {
    expect(isHttpUrl(url)).toBe(true);
  }
  for (const url of ['javascript:alert(1)', 'data:text/html,test', '//github.com/example', '/relative', '', 'https://']) {
    expect(isHttpUrl(url)).toBe(false);
  }
});
