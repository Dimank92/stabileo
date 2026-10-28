/**
 * The dictionary entries the public pages read before the application loads: vite.config.ts
 * builds `virtual:public-i18n` from these, and `public-keys.test.ts` holds the pages to them.
 *
 * `tabBar.`: the stores are evaluated before the application registers its full dictionaries
 * (Root.svelte reaches them through `app-mode.ts`), and the model store names the first project
 * then, with `t('tabBar.newStructure')`. Without the tab bar's entries here that name was the
 * key itself: every fresh Basic and PRO session showed `tabBar.newStructure` in the project tab
 * (`e2e/no-raw-i18n-keys.spec.ts`, #719), as the desktop app did in its window title.
 */
export const PUBLIC_KEY_PREFIXES = ['landing.', 'blog.', 'contact.', 'lang.', 'tabBar.'];
