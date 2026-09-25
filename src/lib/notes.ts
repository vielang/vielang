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
// Một file gộp mọi sách, do build-content sinh (xem `buildIndex`) — thêm nội
// dung cho sách mới là tự có mặt, không phải sửa danh sách import ở đây.
import index from "../../content/notes/index.json";

const NOTES = index as Record<string, Record<string, string>>;

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
