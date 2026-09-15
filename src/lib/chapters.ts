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

/** Danh sách trang bắt đầu của từng bài (bài 1..N), theo đúng thứ tự. */
export function getChapterStartPages(bookId: string): number[] {
  const textbookLayout = AUDIO_LAYOUTS[bookId];
  if (textbookLayout) return textbookLayout.lessonStartPages;

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
