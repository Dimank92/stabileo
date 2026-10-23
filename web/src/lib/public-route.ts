/**
 * Which kind of page an address is: the application, the blog (and the standing pages), or the
 * landing. main.ts reads it to mount the application directly on an app route, and Root.svelte to
 * render the public pages without it, so both decide the same way.
 */
import { parsePublicPath, publicHref } from './i18n/public-routes';
import { findStandingPage } from './blog/pages';

/**
 * The fragments `utils/url-sharing.ts` restores a model from (`CODE_HASH` is '#code='; not
 * imported, so the public pages do not load the compression code it brings).
 */
const SHARED_MODEL = /^#(data=|embed=|code=)/;

export type RouteKind = 'app' | 'blog' | 'landing';

export function routeKind(loc: Pick<Location, 'pathname' | 'search' | 'hash'> = location): { kind: RouteKind; path: string } {
  const { path } = parsePublicPath(loc.pathname);
  const params = new URLSearchParams(loc.search);
  const p = loc.pathname;
  if (params.has('embed') || p === '/app' || p.startsWith('/app/') || p === '/demo' || p === '/demo/') return { kind: 'app', path };
  // A shared model (`/#data=…`, `#embed=…`, `#code=…`) is the application's to restore
  // (`loadFromURLHash`): it used to be mounted behind the landing and read the fragment.
  if (SHARED_MODEL.test(loc.hash)) return { kind: 'app', path };
  if (path === '/blog' || path.startsWith('/blog/') || findStandingPage(path)) return { kind: 'blog', path };
  return { kind: 'landing', path };
}

/**
 * A path 404.html handed over (`/?route=/es/blog/x`), restored as App.svelte does: through the
 * URL builder, with the fragment kept on this address. Before anything reads the route.
 */
export function restoreRedirectedRoute(): void {
  const redirectedRoute = new URLSearchParams(location.search).get('route');
  if (!redirectedRoute) return;
  const { locale, path } = parsePublicPath(redirectedRoute.split('?')[0]);
  const normalised = locale ? publicHref(path, locale) : redirectedRoute;
  history.replaceState(null, '', normalised + (redirectedRoute.includes('#') ? '' : location.hash));
}
