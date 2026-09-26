import type { ExamLevel } from "@/lib/exam-types";
import { LANGUAGES, type LanguageConfig } from "@/lib/languages";

/**
 * Cấp đề thi và đường dẫn của từng cấp (`/exam/topik-i`, `/exam/topik-ii`).
 *
 * Module nhẹ, KHÔNG import dữ liệu đề (lib/exams.ts nạp cả chục file JSON) —
 * header dùng nó ở mọi trang để hiện tab cấp đề. Cấp nằm trên URL chứ không
 * trong state của trang: header mới đổi được, và chia sẻ link / bấm quay lại
 * giữ đúng cấp.
 */
export interface ExamLevelInfo {
  id: ExamLevel;
  slug: string;
  /** Ngôn ngữ của kỳ thi — tầng giữa: Luyện thi → ngôn ngữ → kỳ thi. */
  lang: string;
  /** Kỹ năng và dải cấp — dòng giới thiệu dưới tab. */
  hint: string;
}

// Chưa có kỳ thi nào — TOEIC sẽ thêm ở đây. Rỗng thì tab Luyện thi tự ẩn (xem lib/nav.ts).
export const EXAM_LEVELS: readonly ExamLevelInfo[] = [];

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
 * Cấp đề của một đường dẫn trong khu Luyện thi: trang danh sách theo cấp
 * (`/exam/topik-ii`), hoặc trang một đề — suy từ mã đề (`102-topik2`), khỏi
 * phải nạp dữ liệu đề.
 */
export function examLevelOfPath(path: string): ExamLevel | undefined {
  const byList = EXAM_LEVELS.find((l) => path === `/exam/${l.slug}`);
  if (byList) return byList.id;
  const m = /^\/exam\/[^/]+-topik([12])(?:\/|$)/.exec(path);
  if (!m) return undefined;
  return m[1] === "1" ? "TOPIK I" : "TOPIK II";
}
