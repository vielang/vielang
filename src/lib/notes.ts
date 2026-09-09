/**
 * Note "bài giảng" tiếng Việt cho từng trang — nội dung do biên soạn thủ
 * công (xem `content/notes/<bookId>/<page>.md`), convert Markdown -> HTML và
 * gộp thành JSON qua `npm run build-notes` (tự chạy qua predev/prebuild, xem
 * package.json). Đây là bản GỐC, chỉ đọc; bản người dùng tự sửa nằm ở
 * localStorage — xem `lib/note-store.ts`.
 *
 * Import TĨNH (không phải fs.readFileSync với path động ở runtime) — chỉ
 * có 4 sách cố định nên liệt kê hết ra đây là đủ, tránh rủi ro Output File
 * Tracing của Next không include được file đọc qua path dựng động lúc
 * request, gây lỗi khi deploy serverless dù chạy đúng ở `next dev`.
 */
import step1 from "../../content/notes/step1.json";
import step2 from "../../content/notes/step2.json";
import step3 from "../../content/notes/step3.json";
import step4 from "../../content/notes/step4.json";

const NOTES: Record<string, Record<string, string>> = {
  step1,
  step2,
  step3,
  step4,
};

export function getNoteContent(bookId: string, page: number): string | null {
  return NOTES[bookId]?.[String(page)] ?? null;
}

export function hasNote(bookId: string, page: number): boolean {
  return getNoteContent(bookId, page) !== null;
}

/** Danh sách số trang có note của 1 sách, đã sắp xếp tăng dần. */
export function getNotePages(bookId: string): number[] {
  const book = NOTES[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}
