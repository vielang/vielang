"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { dayKey, EMPTY_DAY, type DayStats } from "@/lib/activity";

interface ActivityState {
  /** key = "YYYY-MM-DD" theo giờ máy — xem `dayKey`. */
  days: Record<string, DayStats>;
  /**
   * Trang đã HỌC (ở lại đủ lâu) theo sách — khác `progress-store.readPages`
   * vốn ghi ngay khi trang vừa hiện ra, lật nhanh qua cũng tính. Để riêng ở
   * đây chứ không đổi nghĩa `readPages`: người đang dùng giữ nguyên tiến độ
   * "đã xem" cũ, còn số "đã học" bắt đầu đếm từ bây giờ.
   */
  studied: Record<string, number[]>;
  /** Mục tiêu phút học mỗi tuần, người dùng tự đặt. */
  weeklyGoalMinutes: number;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;

  addActiveTime: (ms: number, now?: Date) => void;
  markPageStudied: (bookId: string, page: number, now?: Date) => void;
  recordQuizCheck: (correct: boolean, now?: Date) => void;
  recordAnswerOpened: (now?: Date) => void;
  recordRecording: (now?: Date) => void;
  setWeeklyGoal: (minutes: number) => void;
}

export const DEFAULT_WEEKLY_GOAL = 90;

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
      studied: {},
      weeklyGoalMinutes: DEFAULT_WEEKLY_GOAL,
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      addActiveTime: (ms, now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({ ...d, activeMs: d.activeMs + ms })),
        })),

      markPageStudied: (bookId, page, now = new Date()) =>
        set((s) => {
          const pageKey = `${bookId}:${page}`;
          const prevBook = s.studied[bookId] ?? [];
          return {
            days: bumpDay(s.days, now, (d) =>
              d.pages.includes(pageKey) ? d : { ...d, pages: [...d.pages, pageKey] }
            ),
            studied: prevBook.includes(page)
              ? s.studied
              : { ...s.studied, [bookId]: [...prevBook, page] },
          };
        }),

      recordQuizCheck: (correct, now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({
            ...d,
            quizChecked: d.quizChecked + 1,
            quizCorrect: d.quizCorrect + (correct ? 1 : 0),
          })),
        })),

      recordAnswerOpened: (now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({ ...d, answersOpened: d.answersOpened + 1 })),
        })),

      recordRecording: (now = new Date()) =>
        set((s) => ({
          days: bumpDay(s.days, now, (d) => ({ ...d, recordings: d.recordings + 1 })),
        })),

      setWeeklyGoal: (minutes) => set({ weeklyGoalMinutes: minutes }),
    }),
    {
      // Khoá tra dữ liệu, KHÔNG đổi theo tên thương hiệu — xem
      // `storage-keys.test.ts`.
      name: "kiip-activity-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: ({ days, studied, weeklyGoalMinutes }) => ({
        days,
        studied,
        weeklyGoalMinutes,
      }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);
