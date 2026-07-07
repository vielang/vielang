import { translations } from './translations';

// Must match LangContext's LANG_KEY. Duplicating to avoid importing a
// 'use client' module from non-React contexts.
const LANG_KEY = 'vinarounding_lang';
type Lang = 'KR' | 'VN' | 'EN';
const VALID: Lang[] = ['KR', 'VN', 'EN'];

/**
 * Translation lookup for code that runs outside React (api-helpers,
 * non-component utilities). Reads the user's current language from
 * localStorage — same source LangContext uses. Falls back to KR (the
 * default in LangContext) on server / when storage isn't accessible.
 *
 * For React components prefer `useLang().t(...)`.
 */
export function tGlobal(key: string): string {
  let lang: Lang = 'KR';
  if (typeof window !== 'undefined') {
    try {
      const saved = window.localStorage.getItem(LANG_KEY);
      if (saved && VALID.includes(saved as Lang)) lang = saved as Lang;
    } catch {
      // localStorage unavailable (privacy mode etc.) — keep default.
    }
  }
  return (translations as any)[lang]?.[key] || key;
}
