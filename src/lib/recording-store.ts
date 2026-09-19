"use client";

import { useEffect } from "react";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { createIdbStorage, deleteBlob } from "@/lib/idb-storage";

/**
 * Một bản ghi âm của người dùng cho một trang sách.
 *
 * Ở đây CHỈ có phần mô tả. Tiếng nói nằm riêng trong bảng blob của
 * IndexedDB, lấy theo chính `id` này (xem `idb-storage`) — nhét âm thanh vào
 * cùng chỗ với mô tả thì mỗi lần mở sách là nạp cả chục MB tiếng.
 */
export interface Recording {
  id: string;
  /** Tên do người dùng đặt. Rỗng thì hiển thị "Bản ghi N" theo thứ tự. */
  label: string;
  durationMs: number;
  /** Kiểu tệp thật sự thu được — Safari và Chrome cho ra hai kiểu khác nhau. */
  mimeType: string;
  createdAt: string;
}

interface RecordingState {
  /** key = `${bookId}:${page}` — xem `recordingKey()`. */
  recordings: Record<string, Recording[]>;
  hasHydrated: boolean;
  /** Lần ghi gần nhất xuống IndexedDB thất bại — xem `idb-storage`. */
  quotaExceeded: boolean;

  /** Bảng ghi âm đang mở (không lưu — mỗi phiên tự mở lại). */
  open: boolean;
  /** Góc trên trái của bảng, px màn hình. null = chưa từng kéo. */
  panelPos: { x: number; y: number } | null;

  setHasHydrated: (v: boolean) => void;
  setOpen: (v: boolean) => void;
  setPanelPos: (p: { x: number; y: number }) => void;

  addRecording: (bookId: string, page: number, rec: Recording) => void;
  renameRecording: (bookId: string, page: number, id: string, label: string) => void;
  removeRecording: (bookId: string, page: number, id: string) => void;
}

export function recordingKey(bookId: string, page: number): string {
  return `${bookId}:${page}`;
}

export const RECORDING_STORAGE_KEY = "kiip-recordings-v1";

/** So sánh trước khi `set` để không thành vòng lặp ghi/lỗi/ghi — xem `idb-storage`. */
function markQuota(exceeded: boolean): void {
  if (useRecordingStore.getState().quotaExceeded === exceeded) return;
  useRecordingStore.setState({ quotaExceeded: exceeded });
}

function withList(
  state: RecordingState,
  key: string,
  next: Recording[]
): Pick<RecordingState, "recordings"> | RecordingState {
  if (next.length === 0) {
    if (!(key in state.recordings)) return state;
    const recordings = { ...state.recordings };
    delete recordings[key];
    return { recordings };
  }
  return { recordings: { ...state.recordings, [key]: next } };
}

/**
 * Bản ghi âm giọng người dùng theo từng trang sách — đọc to một đoạn tiếng
 * Hàn rồi nghe lại xem phát âm đã giống chưa.
 *
 * Lưu ở IndexedDB (xem `idb-storage`), không backend, giống nét vẽ và bảng
 * vẽ. Phần mô tả đi qua zustand persist, còn tiếng nói nằm ở bảng blob riêng.
 *
 * `skipHydration`: nạp ở effect chứ không lúc dựng store, để lần render đầu
 * ở client khớp HTML server render — chấm báo "trang này có bản ghi" nằm
 * trên thanh công cụ vốn được server render.
 */
export const useRecordingStore = create<RecordingState>()(
  persist(
    (set) => ({
      recordings: {},
      hasHydrated: false,
      quotaExceeded: false,
      open: false,
      panelPos: null,

      setHasHydrated: (v) => set({ hasHydrated: v }),
      setOpen: (open) => set({ open }),
      setPanelPos: (panelPos) => set({ panelPos }),

      addRecording: (bookId, page, rec) =>
        set((state) => {
          const key = recordingKey(bookId, page);
          return {
            recordings: { ...state.recordings, [key]: [...(state.recordings[key] ?? []), rec] },
          };
        }),

      renameRecording: (bookId, page, id, label) =>
        set((state) => {
          const key = recordingKey(bookId, page);
          const current = state.recordings[key];
          if (!current) return state;
          return {
            recordings: {
              ...state.recordings,
              [key]: current.map((r) => (r.id === id ? { ...r, label } : r)),
            },
          };
        }),

      removeRecording: (bookId, page, id) =>
        set((state) => {
          const key = recordingKey(bookId, page);
          const current = state.recordings[key];
          if (!current) return state;
          const next = current.filter((r) => r.id !== id);
          if (next.length === current.length) return state;
          // Xoá luôn tiếng nói. Không chờ kết quả: mô tả mới là nguồn sự
          // thật, còn sót lại một blob mồ côi thì cũng không ai thấy.
          void deleteBlob(id);
          return withList(state, key, next);
        }),
    }),
    {
      name: RECORDING_STORAGE_KEY,
      storage: createJSONStorage(() => createIdbStorage(markQuota)),
      // Cờ hydrate/quota và trạng thái bảng là của phiên, không phải dữ liệu.
      partialize: (state) => ({ recordings: state.recordings }),
      skipHydration: true,
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

/** Nạp danh sách bản ghi từ IndexedDB sau lần render đầu — gọi ở `ReaderView`. */
export function useRecordingHydration(): boolean {
  const hasHydrated = useRecordingStore((s) => s.hasHydrated);
  useEffect(() => {
    if (!useRecordingStore.getState().hasHydrated)
      void useRecordingStore.persist.rehydrate();
  }, []);
  return hasHydrated;
}

/** Trong các trang đang hiện, có trang nào đã ghi âm chưa — cho chấm báo. */
export function useHasRecordings(bookId: string, pages: number[]): boolean {
  return useRecordingStore(
    (s) =>
      s.hasHydrated &&
      pages.some((p) => (s.recordings[recordingKey(bookId, p)]?.length ?? 0) > 0)
  );
}

/** "0:07", "1:23" — đủ cho bản ghi một câu, không cần giờ. */
export function formatDuration(ms: number): string {
  const total = Math.max(0, Math.round(ms / 1000));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

/** Tên hiển thị: tên người dùng đặt, hoặc "Bản ghi N" theo thứ tự trong trang. */
export function recordingLabel(rec: Recording, index: number): string {
  return rec.label.trim() || `Bản ghi ${index + 1}`;
}
