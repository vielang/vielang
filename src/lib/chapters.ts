/**
 * Ranh giới "Bài N" (과) của từng sách — dùng để nhóm PageGrid theo bài học
 * thay vì hiển thị 1 khối trang liền không phân đoạn.
 *
 * Textbook: tái dùng `lessonStartPages` đã verify sẵn trong audio-config.ts
 * (đọc trực tiếp ảnh trang thật, xử lý cả phần ôn tập dài ngắn không đều).
 *
 * Workbook: XÁC MINH bằng ảnh trang thật (WB_step1 trang 10 "1과 안녕하세요?",
 * trang 16 "2과 방에 책상이 있어요"; WB_step3 trang 10 "1과 대인 관계", trang 18
 * "2과 성격") — khoảng cách giữa các bài ĐỀU NHAU (không có phần ôn tập chen
 * vào như textbook, đã xác nhận qua toàn bộ vị trí trang có audio), nên rút
 * gọn thành công thức: `lessonStart(n) = trang audio bài n - offset cố định`
 * (offset = 4 cho step1/2, 5 cho step3/4 — cùng seri thì offset giống nhau).
 */
import { AUDIO_LAYOUTS } from "@/lib/audio-config";

interface WorkbookChapterLayout {
  lessonCount: number;
  /** Trang bắt đầu của Bài 1. */
  firstPage: number;
  /** Khoảng cách trang giữa 2 bài liên tiếp (đều nhau). */
  pageStep: number;
}

const WORKBOOK_CHAPTERS_1_2: WorkbookChapterLayout = {
  lessonCount: 18,
  firstPage: 10,
  pageStep: 6,
};

const WORKBOOK_CHAPTERS_3_4: WorkbookChapterLayout = {
  lessonCount: 16,
  firstPage: 10,
  pageStep: 8,
};

const WORKBOOK_CHAPTERS: Record<string, WorkbookChapterLayout> = {
  "wb-step1": WORKBOOK_CHAPTERS_1_2,
  "wb-step2": WORKBOOK_CHAPTERS_1_2,
  "wb-step3": WORKBOOK_CHAPTERS_3_4,
  "wb-step4": WORKBOOK_CHAPTERS_3_4,
};

/**
 * Ranh giới "Unit N" cho sách không có audio (vd English File) — verify tay
 * bằng ảnh trang thật (đọc mục lục + đối chiếu 2-3 trang mẫu), giống cách
 * làm với AUDIO_LAYOUTS. Trang PDF = trang in trong sách + 1 (2 trang bìa/
 * mục lục đứng trước trang in số 1) — cùng offset ở mọi cấp English File
 * 4th edition đã kiểm (Beginner: Unit 1, 7, Revise&Check 11&12; Elementary:
 * Unit 1, 7). Mỗi Unit dài đều nhau trong 1 cấp (Beginner 6 trang/unit,
 * Elementary 8 trang/unit — do Elementary có 3 lesson A/B/C thay vì 2).
 */
const TEXTBOOK_CHAPTERS: Record<string, number[]> = {
  "en-beginner": [7, 13, 19, 25, 31, 37, 43, 49, 55, 61, 67, 73],
  "en-elementary": [7, 15, 23, 31, 39, 47, 55, 63, 71, 79, 87, 95],
  // Pre-intermediate: offset 0 (KHÔNG +1 như Beginner/Elementary — đã verify
  // Unit 1 và Unit 7 bằng ảnh trang thật, trang PDF khớp thẳng trang in).
  // File PDF gốc có 258 trang nhưng trang 168-258 là Teacher's Guide
  // photocopiable đóng kèm — không tách vào sourceDir, totalPages chỉ tính
  // 167 trang Student's Book thật.
  "en-pre-intermediate": [6, 14, 22, 30, 38, 46, 54, 62, 70, 78, 86, 94],
  // Intermediate: chỉ 10 Unit (không phải 12), mỗi Unit đều 10 trang (A+B+
  // Practical English/Revise&Check) — offset +1 giống Beginner/Elementary,
  // verify bằng Unit 1 (trang in 6) và Unit 6 (trang in 56).
  "en-intermediate": [7, 17, 27, 37, 47, 57, 67, 77, 87, 97],
  // Intermediate Plus: 10 Unit, offset 0 (giống Pre-intermediate, KHÔNG +1)
  // — verify bằng Unit 1 (trang 6) và Unit 6 (trang 56).
  "en-intermediate-plus": [6, 16, 26, 36, 46, 56, 66, 76, 86, 96],
  // Upper-Intermediate: 10 Unit, offset +1 (giống Beginner/Elementary/
  // Intermediate, KHÔNG như Pre-intermediate/Intermediate Plus) — verify
  // bằng Unit 1 (trang in 6) và Unit 6 (trang in 56).
  "en-upper-intermediate": [7, 17, 27, 37, 47, 57, 67, 77, 87, 97],
  // Advanced: 10 Unit, offset +2 (nhiều hơn các cấp khác 1 trang do có thêm
  // trang "Course overview" 2 trang trước Contents) — verify bằng Unit 1
  // (trang in 6) và Unit 6 (trang in 56).
  "en-advanced": [8, 18, 28, 38, 48, 58, 68, 78, 88, 98],
};

