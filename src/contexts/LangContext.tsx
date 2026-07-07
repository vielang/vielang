'use client';

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { translations } from '@/lib/translations';

const LANG_KEY = 'vielang_lang';

// VN + EN only. KR support dropped 2026-07-02 after the pivot audit —
// legacy strings stripped from translations.ts, no UI selector exposes it.
export type Lang = 'VN' | 'EN';

const VALID_LANGS: Lang[] = ['VN', 'EN'];

interface LangContextType {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (key: string) => string;
}

// Safe default so `const { lang } = useLang()` never throws when a component
// somehow renders outside <LangProvider> — Turbopack HMR blips, an accidental
// refactor, React 19 concurrent Suspense — the app degrades to EN + no-op
// setLang instead of blowing up SSR. The provider always overrides this.
const DEFAULT_LANG_CTX: LangContextType = {
  lang: 'EN',
  setLang: () => {},
  t: (key: string) =>
    (translations as unknown as Record<'EN', Record<string, string>>).EN?.[key] || key,
};

const LangContext = createContext<LangContextType>(DEFAULT_LANG_CTX);

export function LangProvider({ children }: { children: React.ReactNode }) {
  // SSR-safe: render with EN default first, hydrate persisted value on mount.
  // Deliberately a small flash risk in exchange for no hydration mismatch.
  const [lang, setLangState] = useState<Lang>('EN');
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    try {
      const saved = localStorage.getItem(LANG_KEY) as Lang | null;
      if (saved && VALID_LANGS.includes(saved)) setLangState(saved);
    } catch {
      /* localStorage may be disabled — silent fallback */
    }
    setHydrated(true);
  }, []);

  useEffect(() => {
    if (hydrated)
      try {
        localStorage.setItem(LANG_KEY, lang);
      } catch {}
  }, [lang, hydrated]);

  const setLang = useCallback((next: Lang) => setLangState(next), []);
  const t = useCallback(
    (key: string) => (translations as any)[lang]?.[key] || (translations as any).EN?.[key] || key,
    [lang],
  );

  return <LangContext.Provider value={{ lang, setLang, t }}>{children}</LangContext.Provider>;
}

export const useLang = () => useContext(LangContext);
