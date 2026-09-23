"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

/** Bài làm của người dùng cho 1 bài tập (1 trang sách, hoặc 1 bài học IT). */
export interface PageAnswers {
  /** id câu hỏi -> lựa chọn (chỉ số) hoặc chữ đã gõ. */
  answers: Record<string, string | number>;
  /** id các câu đã bấm "Kiểm tra" — để mở lại panel vẫn thấy kết quả cũ. */
  checked: string[];
}

interface QuizState {
  /**
   * Khoá là một CHUỖI MỜ do nơi gọi đặt, store không hiểu và không tách nó ra:
   *
   * - trang sách: `${bookId}:${page}` (xem `noteKey`) — định dạng này có từ
   *   đầu và phải giữ nguyên, người dùng đã có bài làm lưu trong máy.
   * - bài học IT: `it:<khoá>/<bài>` (xem `lessonQuizId`).
   *
   * Tên field vẫn là `pages` vì đây chính là dữ liệu đã nằm trong
   * localStorage; đổi tên là mất bài làm cũ của mọi người.
   */
  pages: Record<string, PageAnswers>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  setAnswer: (quizId: string, questionId: string, value: string | number) => void;
  markChecked: (quizId: string, questionId: string) => void;
  resetQuiz: (quizId: string) => void;
}

const EMPTY: PageAnswers = { answers: {}, checked: [] };

/**
 * Bài làm chỉ lưu trên trình duyệt (localStorage), cùng kiểu với
 * `progress-store` và `note-store` — app không có tài khoản/backend.
 *
 * Lưu CẢ câu trả lời lẫn danh sách câu đã chấm: đóng panel hay lật trang rồi
 * quay lại vẫn thấy nguyên bài làm và kết quả, không phải làm lại từ đầu —
 * đúng lý do note cũng lưu local.
 */
export const useQuizStore = create<QuizState>()(
  persist(
    (set) => ({
      pages: {},
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      setAnswer: (quizId, questionId, value) =>
        set((state) => {
          const prev = state.pages[quizId] ?? EMPTY;
          return {
            pages: {
              ...state.pages,
              [quizId]: {
                ...prev,
                answers: { ...prev.answers, [questionId]: value },
                // Sửa đáp án thì bỏ dấu "đã chấm" của chính câu đó — nếu
                // không, phản hồi cũ vẫn đứng đó trong khi đáp án đã khác.
                checked: prev.checked.filter((id) => id !== questionId),
              },
            },
          };
        }),

      markChecked: (quizId, questionId) =>
        set((state) => {
          const prev = state.pages[quizId] ?? EMPTY;
          if (prev.checked.includes(questionId)) return state;
          return {
            pages: {
              ...state.pages,
              [quizId]: { ...prev, checked: [...prev.checked, questionId] },
            },
          };
        }),

      resetQuiz: (quizId) =>
        set((state) => {
          if (!(quizId in state.pages)) return state;
          const pages = { ...state.pages };
          delete pages[quizId];
          return { pages };
        }),
    }),
    {
      name: "kiip-quiz-v1",
      storage: createJSONStorage(() => localStorage),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

// Nhiều tab cùng mở thì tab ghi sau không được xoá mất dữ liệu tab kia vừa
// ghi — xem `cross-tab-sync`.
syncAcrossTabs(useQuizStore);

/** Bài làm của 1 bài tập; luôn trả về object hợp lệ để component khỏi phải kiểm. */
export function useQuizAnswers(quizId: string): PageAnswers {
  return useQuizStore((s) => s.pages[quizId]) ?? EMPTY;
}