/** Danh sách trang bắt đầu của từng bài (bài 1..N), theo đúng thứ tự. */
export function getChapterStartPages(bookId: string): number[] {
  const textbookLayout = AUDIO_LAYOUTS[bookId];
  if (textbookLayout) return textbookLayout.lessonStartPages;

  const noAudioLayout = TEXTBOOK_CHAPTERS[bookId];
  if (noAudioLayout) return noAudioLayout;

  const wbLayout = WORKBOOK_CHAPTERS[bookId];
  if (wbLayout) {
    return Array.from(
      { length: wbLayout.lessonCount },
      (_, i) => wbLayout.firstPage + wbLayout.pageStep * i
    );
  }

  return [];
}

export interface Chapter {
  lesson: number;
  startPage: number;
  /** Trang cuối bài (inclusive) — null cho bài cuối cùng nếu không biết trang cuối sách. */
  endPage: number;
}

/** Chia toàn bộ sách thành các đoạn [bài N: startPage..endPage]. */
export function getChapters(bookId: string, totalPages: number): Chapter[] {
  const starts = getChapterStartPages(bookId);
  return starts.map((startPage, i) => ({
    lesson: i + 1,
    startPage,
    endPage: (starts[i + 1] ?? totalPages + 1) - 1,
  }));
}

/**
 * Chế độ xem 2 trang: ghép 2 TRANG ẢNH đang có thành đúng 1 spread in gốc
 * (đã xác nhận bằng ảnh trang thật — mỗi Lesson thực sự trải trên 2 trang in
 * liên tiếp, nội dung tiếp nối nhau, không phải 2 trang độc lập). Trang PDF
 * ta lưu khác trang in ở mỗi sách theo 1 offset riêng (0/+1/+2, xem
 * TEXTBOOK_CHAPTERS) — offset lệ (chẵn/lẻ) sẽ đổi trang nào là "trang trái"
 * của spread trong hệ số trang PDF. Suy ra offset đó từ mốc Bài 1 (luôn là
 * trang trái thật) thay vì hard-code, để không phải nhớ offset riêng ở đây.
 */
function isSpreadLeftPage(bookId: string, page: number): boolean {
  const starts = getChapterStartPages(bookId);
  const leftIsOdd = starts.length > 0 && starts[0] % 2 === 1;
  return leftIsOdd ? page % 2 === 1 : page % 2 === 0;
}

/**
 * Trang "trái" (hoặc trang lẻ đứng riêng ở đầu/cuối sách) của spread chứa
 * `page`. Dùng để chuẩn hoá trang đích khi nhảy trang ở chế độ xem 2 trang —
 * luôn hiện đúng spread thật, không hiện nửa bên phải trước nửa bên trái.
 */
export function getSpreadAnchor(bookId: string, page: number): number {
  const p = Math.max(1, page);
  return isSpreadLeftPage(bookId, p) ? p : Math.max(1, p - 1);
}

/** Danh sách trang hiện trong spread bắt đầu từ `anchor` — 1 trang nếu là trang lẻ cuối sách. */
export function getSpreadPages(bookId: string, anchor: number, totalPages: number): number[] {
  const right = anchor + 1;
  return isSpreadLeftPage(bookId, anchor) && right <= totalPages ? [anchor, right] : [anchor];
}

/** Spread liền trước/sau spread bắt đầu từ `anchor` (bước đúng 1 hoặc 2 trang tuỳ độ dài spread hiện tại). */
export function getAdjacentSpreadAnchor(
  bookId: string,
  anchor: number,
  totalPages: number,
  direction: 1 | -1
): number {
  if (direction === 1) {
    const step = getSpreadPages(bookId, anchor, totalPages).length;
    return Math.min(anchor + step, totalPages);
  }
  const prevRaw = anchor - 1;
  return prevRaw < 1 ? anchor : getSpreadAnchor(bookId, prevRaw);
}
