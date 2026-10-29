import { publicI18n, type PublicLocale } from '../../lib/i18n/store.svelte';

/** The page around the paper; the paper itself is in Spanish, as written. */
type PaperStrings = { venue: string; download: string; language: string; metaTitle: string; metaDescription: string };

const S: Record<'es' | 'en', PaperStrings> & Partial<Record<PublicLocale, PaperStrings>> = {
  es: {
    venue: 'Trabajo presentado en las 29.ª Jornadas Argentinas de Ingeniería Estructural (JAIE 2026).',
    download: 'Descargar PDF',
    language: '',
    metaTitle: 'Desarrollo de un agente estructural de cálculo sobre un solver verificado — Stabileo',
    metaDescription: 'El paper completo: un agente de IA confiable para cálculo estructural opera como orquestador sobre un solver verificado, sin generar resultados numéricos por cuenta propia.',
  },
  en: {
    venue: 'Paper presented at the 29th Argentine Structural Engineering Conference (JAIE 2026).',
    download: 'Download PDF',
    language: 'The paper is written in Spanish; its abstract is in English.',
    metaTitle: 'Developing a structural calculation agent on a verified solver (paper, in Spanish) — Stabileo',
    metaDescription: 'The full paper: an AI agent for structural calculation is reliable only if it orchestrates a verified solver and never produces a number on its own.',
  },
  pt: {
    venue: 'Trabalho apresentado nas 29.ª Jornadas Argentinas de Engenharia Estrutural (JAIE 2026).',
    download: 'Baixar PDF',
    language: 'O artigo está escrito em espanhol; o resumo em inglês está incluído.',
    metaTitle: 'Desenvolvimento de um agente estrutural de cálculo sobre um solver verificado (artigo, em espanhol) — Stabileo',
    metaDescription: 'O artigo completo: um agente de IA para cálculo estrutural só é confiável se orquestra um solver verificado e nunca produz um número por conta própria.',
  },
  de: {
    venue: 'Vorgestellt auf den 29. Argentinischen Tagen des Konstruktiven Ingenieurbaus (JAIE 2026).',
    download: 'PDF herunterladen',
    language: 'Der Fachartikel ist auf Spanisch verfasst; die Zusammenfassung liegt auch auf Englisch vor.',
    metaTitle: 'Ein Agent für statische Berechnungen auf einem verifizierten Solver (Fachartikel, auf Spanisch) — Stabileo',
    metaDescription: 'Der vollständige Fachartikel: Ein KI-Agent für statische Berechnungen ist nur verlässlich, wenn er einen verifizierten Solver orchestriert und selbst keine Zahl erzeugt.',
  },
  fr: {
    venue: 'Article présenté aux 29es Journées argentines de génie des structures (JAIE 2026).',
    download: 'Télécharger le PDF',
    language: 'L’article est rédigé en espagnol ; son résumé est aussi en anglais.',
    metaTitle: 'Un agent de calcul de structures sur un solveur vérifié (article, en espagnol) — Stabileo',
    metaDescription: 'L’article complet : un agent d’IA pour le calcul de structures n’est fiable que s’il orchestre un solveur vérifié sans produire lui-même de chiffres.',
  },
  tr: {
    venue: '29. Arjantin Yapı Mühendisliği Günleri’nde (JAIE 2026) sunulan bildiri.',
    download: 'PDF’yi indir',
    language: 'Makale İspanyolca yazılmıştır; özeti İngilizce olarak da yer alır.',
    metaTitle: 'Doğrulanmış bir çözücü üzerinde yapısal hesap ajanı (makale, İspanyolca) — Stabileo',
    metaDescription: 'Makalenin tamamı: yapısal hesap için bir yapay zekâ ajanı, ancak doğrulanmış bir çözücüyü yönetir ve kendisi hiçbir sayı üretmezse güvenilirdir.',
  },
};

export function ps(): PaperStrings {
  return S[publicI18n.locale] ?? S.en;
}
