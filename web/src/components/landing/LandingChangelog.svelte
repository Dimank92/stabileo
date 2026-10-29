<script lang="ts">
  import { tPublic as t, tpPublic as tp, publicI18n } from '../../lib/i18n/store.svelte';
  import { LATEST_CHANGES, AREA_LABELS, entryAreas, formatChangeDate, itemsByArea } from '../../lib/changelog';
  import PublicLink from './PublicLink.svelte';
  import Shot from './Shot.svelte';

  const locale = $derived(publicI18n.locale);
  const [latest, ...earlier] = LATEST_CHANGES;
</script>

<section class="sec sec--paper changelog reveal" data-section="changelog" id="changelog" aria-labelledby="changelog-title">
  <div class="wrap">
    <div class="changelog-top">
      <div>
        <p class="eyebrow">
          <span class="eyebrow-rule" aria-hidden="true"></span>
          <span class="eyebrow-label">{t('landing.changelogEyebrow')}</span>
        </p>
        <h2 id="changelog-title" class="display">{t('landing.changelogTitle')}</h2>
        <p class="lead">{t('landing.changelogBody')}</p>
      </div>
      <PublicLink to="/changelog" class="btn btn-primary changelog-all">{t('landing.changelogLink')}</PublicLink>
    </div>

    <div class="changelog-grid">
      <article class="changelog-feature" data-testid="changelog-week">
        <p class="changelog-kicker">
          <span class="changelog-new">{t('landing.changelogLatest')}</span>
          <time datetime={latest.date}>{formatChangeDate(latest.date, locale)}</time>
        </p>
        <h3 class="changelog-title">{latest.i18n[locale].title}</h3>
        {#each itemsByArea(latest, locale) as g (g.area)}
          <div class="changelog-group">
            <h4 class="changelog-area">{AREA_LABELS[locale][g.area]}</h4>
            <ul class="changelog-items">
              {#each g.items as item, i (i)}<li>{item}</li>{/each}
            </ul>
          </div>
        {/each}
        {#if latest.shot}
          <figure class="changelog-shot">
            <Shot base={latest.shot.base} w={latest.shot.w} h={latest.shot.h} alt={latest.shot.alt[locale]} sizes="(max-width: 900px) 92vw, 55vw" />
          </figure>
        {/if}
      </article>

      <div class="changelog-side">
        {#each earlier as entry (entry.date)}
          {@const n = entry.areas.length}
          <article class="changelog-mini" data-testid="changelog-week">
            {#if entry.shot}
              <figure class="changelog-thumb">
                <Shot base={entry.shot.base} w={entry.shot.w} h={entry.shot.h} alt={entry.shot.alt[locale]} sizes="(max-width: 900px) 92vw, 30vw" />
              </figure>
            {/if}
            <div class="changelog-mini-body">
              <time class="changelog-date" datetime={entry.date}>{formatChangeDate(entry.date, locale)}</time>
              <h3 class="changelog-title changelog-title--mini">{entry.i18n[locale].title}</h3>
              <p class="changelog-meta">
                {#each entryAreas(entry) as a (a)}<span class="changelog-tag">{AREA_LABELS[locale][a]}</span>{/each}
                <span class="changelog-count">{n === 1 ? t('changelog.changesOne') : tp('changelog.changes', { n })}</span>
              </p>
            </div>
          </article>
        {/each}
      </div>
    </div>
  </div>
</section>

<style>
  .changelog-top {
    display: grid;
    gap: 1.5rem;
    align-items: end;
  }

  @media (min-width: 900px) {
    .changelog-top {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 3rem;
    }
  }

  .changelog-top :global(.changelog-all) {
    justify-self: start;
  }

  .changelog-grid {
    display: grid;
    gap: 1.25rem;
    margin-top: clamp(1.75rem, 4vw, 3rem);
  }

  @media (min-width: 900px) {
    .changelog-grid {
      grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
      align-items: start;
    }
  }

  .changelog-feature,
  .changelog-mini {
    background: var(--card);
    border: 1px solid var(--card-hair);
    border-radius: calc(var(--lp-radius) * 2);
  }

  .changelog-feature {
    padding: clamp(1.25rem, 3vw, 2rem);
    border-top: 3px solid var(--lp-accent);
  }

  .changelog-kicker {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
    margin: 0;
    font-family: var(--lp-mono);
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    color: var(--fg-2);
  }

  .changelog-new {
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    background: var(--lp-accent);
    color: #fff;
    font-size: 0.66rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .changelog-title {
    margin: 0.6rem 0 0;
    font-family: var(--lp-display);
    font-size: clamp(1.25rem, 2.2vw, 1.5rem);
    font-weight: 600;
    line-height: 1.25;
  }

  .changelog-title--mini {
    margin-top: 0.35rem;
    font-size: 1.08rem;
    line-height: 1.3;
  }

  .changelog-group {
    margin-top: 1.25rem;
  }

  .changelog-area {
    margin: 0 0 0.35rem;
    font-family: var(--lp-mono);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--accent-text);
  }

  .changelog-items {
    margin: 0;
    padding-left: 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.95rem;
    line-height: 1.55;
  }

  .changelog-items li::marker {
    color: var(--fg-2);
  }

  .changelog-shot {
    margin: 1.5rem 0 0;
  }

  .changelog-side {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .changelog-mini {
    overflow: hidden;
  }

  .changelog-thumb {
    margin: 0;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    border-bottom: 1px solid var(--card-hair);
  }

  .changelog-thumb :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: top center;
  }

  .changelog-shot :global(img) {
    display: block;
    width: 100%;
    height: auto;
    border: 1px solid var(--hair);
    border-radius: var(--lp-radius);
  }

  .changelog-mini-body {
    padding: 1.1rem 1.25rem 1.2rem;
  }

  .changelog-date {
    font-family: var(--lp-mono);
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    color: var(--fg-2);
  }

  .changelog-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin: 0.75rem 0 0;
  }

  .changelog-tag {
    font-family: var(--lp-mono);
    font-size: 0.64rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--fg-2);
    border: 1px solid var(--hair);
    border-radius: 999px;
    padding: 0.15rem 0.55rem;
  }

  .changelog-count {
    margin-left: auto;
    font-size: 0.8rem;
    color: var(--fg-2);
  }
</style>
