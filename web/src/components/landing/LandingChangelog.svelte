<script lang="ts">
  import { tPublic as t, tpPublic as tp, publicI18n } from '../../lib/i18n/store.svelte';
  import { LATEST_CHANGES, AREA_LABELS, formatChangeDate } from '../../lib/changelog';
  import PublicLink from './PublicLink.svelte';
  import Shot from './Shot.svelte';

  const SHOWN_ITEMS = 3;
  const locale = $derived(publicI18n.locale);
  const pictured = $derived(LATEST_CHANGES.find((e) => e.shot));
</script>

<section class="sec sec--paper changelog reveal" data-section="changelog" id="changelog" aria-labelledby="changelog-title">
  <div class="wrap">
    <p class="eyebrow">
      <span class="eyebrow-rule" aria-hidden="true"></span>
      <span class="eyebrow-label">{t('landing.changelogEyebrow')}</span>
    </p>
    <h2 id="changelog-title" class="display">{t('landing.changelogTitle')}</h2>
    <p class="lead">{t('landing.changelogBody')}</p>

    <div class="changelog-grid" class:with-shot={!!pictured}>
      <ol class="changelog-list">
        {#each LATEST_CHANGES as entry (entry.date)}
          {@const body = entry.i18n[locale]}
          <li class="changelog-week" data-testid="changelog-week">
            <time class="changelog-date" datetime={entry.date}>{formatChangeDate(entry.date, locale)}</time>
            <h3 class="changelog-title">{body.title}</h3>
            <ul class="changelog-items">
              {#each body.items.slice(0, SHOWN_ITEMS) as item, i (i)}
                <li><span class="changelog-area">{AREA_LABELS[locale][entry.areas[i]]}</span><span>{item}</span></li>
              {/each}
            </ul>
            {#if body.items.length > SHOWN_ITEMS}
              {@const rest = body.items.length - SHOWN_ITEMS}
              <p class="changelog-more">{rest === 1 ? t('landing.changelogMoreOne') : tp('landing.changelogMore', { n: rest })}</p>
            {/if}
          </li>
        {/each}
      </ol>

      {#if pictured?.shot}
        <figure class="changelog-shot" class:portrait={pictured.shot.h > pictured.shot.w}>
          <Shot base={pictured.shot.base} w={pictured.shot.w} h={pictured.shot.h} alt={pictured.shot.alt[locale]} sizes="(max-width: 760px) 92vw, 40vw" />
          <figcaption>{formatChangeDate(pictured.date, locale)} · {pictured.i18n[locale].title}</figcaption>
        </figure>
      {/if}
    </div>

    <div class="changelog-cta">
      <PublicLink to="/changelog" class="btn btn-primary">{t('landing.changelogLink')}</PublicLink>
    </div>
  </div>
</section>

<style>
  .changelog-grid {
    display: grid;
    gap: clamp(1.5rem, 4vw, 3.5rem);
    margin-top: clamp(1.75rem, 4vw, 3rem);
  }

  @media (min-width: 900px) {
    .changelog-grid.with-shot {
      grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
      align-items: start;
    }
  }

  .changelog-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1.75rem;
  }

  .changelog-week {
    padding-left: 1.1rem;
    border-left: 2px solid var(--hair);
  }

  .changelog-week:first-child {
    border-left-color: var(--lp-accent);
  }

  .changelog-date {
    font-family: var(--lp-mono);
    font-size: 0.78rem;
    letter-spacing: 0.04em;
    color: var(--fg-2);
  }

  .changelog-title {
    margin: 0.3rem 0 0.6rem;
    font-family: var(--lp-display);
    font-size: 1.15rem;
    font-weight: 600;
    line-height: 1.3;
  }

  .changelog-items {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    font-size: 0.95rem;
    line-height: 1.5;
    color: var(--fg);
  }

  .changelog-items li {
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr);
    gap: 0.5rem;
    align-items: baseline;
  }

  @media (max-width: 600px) {
    .changelog-items li {
      grid-template-columns: minmax(0, 1fr);
      gap: 0.1rem;
    }
  }

  .changelog-area {
    font-family: var(--lp-mono);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--accent-text);
  }

  .changelog-more {
    margin: 0.45rem 0 0;
    font-size: 0.85rem;
    color: var(--fg-2);
  }

  .changelog-shot {
    margin: 0;
  }

  .changelog-shot.portrait {
    max-width: 280px;
    justify-self: center;
  }

  .changelog-shot :global(img) {
    display: block;
    width: 100%;
    height: auto;
    border: 1px solid var(--hair);
    border-radius: var(--lp-radius);
  }

  .changelog-shot figcaption {
    margin-top: 0.6rem;
    font-size: 0.8rem;
    color: var(--fg-2);
  }

  .changelog-cta {
    margin-top: clamp(1.75rem, 4vw, 2.75rem);
  }
</style>
