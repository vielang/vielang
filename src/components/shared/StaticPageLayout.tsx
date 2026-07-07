'use client';

import { useLang } from '@/contexts';

type Text = { vn: string; en: string };

export interface StaticPageSection {
  heading?: Text;
  paragraphs?: Text[];
  bullets?: Text[];
}

interface Props {
  title: Text;
  intro?: Text;
  updated?: string;
  sections: StaticPageSection[];
}

/**
 * Renders a legal / support page from a structured content model. Copy lives
 * inline in the page file (see /terms, /privacy, etc.) — no runtime fetch, no
 * dangerouslySetInnerHTML, no HTML sanitization step needed. Simpler for the
 * caller to reason about and cheaper than the old CMS fetch path.
 *
 * The layout picks VN or EN per key based on `useLang()`. Missing translations
 * fall through to the other language rather than an empty gap so a partial
 * translation never leaves the page half-blank.
 */
export function StaticPageLayout({ title, intro, updated, sections }: Props) {
  const { lang } = useLang();
  const pick = (t: Text) => (lang === 'VN' ? t.vn || t.en : t.en || t.vn);

  return (
    <article className="mx-auto max-w-3xl space-y-8 px-4 py-10 md:py-16">
      <header className="space-y-2">
        <h1 className="text-brand dark:text-accent-warm font-serif text-3xl font-bold md:text-4xl">
          {pick(title)}
        </h1>
        {updated && (
          <p className="text-xs text-slate-500 tabular-nums dark:text-slate-400">
            {lang === 'VN' ? 'Cập nhật lần cuối' : 'Last updated'}: {updated}
          </p>
        )}
        {intro && (
          <p className="max-w-2xl pt-2 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
            {pick(intro)}
          </p>
        )}
      </header>

      <div className="space-y-8">
        {sections.map((section, i) => (
          <section key={i} className="space-y-3">
            {section.heading && (
              <h2 className="font-serif text-xl font-semibold text-slate-900 dark:text-slate-100">
                {pick(section.heading)}
              </h2>
            )}
            {section.paragraphs?.map((p, j) => (
              <p key={j} className="text-sm leading-relaxed text-slate-700 dark:text-slate-300">
                {pick(p)}
              </p>
            ))}
            {section.bullets && section.bullets.length > 0 && (
              <ul className="list-disc space-y-1.5 pl-5 text-sm leading-relaxed text-slate-700 dark:text-slate-300">
                {section.bullets.map((b, k) => (
                  <li key={k}>{pick(b)}</li>
                ))}
              </ul>
            )}
          </section>
        ))}
      </div>
    </article>
  );
}
