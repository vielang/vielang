"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { noteKey } from "@/lib/note-store";

/** Bài làm của người dùng cho 1 trang. */
export interface PageAnswers {
  /** id câu hỏi -> lựa chọn (chỉ số) hoặc chữ đã gõ. */
  answers: Record<string, string | number>;
  /** id các câu đã bấm "Kiểm tra" — để mở lại panel vẫn thấy kết quả cũ. */
  checked: string[];
}

interface QuizState {
  /** key = `${bookId}:${page}` (dùng chung `noteKey`). */
  pages: Record<string, PageAnswers>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  setAnswer: (
    bookId: string,
    page: number,
    questionId: string,
    value: string | number
  ) => void;
  markChecked: (bookId: string, page: number, questionId: string) => void;
  resetPage: (bookId: string, page: number) => void;
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

      setAnswer: (bookId, page, questionId, value) =>
        set((state) => {
          const key = noteKey(bookId, page);
          const prev = state.pages[key] ?? EMPTY;
          return {
            pages: {
              ...state.pages,
              [key]: {
                ...prev,
                answers: { ...prev.answers, [questionId]: value },
                // Sửa đáp án thì bỏ dấu "đã chấm" của chính câu đó — nếu
                // không, phản hồi cũ vẫn đứng đó trong khi đáp án đã khác.
                checked: prev.checked.filter((id) => id !== questionId),
              },
            },
          };
        }),

      markChecked: (bookId, page, questionId) =>
        set((state) => {
          const key = noteKey(bookId, page);
          const prev = state.pages[key] ?? EMPTY;
          if (prev.checked.includes(questionId)) return state;
          return {
            pages: {
              ...state.pages,
              [key]: { ...prev, checked: [...prev.checked, questionId] },
            },
          };
        }),

      resetPage: (bookId, page) =>
        set((state) => {
          const key = noteKey(bookId, page);
          if (!(key in state.pages)) return state;
          const pages = { ...state.pages };
          delete pages[key];
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

/** Bài làm của 1 trang; luôn trả về object hợp lệ để component khỏi phải kiểm. */
export function usePageAnswers(bookId: string, page: number): PageAnswers {
  return useQuizStore((s) => s.pages[noteKey(bookId, page)]) ?? EMPTY;
}
