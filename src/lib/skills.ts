/**
 * Gắn một bài tập với KỸ NĂNG nó luyện (từ vựng, ngữ pháp, nghe, đọc, viết)
 * và — với bài ngữ pháp của sách bài tập — với đúng ĐIỂM NGỮ PHÁP trong giáo
 * trình.
 *
 * Chỉ dựa vào tên mục in trong sách ("어휘 1", "문법 · 동-고 싶다 2",
 * "말하기와 듣기 1", "읽기"...), không đoán. Mục nào không rõ luyện kỹ năng gì
 * thì trả `null` và KHÔNG đưa vào thống kê — xếp nhầm thì con số năng lực nói
 * sai về người học, tệ hơn là không nói gì.
 */
import { BOOKS } from "@/lib/books";
import { getChapters } from "@/lib/chapters";
import { getAllGrammar, type GrammarEntry } from "@/lib/page-grammar";
import { getPageAnswers } from "@/lib/page-answers";

export type Skill = "어휘" | "문법" | "듣기" | "읽기" | "쓰기";

/** Thứ tự cố định — so sánh giữa các lần xem thì hàng không được nhảy chỗ. */
export const SKILLS: { id: Skill; vi: string }[] = [
  { id: "어휘", vi: "Từ vựng" },
  { id: "문법", vi: "Ngữ pháp" },
  { id: "듣기", vi: "Nghe" },
  { id: "읽기", vi: "Đọc" },
  { id: "쓰기", vi: "Viết" },
];

/**
 * Kỹ năng của một mục theo tên in trong sách.
 *
 * - "어휘와 문법" (quiz Sơ cấp 1) luyện cả hai lẫn lộn → `null`.
 * - "말하기와 듣기" → nghe: phần chấm được của nó là câu nghe hiểu; phần nói
 *   không có đáp án nên vốn không có trong dữ liệu.
 * - "읽기와 쓰기" → đọc, cùng lý do: câu có đáp án là câu đọc hiểu.
 */
export function skillOfSection(section: string): Skill | null {
  const title = section.replace(/^복습\s*\d+\s*—\s*/, "").trim();
  if (/^어휘와\s*문법/.test(title)) return null;
  if (title.startsWith("문법")) return "문법";
  if (title.startsWith("어휘")) return "어휘";
  if (title.includes("듣기")) return "듣기";
  if (title.startsWith("읽기")) return "읽기";
  if (title.startsWith("쓰기")) return "쓰기";
  return null;
}

/**
 * Tên điểm ngữ pháp trong tên mục của sách bài tập:
 * "문법 · 동-어서 1 (기본형 → -아서/어서)" → "동-어서".
 */
export function grammarHeading(section: string): string | null {
  const m = /^문법 · (.+?)(?: \d+)?(?: \(.*\))?$/.exec(section);
  return m ? m[1].replace(/ \d+$/, "").trim() : null;
}

const grammarCache = new Map<string, GrammarEntry | null>();

/**
 * Điểm ngữ pháp trong GIÁO TRÌNH ứng với một bài ngữ pháp của sách bài tập.
 *
 * Không so tên — sách bài tập viết "동-고 싶다", giáo trình viết "동 -고
 * 싶다", tìm theo chữ chỉ khớp được khoảng 1/3. Thay vào đó dựa vào cấu
 * trúc: bài N của sách bài tập luyện đúng hai điểm ngữ pháp của bài N giáo
 * trình, CÙNG THỨ TỰ (đã soát cả 68 bài của bốn cuốn: khớp hết). Nên điểm
 * ngữ pháp thứ i xuất hiện trong bài N của sách bài tập = điểm thứ i của bài
 * N giáo trình.
 */
export function textbookGrammarFor(
  workbookId: string,
  page: number,
  section: string
): GrammarEntry | null {
  const heading = grammarHeading(section);
  if (!heading || !workbookId.startsWith("wb-")) return null;
  const cacheKey = `${workbookId}:${page}:${heading}`;
  if (grammarCache.has(cacheKey)) return grammarCache.get(cacheKey)!;

  const textbookId = workbookId.slice(3);
  const wb = BOOKS.find((b) => b.id === workbookId);
  const tb = BOOKS.find((b) => b.id === textbookId);
  let result: GrammarEntry | null = null;
  if (wb && tb) {
    const wbChapter = getChapters(wb.id, wb.totalPages).find(
      (c) => page >= c.startPage && page <= c.endPage
    );
    const tbChapter = wbChapter
      ? getChapters(tb.id, tb.totalPages).find((c) => c.lesson === wbChapter.lesson)
      : undefined;
    if (wbChapter && tbChapter) {
      const headings: string[] = [];
      for (let p = wbChapter.startPage; p <= wbChapter.endPage; p++) {
        for (const key of getPageAnswers(wb.id, p)) {
          const h = grammarHeading(key.section);
          if (h && !headings.includes(h)) headings.push(h);
        }
      }
      const points = getAllGrammar().filter(
        (g) => g.bookId === tb.id && g.page >= tbChapter.startPage && g.page <= tbChapter.endPage
      );
      const i = headings.indexOf(heading);
      // Số điểm hai bên lệch nhau thì không dám ghép theo thứ tự.
      if (i >= 0 && headings.length === points.length) result = points[i];
    }
  }
  grammarCache.set(cacheKey, result);
  return result;
}
