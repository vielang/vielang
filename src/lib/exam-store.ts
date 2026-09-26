"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";
import type { Answers, QuestionKey } from "@/lib/exams";

/** Bài luyện tập (không tính giờ) của một đề. */
export interface PracticeState {
  answers: Answers;
  /** Câu đã bấm "Kiểm tra" — để mở lại vẫn thấy kết quả và lời giải thích. */
  checked: string[];
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
  /**
   * Điểm quy đổi lúc nộp, lưu sẵn để danh sách lịch sử khỏi phải chấm lại.
   * (Bản lưu cũ có thể còn trường thừa như `level`, `texts` — không đọc tới.)
   */
  score?: number;
}

interface ExamState {
  practice: Record<string, PracticeState>;
  attempts: MockAttempt[];
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;

  setPracticeAnswer: (examId: string, key: QuestionKey, choice: number) => void;
  checkPractice: (examId: string, key: string) => void;
  resetPractice: (examId: string, keys: string[]) => void;

  startMock: (examId: string, firstSectionMinutes: number, now?: number) => MockAttempt;
  setMockAnswer: (attemptId: string, key: QuestionKey, choice: number) => void;
  nextMockSection: (attemptId: string, minutes: number, now?: number) => void;
  finishMock: (attemptId: string, score: number, now?: number) => void;
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

/**
 * Bản lưu cũ (version 0) khoá bài làm bằng số câu trơn, theo bộ đề đầu tiên
 * của app: 1–30 là nghe, 31–70 là đọc. Từ version 1 khoá là "<phần>:<số>"
 * (xem `qKey`). Giữ bước chuyển này để bản lưu cũ vẫn đọc được, không vỡ.
 */
export function migrateExamState(persisted: unknown, version: number): unknown {
  if (version >= 1 || !persisted || typeof persisted !== "object") return persisted;
  const old = persisted as { practice?: Record<string, PracticeState>; attempts?: MockAttempt[] };
  const key = (k: string | number) => {
    const no = Number(k);
    return Number.isFinite(no) ? `${no <= 30 ? "listening" : "reading"}:${no}` : String(k);
  };
  const rekey = (answers: Record<string, number> = {}) =>
    Object.fromEntries(Object.entries(answers).map(([k, v]) => [key(k), v]));
  return {
    ...old,
    practice: Object.fromEntries(
      Object.entries(old.practice ?? {}).map(([id, p]) => [
        id,
        { ...p, answers: rekey(p.answers), checked: (p.checked ?? []).map(key) },
      ])
    ),
    attempts: (old.attempts ?? []).map((a) => ({ ...a, answers: rekey(a.answers) })),
  };
}

/** Bài làm đề thi — chỉ lưu trên trình duyệt, như các kho khác của app. */
export const useExamStore = create<ExamState>()(
  persist(
    (set, get) => {
      const updatePractice = (examId: string, change: (p: PracticeState) => PracticeState) =>
        set((s) => ({ practice: { ...s.practice, [examId]: change(s.practice[examId] ?? EMPTY) } }));

      return {
        practice: {},
        attempts: [],
        hasHydrated: false,
        setHasHydrated: (v) => set({ hasHydrated: v }),

        setPracticeAnswer: (examId, key, choice) =>
          updatePractice(examId, (prev) => ({
            ...prev,
            answers: { ...prev.answers, [key]: choice },
            // Đổi đáp án thì bỏ dấu "đã kiểm tra" của câu đó — kết quả cũ
            // không còn đúng với đáp án mới.
            checked: prev.checked.filter((k) => k !== key),
          })),

        checkPractice: (examId, key) =>
          updatePractice(examId, (prev) =>
            prev.checked.includes(key) ? prev : { ...prev, checked: [...prev.checked, key] }
          ),

        resetPractice: (examId, keys) =>
          updatePractice(examId, (prev) => {
            const answers = { ...prev.answers };
            for (const k of keys) delete answers[k];
            return { ...prev, answers, checked: prev.checked.filter((k) => !keys.includes(k)) };
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

        setMockAnswer: (id, key, choice) =>
          set((s) => ({
            attempts: updateAttempt(s.attempts, id, (a) =>
              a.finishedAt ? a : { ...a, answers: { ...a.answers, [key]: choice } }
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

        finishMock: (id, score, now = Date.now()) =>
          set((s) => ({
            attempts: updateAttempt(s.attempts, id, (a) => ({
              ...a,
              finishedAt: new Date(now).toISOString(),
              score,
            })),
          })),

        discardMock: (id) => set({ attempts: get().attempts.filter((a) => a.id !== id) }),
      };
    },
    {
      // Khoá tra dữ liệu, KHÔNG đổi theo tên thương hiệu — xem storage-keys.test.
      name: "kiip-exam-v1",
      version: 1,
      migrate: migrateExamState as (persisted: unknown, version: number) => ExamState,
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
