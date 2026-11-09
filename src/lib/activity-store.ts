"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { dayKey, EMPTY_DAY, type DayStats } from "@/lib/activity";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

interface ActivityState {
  /** key = "YYYY-MM-DD" theo giờ máy — xem `dayKey`. */
  days: Record<string, DayStats>;
  /** Mục tiêu phút học mỗi tuần, người dùng tự đặt. */
  weeklyGoalMinutes: number;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;

  addActiveTime: (ms: number, now?: Date) => void;
  recordQuizCheck: (correct: boolean, now?: Date) => void;
  setWeeklyGoal: (minutes: number) => void;
}

const DEFAULT_WEEKLY_GOAL = 90;

/** Sửa đúng ngày `now`, tạo ngày mới nếu chưa có. */
function bumpDay(
  days: Record<string, DayStats>,
  now: Date,
  change: (day: DayStats) => DayStats
): Record<string, DayStats> {
  const key = dayKey(now);
  return { ...days, [key]: change(days[key] ?? EMPTY_DAY) };
}

/**
 * Lịch sử học — chỉ lưu trên trình duyệt (localStorage), cùng kiểu với
 * `progress-store`. App không có tài khoản: xoá dữ liệu trình duyệt là mất,
 * nên My page có nút xuất/nhập file sao lưu (xem `learning-backup`).
 */
export const useActivityStore = create<ActivityState>()(
  persist(
    (set) => ({
      days: {},
      weeklyGoalMinutes: DEFAULT_WEEKLY_GOAL,
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      addActiveTime: (ms, now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({ ...d, activeMs: d.activeMs + ms })),
        })),

      recordQuizCheck: (correct, now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({
            ...d,
            quizChecked: d.quizChecked + 1,
            quizCorrect: d.quizCorrect + (correct ? 1 : 0),
          })),
        })),

      setWeeklyGoal: (minutes) => set({ weeklyGoalMinutes: minutes }),
    }),
    {
      // Khoá tra dữ liệu, KHÔNG đổi theo tên thương hiệu — xem
      // `storage-keys.test.ts`.
      name: "kiip-activity-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: ({ days, weeklyGoalMinutes }) => ({ days, weeklyGoalMinutes }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

// Hai tab cùng mở (bài học + Góc học tập) — xem `cross-tab-sync`.
syncAcrossTabs(useActivityStore);
