"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type PageLayout = "single" | "double";

interface ReaderPrefsState {
  /** Áp dụng cho mọi sách — người dùng chọn 1 lần, không cần chọn lại mỗi sách. */
  pageLayout: PageLayout;
  setPageLayout: (layout: PageLayout) => void;
}

/** Chế độ xem 1 trang/2 trang, lưu trên trình duyệt (localStorage) — giống progress-store. */
export const useReaderPrefsStore = create<ReaderPrefsState>()(
  persist(
    (set) => ({
      pageLayout: "single",
      setPageLayout: (pageLayout) => set({ pageLayout }),
    }),
    {
      name: "kiip-reader-prefs-v1",
      storage: createJSONStorage(() => localStorage),
    }
  )
);
