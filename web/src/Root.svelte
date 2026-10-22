<script lang="ts">
  /**
   * The public pages without the application.
   *
   * The landing, the blog and the standing pages used to be rendered by App.svelte, so every
   * visit to them downloaded and ran the whole application (about 3 MB of JavaScript) and the
   * engine (3 MB of WASM) before the page could settle: Lighthouse measured 6 s of blocked main
   * thread and, on the blog, 19 s to the largest paint. They need none of it.
   *
   * Here they render on their own, and the application is loaded on the first route that needs
   * it: an app route on arrival, or the landing's "enter the app". From then on App.svelte owns
   * every route, public ones included, exactly as before, so nothing changes once it is up.
   */
  import { tick } from 'svelte';
  import LandingPage from './components/LandingPage.svelte';
  import BlogPage from './components/blog/BlogPage.svelte';
  import { parsePublicPath, publicHref } from './lib/i18n/public-routes';
  import { publicI18n, setLocale } from './lib/i18n/store.svelte';
  import { findStandingPage } from './lib/blog/pages';

  // A path 404.html handed over (`/?route=/es/blog/x`), restored as App.svelte does: through
  // the URL builder, with the fragment kept on this address.
  const redirectedRoute = new URLSearchParams(location.search).get('route');
  if (redirectedRoute) {
    const { locale, path } = parsePublicPath(redirectedRoute.split('?')[0]);
    const normalised = locale ? publicHref(path, locale) : redirectedRoute;
    history.replaceState(null, '', normalised + (redirectedRoute.includes('#') ? '' : location.hash));
  }

  /**
   * The fragments `lib/utils/url-sharing.ts` restores a model from (`CODE_HASH` is '#code=';
   * not imported, so the public pages do not load the compression code it brings).
   */
  const SHARED_MODEL = /^#(data=|embed=|code=)/;

  /** The routes App.svelte calls public: the landing, the blog and the standing pages. */
  function route() {
    const { path } = parsePublicPath(location.pathname);
    const params = new URLSearchParams(location.search);
    const p = location.pathname;
    if (params.has('embed') || p === '/app' || p.startsWith('/app/') || p === '/demo' || p === '/demo/') return { kind: 'app' as const, path };
    // A shared model (`/#data=…`, `#embed=…`, `#code=…`) is the application's to restore
    // (`loadFromURLHash`): it used to be mounted behind the landing and read the fragment.
    if (SHARED_MODEL.test(location.hash)) return { kind: 'app' as const, path };
    if (path === '/blog' || path.startsWith('/blog/') || findStandingPage(path)) return { kind: 'blog' as const, path };
    return { kind: 'landing' as const, path };
  }

  function adoptLocaleFromPath() {
    const { locale } = parsePublicPath(location.pathname);
    if (locale && locale !== publicI18n.locale) setLocale(locale);
  }
  adoptLocaleFromPath();

  let current = $state(route());
  /** Once loaded, the application owns the page. */
  let app = $state<typeof import('./App.svelte').default | null>(null);
  let loading: Promise<void> | null = null;

  function loadApp() {
    loading ??= import('./App.svelte').then(async (m) => {
      app = m.default;
      // Tell main.ts the application is up: it installs the browser-test hooks then (in an
      // e2e build only), so that "the hooks are there" still means "the application is up".
      await tick();
      window.dispatchEvent(new CustomEvent('stabileo-app-ready'));
    });
    return loading;
  }

  function sync() {
    adoptLocaleFromPath();
    current = route();
    if (current.kind === 'app') void loadApp();
  }
  if (route().kind === 'app') void loadApp();

  // Until the application is up, the public pages' own navigation comes through here
  // (`landing-utils.ts`); App.svelte listens for the same events once it is mounted.
  function onNavigate(e: Event) {
    if (app) return;
    const path = (e as CustomEvent<string>).detail;
    if (typeof path !== 'string') return;
    history.pushState(null, '', publicHref(path, publicI18n.locale));
    sync();
  }
  function onEnterApp() {
    if (app) return;
    // App.svelte decides its mode from the address, so the address goes first: the one its own
    // enterApp() pushes from a public page (`modeToPath('basico')`).
    if (route().kind !== 'app') history.pushState(null, '', '/app/basic');
    sync();
  }
  function onPopState() {
    if (!app) sync();
  }
  $effect(() => {
    window.addEventListener('stabileo-navigate', onNavigate);
    window.addEventListener('stabileo-enter-app', onEnterApp);
    window.addEventListener('popstate', onPopState);
    return () => {
      window.removeEventListener('stabileo-navigate', onNavigate);
      window.removeEventListener('stabileo-enter-app', onEnterApp);
      window.removeEventListener('popstate', onPopState);
    };
  });
</script>

{#if app}
  {@const App = app}
  <App />
{:else if current.kind === 'blog'}
  <BlogPage path={current.path} />
{:else if current.kind === 'landing'}
  <LandingPage />
{/if}
