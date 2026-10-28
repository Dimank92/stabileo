/**
 * The dictionary entries the public pages read before the application loads: vite.config.ts
 * builds `virtual:public-i18n` from these, and `public-keys.test.ts` holds the pages to them.
 * `tabBar.newStructure` is here because the model store names its first model when it is
 * imported, before the application registers the rest.
 */
export const PUBLIC_KEY_PREFIXES = ['landing.', 'blog.', 'contact.', 'lang.', 'tabBar.newStructure'];
