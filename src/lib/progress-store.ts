"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

export interface BookProgress {
  lastPage: number;
  readPages: number[];
  bookmarks: number[];
  updatedAt: string;
}

interface ProgressState {
  books: Record<string, BookProgress>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  markPageRead: (bookId: string, page: number) => void;
  setLastPage: (bookId: string, page: number) => void;
  toggleBookmark: (bookId: string, page: number) => void;
}

const EMPTY_PROGRESS: BookProgress = {
  lastPage: 1,
  readPages: [],
  bookmarks: [],
  updatedAt: "",
};

function getOrInit(books: Record<string, BookProgress>, bookId: string): BookProgress {
  return books[bookId] ?? { ...EMPTY_PROGRESS };
}

/**
 * Tiến độ đọc/bookmark, chỉ lưu trên trình duyệt (localStorage) — không có
 * tài khoản/backend. Key có version (`kiip-progress-v1`) để dễ migrate schema
 * sau này nếu cần.
 */
export const useProgressStore = create<ProgressState>()(
  persist(
    (set) => ({
      books: {},
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      markPageRead: (bookId, page) =>
        set((state) => {
          const prev = getOrInit(state.books, bookId);
          const readPages = prev.readPages.includes(page)
            ? prev.readPages
            : [...prev.readPages, page];
          return {
            books: {
              ...state.books,
              [bookId]: {
                ...prev,
                readPages,
                lastPage: page,
                updatedAt: new Date().toISOString(),
              },
            },
          };
        }),

      setLastPage: (bookId, page) =>
        set((state) => {
          const prev = getOrInit(state.books, bookId);
          return {
            books: {
              ...state.books,
              [bookId]: { ...prev, lastPage: page, updatedAt: new Date().toISOString() },
            },
          };
        }),

      toggleBookmark: (bookId, page) =>
        set((state) => {
          const prev = getOrInit(state.books, bookId);
          const bookmarks = prev.bookmarks.includes(page)
            ? prev.bookmarks.filter((p) => p !== page)
            : [...prev.bookmarks, page].sort((a, b) => a - b);
          return {
            books: { ...state.books, [bookId]: { ...prev, bookmarks } },
          };
        }),
    }),
    {
      name: "kiip-progress-v1",
      storage: createJSONStorage(() => localStorage),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

// Nhiều tab cùng mở thì tab ghi sau không được xoá mất dữ liệu tab kia vừa
// ghi — xem `cross-tab-sync`.
syncAcrossTabs(useProgressStore);

export function getBookProgress(
  books: Record<string, BookProgress>,
  bookId: string
): BookProgress {
  return books[bookId] ?? EMPTY_PROGRESS;
}

export function percentRead(progress: BookProgress, totalPages: number): number {
  if (totalPages <= 0) return 0;
  return Math.round((progress.readPages.length / totalPages) * 100);
}

export function resumePage(progress: BookProgress): number {
  return progress.lastPage || 1;
}
