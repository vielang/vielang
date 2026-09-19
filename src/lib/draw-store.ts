"use client";

import { useEffect } from "react";
import { create } from "zustand";
import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";
import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import type { BinaryFiles } from "@excalidraw/excalidraw/types";

/** Bảng vẽ của người dùng cho 1 trang sách. */
export interface LocalDrawing {
  /** Element của Excalidraw, đã bỏ phần `isDeleted` (xem `saveDrawing`). */
  elements: ExcalidrawElement[];
  /** Ảnh dán/chèn vào bảng vẽ, dạng dataURL — chỉ giữ ảnh còn được dùng. */
  files: BinaryFiles;
  /** Màu nền canvas (đổi được trong menu Excalidraw). */
  background: string | null;
  updatedAt: string;
}

interface DrawState {
  /** key = `${bookId}:${page}` — xem `drawKey()`. */
  drawings: Record<string, LocalDrawing>;
  hasHydrated: boolean;
  /**
   * Lần ghi gần nhất bị localStorage từ chối (hết dung lượng). Bảng vẽ có
   * ảnh rất dễ vượt hạn mức ~5MB, và im lặng nuốt lỗi ở đây là mất bài của
   * người dùng — panel vẽ đọc cờ này để báo ra màn hình.
   */
  quotaExceeded: boolean;
  setHasHydrated: (v: boolean) => void;
  saveDrawing: (
    bookId: string,
    page: number,
    elements: readonly ExcalidrawElement[],
    files: BinaryFiles,
    background: string | null
  ) => void;
  clearDrawing: (bookId: string, page: number) => void;
}

export function drawKey(bookId: string, page: number): string {
  return `${bookId}:${page}`;
}

export const DRAW_STORAGE_KEY = "kiip-draws-v1";

/**
 * localStorage bọc try/catch: khác `note-store` (vài KB HTML), bảng vẽ kèm
 * ảnh có thể đụng trần dung lượng, và persist của zustand để lỗi ném thẳng
 * ra `onChange` của Excalidraw thì vỡ cả canvas. Bắt lại rồi bật cờ
 * `quotaExceeded` để UI báo, thay vì crash.
 */
const safeStorage: StateStorage = {
  getItem: (name) => {
    try {
      return localStorage.getItem(name);
    } catch {
      return null;
    }
  },
  setItem: (name, value) => {
    try {
      localStorage.setItem(name, value);
      markQuota(false);
    } catch {
      markQuota(true);
    }
  },
  removeItem: (name) => {
    try {
      localStorage.removeItem(name);
    } catch {
      /* localStorage bị chặn — không còn gì để dọn */
    }
  },
};

/**
 * Đổi cờ quota từ bên trong storage adapter. Phải so sánh trước khi `set`:
 * mỗi lần đổi state là persist lại ghi xuống storage, ghi lại lỗi, lại đổi
 * cờ — không chặn thì thành vòng lặp vô tận.
 */
function markQuota(exceeded: boolean): void {
  if (useDrawStore.getState().quotaExceeded === exceeded) return;
  useDrawStore.setState({ quotaExceeded: exceeded });
}

/** Giữ lại ảnh còn được element nào đó dùng — bỏ ảnh của element đã xoá. */
function pruneFiles(
  elements: readonly ExcalidrawElement[],
  files: BinaryFiles
): BinaryFiles {
  const used = new Set<string>();
  for (const el of elements) {
    const fileId = (el as { fileId?: string | null }).fileId;
    if (fileId) used.add(fileId);
  }
  const kept: BinaryFiles = {};
  for (const id of used) {
    const file = files[id];
    if (file) kept[id] = file;
  }
  return kept;
}

/**
 * Bảng vẽ của người dùng theo từng trang sách — cùng mô hình với
 * `note-store`: chỉ nằm trên trình duyệt (localStorage), không tài khoản,
 * không backend.
 *
 * Khác `note-store` ở 2 điểm, đều vì dữ liệu nặng hơn nhiều:
 * - `skipHydration`: hoãn nạp tới `useDrawHydration()` trong effect, để lần
 *   render đầu ở client khớp HTML server render (chấm báo "trang này có bản
 *   vẽ" không nhấp nháy sai và không gây hydration mismatch).
 * - lưu qua `safeStorage` để lỗi hết dung lượng thành thông báo, không phải
 *   exception giữa lúc đang vẽ.
 */
export const useDrawStore = create<DrawState>()(
  persist(
    (set) => ({
      drawings: {},
      hasHydrated: false,
      quotaExceeded: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      saveDrawing: (bookId, page, elements, files, background) =>
        set((state) => {
          const key = drawKey(bookId, page);
          // Excalidraw giữ element đã xoá lại để undo được; đó là việc của
          // phiên đang mở, không đáng chiếm chỗ trong localStorage.
          const live = elements.filter((el) => !el.isDeleted);

          if (live.length === 0) {
            if (!(key in state.drawings)) return state;
            const drawings = { ...state.drawings };
            delete drawings[key];
            return { drawings };
          }

          return {
            drawings: {
              ...state.drawings,
              [key]: {
                elements: live,
                files: pruneFiles(live, files),
                background,
                updatedAt: new Date().toISOString(),
              },
            },
          };
        }),

      clearDrawing: (bookId, page) =>
        set((state) => {
          const key = drawKey(bookId, page);
          if (!(key in state.drawings)) return state;
          const drawings = { ...state.drawings };
          delete drawings[key];
          return { drawings };
        }),
    }),
    {
      name: DRAW_STORAGE_KEY,
      storage: createJSONStorage(() => safeStorage),
      // Cờ hydrate/quota là trạng thái của phiên, không phải dữ liệu.
      partialize: (state) => ({ drawings: state.drawings }),
      skipHydration: true,
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

/**
 * Nạp bản vẽ từ localStorage sau lần render đầu (xem `skipHydration`). Gọi
 * ở `NotePanel` — nơi duy nhất mở đường tới tab vẽ, dùng chung cho cả panel
 * nổi, sheet toàn màn hình lẫn cửa sổ riêng.
 */
export function useDrawHydration(): boolean {
  const hasHydrated = useDrawStore((s) => s.hasHydrated);
  useEffect(() => {
    if (!useDrawStore.getState().hasHydrated) void useDrawStore.persist.rehydrate();
  }, []);
  return hasHydrated;
}

/** Trang này đã có nét vẽ nào chưa — dùng cho chấm báo trên tab. */
export function useHasDrawing(bookId: string, page: number): boolean {
  return useDrawStore((s) => (s.drawings[drawKey(bookId, page)]?.elements.length ?? 0) > 0);
}
