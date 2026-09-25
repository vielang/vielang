"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

interface GuideChecklistState {
  /** id bài (xem `checklistId`) -> các khoá dòng đã tick (xem `checkKey` lúc build). */
  checked: Record<string, string[]>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  toggle: (articleId: string, key: string) => void;
  reset: (articleId: string) => void;
}

/**
 * Giấy tờ đã chuẩn bị trong checklist hồ sơ của bài cẩm nang.
 *
 * Gom hồ sơ visa mất vài tuần, người ta quay lại nhiều lần để xem còn thiếu
 * gì — không nhớ thì checklist chỉ là một danh sách đọc suông.
 */
export const useGuideChecklistStore = create<GuideChecklistState>()(
  persist(
    (set) => ({
      checked: {},
      hasHydrated: false,
      setHasHydrated: (hasHydrated) => set({ hasHydrated }),
      toggle: (articleId, key) =>
        set((state) => {
          const prev = state.checked[articleId] ?? [];
          const next = prev.includes(key) ? prev.filter((k) => k !== key) : [...prev, key];
          return { checked: { ...state.checked, [articleId]: next } };
        }),
      reset: (articleId) =>
        set((state) => {
          const checked = { ...state.checked };
          delete checked[articleId];
          return { checked };
        }),
    }),
    {
      name: "kiip-guide-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ checked: s.checked }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

syncAcrossTabs(useGuideChecklistStore);
