"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";
import type { Answers } from "@/lib/exams";

/** Bài luyện tập (không tính giờ) của một đề. */
export interface PracticeState {
  answers: Answers;
  /** Câu đã bấm "Kiểm tra" — để mở lại vẫn thấy kết quả. */
  checked: number[];
}

/**
 * Một lượt THI THỬ. Đang làm dở thì `finishedAt` rỗng.
 *
 * Lưu MỐC HẾT GIỜ của phần đang làm (`deadline`) chứ không lưu "còn bao nhiêu
 * giây": tải lại trang giữa chừng thì đồng hồ vẫn chạy đúng, không được tặng
 * thêm thời gian — đúng như thi thật.
 */
export interface MockAttempt {
  id: string;
  examId: string;
  startedAt: string;
  finishedAt?: string;
  answers: Answers;
  /** Phần đang làm (chỉ số trong `exam.sections`). Sang phần sau là KHÔNG quay lại được. */
  sectionIndex: number;
  /** Mốc hết giờ của phần đang làm (ms, Date.now()). */
  deadline: number;
  /** Điểm lúc nộp, lưu sẵn để danh sách lịch sử khỏi phải chấm lại. */
  score?: number;
  level?: string | null;
}

interface ExamState {
  practice: Record<string, PracticeState>;
  attempts: MockAttempt[];
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;

  setPracticeAnswer: (examId: string, no: number, choice: number) => void;
  checkPractice: (examId: string, no: number) => void;
  resetPractice: (examId: string, nos: number[]) => void;

  startMock: (examId: string, firstSectionMinutes: number, now?: number) => MockAttempt;
  setMockAnswer: (attemptId: string, no: number, choice: number) => void;
  nextMockSection: (attemptId: string, minutes: number, now?: number) => void;
  finishMock: (attemptId: string, score: number, level: string | null, now?: number) => void;
  discardMock: (attemptId: string) => void;
}

const EMPTY: PracticeState = { answers: {}, checked: [] };

function updateAttempt(
  attempts: MockAttempt[],
  id: string,
  change: (a: MockAttempt) => MockAttempt
): MockAttempt[] {
  return attempts.map((a) => (a.id === id ? change(a) : a));
}

/** Bài làm đề thi — chỉ lưu trên trình duyệt, như các kho khác của app. */
export const useExamStore = create<ExamState>()(
  persist(
    (set, get) => ({
      practice: {},
      attempts: [],
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      setPracticeAnswer: (examId, no, choice) =>
        set((s) => {
          const prev = s.practice[examId] ?? EMPTY;
          return {
            practice: {
              ...s.practice,
              [examId]: {
                answers: { ...prev.answers, [no]: choice },
                // Đổi đáp án thì bỏ dấu "đã kiểm tra" của câu đó — kết quả cũ
                // không còn đúng với đáp án mới.
                checked: prev.checked.filter((n) => n !== no),
              },
            },
          };
        }),

      checkPractice: (examId, no) =>
        set((s) => {
          const prev = s.practice[examId] ?? EMPTY;
          if (prev.checked.includes(no)) return s;
          return {
            practice: { ...s.practice, [examId]: { ...prev, checked: [...prev.checked, no] } },
          };
        }),

      resetPractice: (examId, nos) =>
        set((s) => {
          const prev = s.practice[examId] ?? EMPTY;
          const answers = { ...prev.answers };
          for (const n of nos) delete answers[n];
          return {
            practice: {
              ...s.practice,
              [examId]: { answers, checked: prev.checked.filter((n) => !nos.includes(n)) },
            },
          };
        }),

      startMock: (examId, minutes, now = Date.now()) => {
        const attempt: MockAttempt = {
          id: `${examId}-${now.toString(36)}`,
          examId,
          startedAt: new Date(now).toISOString(),
          answers: {},
          sectionIndex: 0,
          deadline: now + minutes * 60_000,
        };
        // Một đề chỉ có MỘT lượt đang làm dở: bắt đầu lượt mới là bỏ lượt dở cũ.
        set((s) => ({
          attempts: [...s.attempts.filter((a) => a.examId !== examId || a.finishedAt), attempt],
        }));
        return attempt;
      },

      setMockAnswer: (id, no, choice) =>
        set((s) => ({
          attempts: updateAttempt(s.attempts, id, (a) =>
            a.finishedAt ? a : { ...a, answers: { ...a.answers, [no]: choice } }
          ),
        })),

      nextMockSection: (id, minutes, now = Date.now()) =>
        set((s) => ({
          attempts: updateAttempt(s.attempts, id, (a) => ({
            ...a,
            sectionIndex: a.sectionIndex + 1,
            deadline: now + minutes * 60_000,
          })),
        })),

      finishMock: (id, score, level, now = Date.now()) =>
        set((s) => ({
          attempts: updateAttempt(s.attempts, id, (a) => ({
            ...a,
            finishedAt: new Date(now).toISOString(),
            score,
            level,
          })),
        })),

      discardMock: (id) => set({ attempts: get().attempts.filter((a) => a.id !== id) }),
    }),
    {
      // Khoá tra dữ liệu, KHÔNG đổi theo tên thương hiệu — xem storage-keys.test.
      name: "kiip-exam-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: ({ practice, attempts }) => ({ practice, attempts }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

// Làm đề ở một tab, xem Góc học tập ở tab khác — xem `cross-tab-sync`.
syncAcrossTabs(useExamStore);

/** Lượt thi thử đang làm dở của một đề (nếu có). */
export function activeAttempt(attempts: MockAttempt[], examId: string): MockAttempt | undefined {
  return attempts.find((a) => a.examId === examId && !a.finishedAt);
}

/** Các lượt đã nộp của một đề, mới nhất trước. */
export function finishedAttempts(attempts: MockAttempt[], examId?: string): MockAttempt[] {
  return attempts
    .filter((a) => a.finishedAt && (!examId || a.examId === examId))
    .sort((a, b) => (b.finishedAt ?? "").localeCompare(a.finishedAt ?? ""));
}
