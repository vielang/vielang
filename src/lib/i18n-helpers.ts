export type Lang = 'KR' | 'VN' | 'EN';
export const ALL_LANGS: Lang[] = ['KR', 'VN', 'EN'];

export type Multilingual = { KR: string; VN: string; EN: string };
export type MultilingualArray = { KR: string[]; VN: string[]; EN: string[] };

/**
 * Pick the best available translation for a multilingual field on a row.
 * Tries the requested lang first; falls back through VN → KR → legacy
 * single-field → empty string. Use everywhere we display content the
 * owner/admin entered through the CMS.
 *
 *   pickLang(course, 'name', 'EN')
 *     -> course.nameEN || course.nameVN || course.nameKR || course.name || ''
 */
export function pickLang(obj: any, fieldBase: string, lang: Lang): string {
  if (!obj) return '';
  return (
    obj[`${fieldBase}${lang}`] ||
    obj[`${fieldBase}VN`] ||
    obj[`${fieldBase}KR`] ||
    obj[fieldBase] ||
    ''
  );
}

/**
 * Same fallback chain but for { KR, VN, EN } objects we construct in memory
 * (e.g. the value held by MultilingualInput before saving).
 */
export function pickFromTriple(
  triple: Partial<Multilingual> | null | undefined,
  lang: Lang,
): string {
  if (!triple) return '';
  return triple[lang] || triple.VN || triple.KR || '';
}

export function pickArrayFromTriple(
  triple: Partial<MultilingualArray> | null | undefined,
  lang: Lang,
): string[] {
  if (!triple) return [];
  const arr = triple[lang] || triple.VN || triple.KR || [];
  return Array.isArray(arr) ? arr : [];
}
