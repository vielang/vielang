import type { ExamLevel } from "@/lib/exam-types";

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
  /** Kỹ năng và dải cấp — dòng giới thiệu dưới tab. */
  hint: string;
}

export const EXAM_LEVELS: readonly ExamLevelInfo[] = [
  { id: "TOPIK I", slug: "topik-i", hint: "Cấp 1–2 · Nghe, Đọc" },
  { id: "TOPIK II", slug: "topik-ii", hint: "Cấp 3–6 · Nghe, Viết, Đọc" },
];

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
