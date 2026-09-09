/**
 * Bản dịch từng đoạn trên ảnh trang sách.
 *
 * Cách hoạt động: mỗi trang có sẵn danh sách vùng chữ nhật (soạn tay) kèm
 * bản dịch đã dịch sẵn. Người dùng bấm vào vùng nào thì hiện bản dịch của
 * vùng đó — KHÔNG có OCR hay dịch máy lúc chạy, mọi thứ tĩnh.
 *
 * PHẠM VI: CHỈ đặt vùng cho BÀI ĐỌC (읽기) và HỘI THOẠI (말하기). Câu đề bài,
 * câu hỏi, dòng thay từ, gợi ý... đều là câu ngắn lặp đi lặp lại, người học
 * quen rất nhanh — dịch hết thì trang nào cũng chi chít vùng bấm mà chẳng
 * giúp được gì thêm.
 *
 * Toạ độ lưu theo TỈ LỆ 0–1 của ảnh gốc (1200x1562), không phải pixel: ảnh
 * hiển thị co giãn theo màn hình và theo mức zoom, chỉ tỉ lệ mới bất biến.
 *
 * Import TĨNH như `lib/notes.ts` và `lib/quiz.ts`, cùng lý do (Output File
 * Tracing của Next không lần được file đọc qua path dựng động lúc chạy).
 */
import step1 from "../../content/translate/step1.json";
import step2 from "../../content/translate/step2.json";
import step3 from "../../content/translate/step3.json";
import step4 from "../../content/translate/step4.json";

/** [x, y, rộng, cao] — tất cả theo tỉ lệ 0–1 của ảnh trang. */
export type Rect = [number, number, number, number];

export interface TranslationRegion {
  id: string;
  rect: Rect;
  /** Nhãn ngắn hiện trên vùng, ví dụ "Hội thoại", "Bài đọc". */
  label?: string;
  /** Nguyên văn tiếng Hàn trong vùng (để đối chiếu khi đọc bản dịch). */
  ko?: string;
  /** Bản dịch tiếng Việt — đã dịch sẵn. */
  vi: string;
}

/**
 * Ép kiểu qua `unknown`: TypeScript suy `rect` trong JSON thành `number[]`,
 * không khớp tuple 4 phần tử `Rect` nên không ép thẳng được. Hình dạng thật
 * đã được `scripts/build-content.ts` kiểm lúc build (đủ 4 số, nằm trong ảnh).
 */
const TRANSLATIONS = {
  step1,
  step2,
  step3,
  step4,
} as unknown as Record<string, Record<string, TranslationRegion[]>>;

export function getPageTranslations(
  bookId: string,
  page: number
): TranslationRegion[] {
  return TRANSLATIONS[bookId]?.[String(page)] ?? [];
}

/** Danh sách số trang đã có bản dịch của 1 sách, sắp xếp tăng dần. */
export function getTranslatedPages(bookId: string): number[] {
  const book = TRANSLATIONS[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}
