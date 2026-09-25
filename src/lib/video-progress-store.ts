"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

export interface VideoWatchProgress {
  time: number;
  /** Đã xem gần hết (>=95%) — coi như xong, mở lại thì phát từ đầu thay vì
   * tua tới giây cuối cùng (vô nghĩa với người xem). */
  done: boolean;
  updatedAt: string;
}

interface VideoProgressState {
  lessons: Record<string, VideoWatchProgress>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  setTime: (id: string, time: number, duration: number) => void;
}

const DONE_RATIO = 0.95;
/** Dưới ngưỡng này thì coi như "chưa xem gì", không đáng nhớ vị trí (và
 * không đáng hiện trong "Xem tiếp"). */
const MIN_RESUME_SEC = 5;

/**
 * Vị trí đang xem của từng video, chỉ lưu trên trình duyệt (localStorage) —
 * cùng khuôn với `progress-store.ts` (sách). Player tự lưu định kỳ lúc phát
 * và lúc rời trang, rồi tự tua tới đúng chỗ lúc mở lại — bù cho lúc mạng
 * chập chờn phải tải lại trang giữa chừng (xem `components/video/video-player.tsx`).
 */
export const useVideoProgressStore = create<VideoProgressState>()(
  persist(
    (set) => ({
      lessons: {},
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      setTime: (id, time, duration) =>
        set((state) => ({
          lessons: {
            ...state.lessons,
            [id]: {
              time,
              done: duration > 0 && time / duration >= DONE_RATIO,
              updatedAt: new Date().toISOString(),
            },
          },
        })),
    }),
    {
      name: "kiip-video-progress-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ lessons: s.lessons }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

syncAcrossTabs(useVideoProgressStore);

/** Vị trí nên tua tới lúc mở video — 0 nếu chưa xem, chưa đủ ngưỡng, hoặc đã xem xong. */
export function resumeTime(progress: VideoWatchProgress | undefined): number {
  if (!progress || progress.done || progress.time < MIN_RESUME_SEC) return 0;
  return progress.time;
}

/** Tập đang xem dở gần đây nhất (chưa xong) — cho khối "Xem tiếp" ở trang danh sách. */
export function mostRecentUnfinished(
  lessons: Record<string, VideoWatchProgress>
): { id: string; progress: VideoWatchProgress } | undefined {
  const entries = Object.entries(lessons).filter(
    ([, p]) => !p.done && p.time >= MIN_RESUME_SEC
  );
  if (entries.length === 0) return undefined;
  entries.sort((a, b) => b[1].updatedAt.localeCompare(a[1].updatedAt));
  return { id: entries[0][0], progress: entries[0][1] };
}
