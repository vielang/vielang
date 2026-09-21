import { AUDIO_LAYOUTS } from "@/lib/audio-config";

/**
 * Trang nào trong sách là trang dạy ngữ pháp.
 *
 * Suy ra từ `AUDIO_LAYOUTS` chứ không liệt kê tay: hai bảng đó cùng mô tả
 * một thứ — bố cục bài học — nên chép ra thành hai danh sách là chuốc lấy
 * cảnh sửa một bên quên bên kia.
 *
 * Đã xác minh bằng ảnh trang thật ở đầu, giữa và cuối Sơ cấp 1 (trang 15,
 * 25, 195): mỗi bài đúng 2 điểm ngữ pháp, ở offset +3 và +5. Sơ cấp 2 dùng
 * chung bố cục với Sơ cấp 1.
 *
 * Trung cấp 1 có bố cục KHÁC: 12 trang một bài, hai trang ngữ pháp nằm LIỀN
 * NHAU ở +3 và +4, mỗi trang đúng một điểm. Đã soi cả 32 trang bằng máy để
 * xác nhận, kể cả bài 9 chỗ số trang nhảy quãng.
 *
 * Trung cấp 2 chưa khảo sát — để trống còn hơn đoán bừa rồi gắn chấm vào
 * trang không có ngữ pháp. Nó dùng chung bảng audio với Trung cấp 1 nhưng
 * đó không đủ để suy ra vị trí trang ngữ pháp.
 */
const GRAMMAR_OFFSETS: Record<string, number[]> = {
  step1: [3, 5],
  step2: [3, 5],
  step3: [3, 4],
};

/** Danh sách trang ngữ pháp của 1 sách, sắp xếp tăng dần. */
export function getGrammarPageNumbers(bookId: string): number[] {
  const offsets = GRAMMAR_OFFSETS[bookId];
  const layout = AUDIO_LAYOUTS[bookId];
  if (!offsets || !layout) return [];

  return layout.lessonStartPages
    .flatMap((start) => offsets.map((offset) => start + offset))
    .sort((a, b) => a - b);
}

/** Trang này có phải trang ngữ pháp không. */
export function isGrammarPage(bookId: string, page: number): boolean {
  return getGrammarPageNumbers(bookId).includes(page);
}
