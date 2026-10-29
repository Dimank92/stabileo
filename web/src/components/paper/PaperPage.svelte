<script lang="ts">
  /**
   * /paper: the JAIE 2026 paper in full, set as the paper is (`lib/paper/jaie-2026.ts`), with its
   * PDF. Its own chunk (Root.svelte, App.svelte), so KaTeX is not loaded by the landing or the blog.
   */
  import { applyPageMeta, restorePageMeta } from '../../lib/page-meta';
  import { publicI18n } from '../../lib/i18n/store.svelte';
  import SiteHeader from '../site/SiteHeader.svelte';
  import LandingFooter from '../landing/LandingFooter.svelte';
  import PaperText from './PaperText.svelte';
  import { PAPER, BODY, REFERENCES, LABELS, PDF_HREF } from '../../lib/paper/jaie-2026';
  import { ps } from './i18n';
  import katex from 'katex';
  import '../landing/landing.css';
  import './paper.css';

  const s = $derived(ps());
  $effect(() => {
    applyPageMeta({ title: s.metaTitle, description: s.metaDescription, locale: publicI18n.locale, path: '/paper' });
    return () => restorePageMeta();
  });
  const display = (tex: string) => katex.renderToString(tex, { displayMode: true, throwOnError: false, output: 'htmlAndMathml' });
  const url = (r: string) => r.match(/https?:\/\/\S+$/)?.[0];
</script>

<div class="landing paper-site">
  <SiteHeader current="paper" />
  <main class="pp-wrap">
    <div class="pp-bar">
      <p class="pp-venue">{s.venue}{#if s.language}<br /><span class="pp-lang">{s.language}</span>{/if}</p>
      <a class="pp-download" href={PDF_HREF} download data-testid="paper-download">{s.download}</a>
    </div>

    <article class="pp-sheet" lang="es" data-testid="paper">
      <header class="pp-front">
        <h1 class="pp-title">{PAPER.title}</h1>
        <p class="pp-authors">
          {#each PAPER.authors as a, i (a.email)}{#if i > 0}{' '}{/if}{a.name}&nbsp;<sup>({a.marks})</sup>{/each}
        </p>
        {#each PAPER.affiliations as [mark, name] (mark)}<p class="pp-affiliation">({mark}) {name}</p>{/each}
        <p class="pp-emails">{PAPER.authors.map((a) => a.email).join(' · ')}</p>
      </header>

      <p class="pp-keywords"><strong>{LABELS.keywords}</strong> {PAPER.keywords}</p>
      <section class="pp-abstract">
        <h2 class="pp-abstract-h">{LABELS.resumen}</h2>
        <p>{PAPER.resumen}</p>
      </section>
      <section class="pp-abstract" lang="en">
        <h2 class="pp-abstract-h">{LABELS.abstract}</h2>
        <p><em>{PAPER.abstract}</em></p>
      </section>

      {#each BODY as b, i (i)}
        {#if b.k === 'h1'}
          <h2 class="pp-h1" id="sec-{b.n.replace(/\.$/, '')}"><span class="pp-num">{b.n}</span> <PaperText t={b.t} /></h2>
        {:else if b.k === 'h2'}
          <h3 class="pp-h2" id="sec-{b.n.replace(/\.$/, '')}"><span class="pp-num">{b.n}</span> <PaperText t={b.t} /></h3>
        {:else if b.k === 'p'}
          <p><PaperText t={b.t} /></p>
        {:else if b.k === 'ol'}
          <ol class="pp-list">{#each b.items as item, j (j)}<li><PaperText t={item} /></li>{/each}</ol>
        {:else if b.k === 'eq'}
          <div class="pp-eq" id="eq-{b.n}">
            <div class="pp-eq-body">{@html display(b.tex)}</div>
            <span class="pp-eq-n">({b.n})</span>
          </div>
        {:else if b.k === 'table'}
          <figure class="pp-table" id="tabla-{b.n}">
            <div class="pp-table-scroll">
              <table>
                <thead><tr>{#each b.head as h, j (j)}<th><PaperText t={h} /></th>{/each}</tr></thead>
                <tbody>{#each b.rows as row, r (r)}<tr>{#each row as cell, j (j)}<td><PaperText t={cell} /></td>{/each}</tr>{/each}</tbody>
              </table>
            </div>
            <figcaption>{LABELS.table} {b.n}. {b.caption}</figcaption>
          </figure>
        {:else if b.k === 'figure'}
          <figure class="pp-figure" id="figura-{b.n}">
            <img src={b.src} width={b.width} height={b.height} alt={b.caption} loading="lazy" decoding="async" />
            <figcaption>{LABELS.figure} {b.n}. {b.caption}</figcaption>
          </figure>
        {/if}
      {/each}

      <h2 class="pp-h1 pp-unnumbered">{LABELS.references}</h2>
      <ol class="pp-refs">
        {#each REFERENCES as r, i (i)}
          {@const link = url(r)}
          <li id="ref-{i + 1}">{#if link}{r.slice(0, r.length - link.length)}<a href={link} rel="noopener">{link}</a>{:else}{r}{/if}</li>
        {/each}
      </ol>

      <h2 class="pp-h1 pp-unnumbered">{LABELS.authors}</h2>
      <div class="pp-author-data">
        {#each PAPER.authors as a (a.email)}
          <p><strong>{a.name}</strong><br />{a.affiliation}<br /><a href="mailto:{a.email}">{a.email}</a></p>
        {/each}
      </div>
    </article>
  </main>
  <LandingFooter />
</div>
