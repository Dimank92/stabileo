import { describe, it, expect } from 'vitest';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import {
  CHANGELOG,
  CHANGELOG_AREAS,
  LATEST_CHANGES,
  AREA_LABELS,
  changeDay,
  changelogByMonth,
  entryAreas,
  formatChangeDate,
  formatChangeMonth,
  itemsByArea,
} from '..';
import { changelogPage } from '../../blog/pages/changelog';

const LOCALES = ['es', 'en', 'pt'] as const;
const SCREENSHOTS = resolve(__dirname, '../../../../public/screenshots');

describe('the changelog', () => {
  it('runs newest first, one entry per weekly report, each dated on a Friday', () => {
    const dates = CHANGELOG.map((e) => e.date);
    expect([...dates].sort().reverse()).toEqual(dates);
    expect(new Set(dates).size).toBe(dates.length);
    for (const date of dates) {
      expect(date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      const [y, m, d] = date.split('-').map(Number);
      expect(new Date(Date.UTC(y, m - 1, d)).getUTCDay(), date).toBe(5);
    }
  });

  it('says the same in every language: one item per area, with a title and no empty line', () => {
    for (const entry of CHANGELOG) {
      expect(entry.areas.length, entry.date).toBeGreaterThan(0);
      for (const area of entry.areas) expect(CHANGELOG_AREAS).toContain(area);
      for (const l of LOCALES) {
        const body = entry.i18n[l];
        expect(body.title.trim(), `${entry.date} ${l}`).not.toBe('');
        expect(body.items, `${entry.date} ${l}`).toHaveLength(entry.areas.length);
        for (const item of body.items) expect(item.trim().length, `${entry.date} ${l}`).toBeGreaterThan(10);
      }
    }
  });

  it('follows the editorial style: no dashes as asides and no angle quotes', () => {
    const text = JSON.stringify(CHANGELOG);
    expect(text).not.toMatch(/—/);
    expect(text).not.toMatch(/[«»]/);
  });

  it('points only at screenshots that exist, at both widths and in both formats, with alt text', () => {
    for (const entry of CHANGELOG) {
      if (!entry.shot) continue;
      for (const w of [800, 1600]) {
        for (const ext of ['avif', 'webp']) {
          const file = `${entry.shot.base}-${w}.${ext}`;
          expect(existsSync(resolve(SCREENSHOTS, file)), file).toBe(true);
        }
      }
      for (const l of LOCALES) expect(entry.shot.alt[l].trim(), `${entry.date} ${l}`).not.toBe('');
    }
  });

  it('shows the three newest weeks on the landing', () => {
    expect(LATEST_CHANGES).toEqual(CHANGELOG.slice(0, 3));
  });

  it('labels every area in every language', () => {
    for (const l of LOCALES) for (const area of CHANGELOG_AREAS) expect(AREA_LABELS[l][area].trim()).not.toBe('');
  });

  it('formats a calendar day without moving it across time zones', () => {
    expect(formatChangeDate('2026-10-23', 'es')).toBe('23 de octubre de 2026');
    expect(formatChangeDate('2026-10-23', 'en')).toBe('October 23, 2026');
  });
});

describe('the changelog page', () => {
  it('is dated with the newest entry, so the sitemap moves with it', () => {
    expect(changelogPage.updated).toBe(CHANGELOG[0].date);
  });

  it('is drawn by its own view, with no article blocks or questions to keep in step', () => {
    expect(changelogPage.view).toBe('changelog');
    for (const l of LOCALES) {
      expect(changelogPage.i18n[l].blocks).toEqual([]);
      expect(changelogPage.i18n[l].faq).toEqual([]);
    }
  });
});

describe('the changelog helpers', () => {
  const entry = CHANGELOG.find((e) => e.date === '2026-09-25')!;

  it('groups the items under their areas, in the order the areas first appear, losing none', () => {
    expect(entryAreas(entry)).toEqual(['site', 'basic', 'edu', 'pro', 'engine']);
    for (const l of LOCALES) {
      const groups = itemsByArea(entry, l);
      expect(groups.map((g) => g.area)).toEqual(entryAreas(entry));
      expect(groups.flatMap((g) => g.items)).toEqual(entry.i18n[l].items);
      expect(groups[1].items).toEqual(entry.i18n[l].items.slice(2, 5));
    }
  });

  it('splits the history into months, newest first, keeping every week', () => {
    const months = changelogByMonth(CHANGELOG);
    expect(months.map((m) => m.key)).toEqual(['2026-10', '2026-09', '2026-08', '2026-07', '2026-06']);
    expect(months.flatMap((m) => m.entries)).toEqual(CHANGELOG);
    expect(months[0].entries.map((e) => e.date)).toEqual(['2026-10-23', '2026-10-16', '2026-10-09', '2026-10-02']);
  });

  it('names the day and the month for the timeline', () => {
    expect(changeDay('2026-10-02', 'es')).toEqual({ day: '2', month: 'oct' });
    expect(changeDay('2026-06-12', 'en')).toEqual({ day: '12', month: 'Jun' });
    expect(formatChangeMonth('2026-10-01', 'es')).toBe('Octubre de 2026');
    expect(formatChangeMonth('2026-10-01', 'en')).toBe('October 2026');
    expect(formatChangeMonth('2026-10-01', 'pt')).toBe('Outubro de 2026');
  });
});
