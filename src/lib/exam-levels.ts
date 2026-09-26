import type { ExamLevel } from "@/lib/exam-types";
import { LANGUAGES, type LanguageConfig } from "@/lib/languages";

/**
 * Kỳ thi và đường dẫn trang danh sách đề của từng kỳ (`/exam/toeic`).
 *
 * Module nhẹ, KHÔNG import dữ liệu đề (lib/exams.ts nạp các file JSON đề) —
 * header dùng nó ở mọi trang để hiện tab kỳ thi. Kỳ thi nằm trên URL chứ
 * không trong state của trang: header mới đổi được, và chia sẻ link / bấm
 * quay lại giữ đúng kỳ thi.
 */
export interface ExamLevelInfo {
  id: ExamLevel;
  slug: string;
  /** Ngôn ngữ của kỳ thi — tầng giữa: Luyện thi → ngôn ngữ → kỳ thi. */
  lang: string;
  /** Kỹ năng và thang điểm — dòng giới thiệu dưới tab. */
  hint: string;
}

// Rỗng thì tab Luyện thi tự ẩn (xem lib/nav.ts).
export const EXAM_LEVELS: readonly ExamLevelInfo[] = [
  { id: "TOEIC", slug: "toeic", lang: "en", hint: "Listening & Reading · 10–990 điểm" },
];

/**
 * Ngôn ngữ có kỳ thi — chỉ những ngôn ngữ này mới hiện trong Luyện thi (chưa
 * có đề thì hiện ra chỉ để thấy trang trống).
 */
export function examLanguages(): LanguageConfig[] {
  return LANGUAGES.filter((lang) => EXAM_LEVELS.some((l) => l.lang === lang.code));
}

export function levelsOfLang(lang: string): ExamLevelInfo[] {
  return EXAM_LEVELS.filter((l) => l.lang === lang);
}

/** Trang của một ngôn ngữ trong Luyện thi = kỳ thi đầu tiên của nó. */
export function examLangHref(lang: string): string {
  const first = levelsOfLang(lang)[0];
  return first ? examLevelHref(first.id) : examLevelHref(EXAM_LEVELS[0].id);
}

export function examLevelInfo(level: ExamLevel): ExamLevelInfo {
  return EXAM_LEVELS.find((l) => l.id === level) ?? EXAM_LEVELS[0];
}

export function examLevelHref(level: ExamLevel): string {
  const info = EXAM_LEVELS.find((l) => l.id === level) ?? EXAM_LEVELS[0];
  return `/exam/${info.slug}`;
}

/**
 * Kỳ thi của một đường dẫn trong khu Luyện thi: trang danh sách
 * (`/exam/toeic`), hoặc trang một đề — suy từ mã đề (`toeic-reading-01` bắt
 * đầu bằng slug của kỳ thi), khỏi phải nạp dữ liệu đề.
 */
export function examLevelOfPath(path: string): ExamLevel | undefined {
  const id = /^\/exam\/([^/]+)/.exec(path)?.[1];
  if (!id) return undefined;
  return EXAM_LEVELS.find((l) => id === l.slug || id.startsWith(`${l.slug}-`))?.id;
}
