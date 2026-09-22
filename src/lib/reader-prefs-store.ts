"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type PageLayout = "single" | "double";

interface ReaderPrefsState {
  /** Áp dụng cho mọi sách — người dùng chọn 1 lần, không cần chọn lại mỗi sách. */
  pageLayout: PageLayout;
  setPageLayout: (layout: PageLayout) => void;
  /**
   * Đã xem bảng hướng dẫn lần đầu chưa. Trang đọc có cả chục thao tác không
   * có dấu hiệu gì trên màn (vuốt lật trang, chạm xem bản dịch, phím tắt…),
   * nên lần mở sách đầu tiên phải nói ra một lượt — nhưng đúng một lần thôi.
   */
  hasSeenReaderHelp: boolean;
  markReaderHelpSeen: () => void;
  /**
   * Những sách người dùng đã TẮT chấm đáp án. Lưu danh sách tắt chứ không
   * phải danh sách bật: mặc định là hiện, sách mới thêm vào sau cũng hiện
   * luôn mà không phải bật tay.
   *
   * Theo từng sách vì nhu cầu khác nhau theo sách: sách bài tập muốn tự làm
   * rồi mới dò (chấm nằm sẵn cạnh bài dễ làm lộ đáp án), giáo trình thì
   * thường chỉ cần tra nhanh.
   */
  hiddenAnswerBooks: string[];
  toggleAnswersHidden: (bookId: string) => void;
}

/** Sách này có đang tắt chấm đáp án không. */
export function answersHidden(
  state: Pick<ReaderPrefsState, "hiddenAnswerBooks">,
  bookId: string
): boolean {
  return state.hiddenAnswerBooks.includes(bookId);
}

/** Chế độ xem 1 trang/2 trang, lưu trên trình duyệt (localStorage) — giống progress-store. */
export const useReaderPrefsStore = create<ReaderPrefsState>()(
  persist(
    (set) => ({
      pageLayout: "single",
      setPageLayout: (pageLayout) => set({ pageLayout }),
      hasSeenReaderHelp: false,
      markReaderHelpSeen: () => set({ hasSeenReaderHelp: true }),
      hiddenAnswerBooks: [],
      toggleAnswersHidden: (bookId) =>
        set((state) => ({
          hiddenAnswerBooks: state.hiddenAnswerBooks.includes(bookId)
            ? state.hiddenAnswerBooks.filter((id) => id !== bookId)
            : [...state.hiddenAnswerBooks, bookId],
        })),
    }),
    {
      // Khoá localStorage, KHÔNG phải nhãn hiển thị — đổi theo tên thương
      // hiệu mới là mất chế độ xem và cờ "đã xem hướng dẫn" của người đang
      // dùng (hướng dẫn sẽ bật lại từ đầu). Giữ nguyên tên cũ; xem thêm
      // `lib/idb-storage.ts`, cùng lý do.
      name: "kiip-reader-prefs-v1",
      storage: createJSONStorage(() => localStorage),
    }
  )
);
