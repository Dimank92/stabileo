import type { StandingPage } from './cirsoc';
import type { Block } from '../types';
import type { PublicLocale } from '../../i18n/store.svelte';
import { CHANGELOG, AREA_LABELS, formatChangeDate } from '../../changelog';

function blocks(locale: PublicLocale): Block[] {
  return CHANGELOG.flatMap((entry) => {
    const body = entry.i18n[locale];
    const out: Block[] = [{ k: 'h', t: `${formatChangeDate(entry.date, locale)} · ${body.title}` }];
    if (entry.shot) out.push({ k: 'shot', base: entry.shot.base, w: entry.shot.w, h: entry.shot.h, alt: entry.shot.alt[locale] });
    out.push({ k: 'ul', items: body.items.map((item, i) => `${AREA_LABELS[locale][entry.areas[i]]}: ${item}`) });
    return out;
  });
}

export const changelogPage: StandingPage = {
  path: '/changelog',
  updated: '2026-10-23',
  i18n: {
    es: {
      title: 'Historial de cambios de Stabileo',
      description: 'Lo que llegó a Stabileo cada semana: funciones nuevas del modo Básico y del PRO, mejoras del motor de cálculo y del sitio. Se actualiza cada viernes.',
      eyebrow: 'Historial de cambios',
      lead: 'Lo que llegó a Stabileo cada semana desde junio de 2026, del más reciente al más viejo. Cada viernes sumamos lo que se publicó esa semana en el modo Básico, el PRO, el motor de cálculo y el sitio.',
      blocks: blocks('es'),
      faqTitle: 'Preguntas frecuentes',
      faq: [],
    },
    en: {
      title: 'Stabileo changelog',
      description: 'What reached Stabileo each week: new Basic and PRO features, and improvements to the analysis engine and the website. Updated every Friday.',
      eyebrow: 'Changelog',
      lead: 'What reached Stabileo each week since June 2026, newest first. Every Friday we add what was released that week in Basic, PRO, the analysis engine and the website.',
      blocks: blocks('en'),
      faqTitle: 'Frequently asked questions',
      faq: [],
    },
    pt: {
      title: 'Histórico de mudanças do Stabileo',
      description: 'O que chegou ao Stabileo a cada semana: funções novas do Básico e do PRO, e melhorias do motor de cálculo e do site. Atualizado toda sexta-feira.',
      eyebrow: 'Histórico de mudanças',
      lead: 'O que chegou ao Stabileo a cada semana desde junho de 2026, do mais recente ao mais antigo. Toda sexta-feira somamos o que foi publicado naquela semana no Básico, no PRO, no motor de cálculo e no site.',
      blocks: blocks('pt'),
      faqTitle: 'Perguntas frequentes',
      faq: [],
    },
  },
};
