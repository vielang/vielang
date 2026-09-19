"use client";

import { useEffect } from "react";
import { useNoteStore } from "@/lib/note-store";
import { DRAW_STORAGE_KEY, useDrawStore } from "@/lib/draw-store";
import { subscribeToStoreChanges } from "@/lib/idb-storage";

/**
 * Cửa sổ bài giảng riêng — mở bằng `window.open` tới route
 * `/read/<book>/<page>/note`, để kéo sang màn hình thứ hai và đọc/ghi song
 * song với cửa sổ chính.
 *
 * Đồng bộ giữa 2 cửa sổ đi hoàn toàn qua `localStorage` + sự kiện `storage`,
 * không dùng BroadcastChannel/postMessage:
 *
 * - Bản note vốn ĐÃ nằm ở localStorage (`note-store`), nên chỉ cần nghe
 *   `storage` rồi `rehydrate()` là hai bên thấy sửa đổi của nhau.
 * - Trang đang đọc được cửa sổ chính ghi vào `FOCUS_KEY`; cửa sổ note nghe
 *   và tự chuyển theo, nên lật trang bên này thì bên kia đổi bài giảng.
 *
 * Sự kiện `storage` cố ý KHÔNG bắn ở chính cửa sổ vừa ghi — đúng thứ cần ở
 * đây, không sợ vòng lặp ghi/đọc lẫn nhau.
 */
const FOCUS_KEY = "kiip-note-focus-v1";
const WINDOW_NAME = "kiip-note";

export interface NoteFocus {
  bookId: string;
  page: number;
}

/** Tham chiếu cửa sổ note — module-scope, không thuộc state React. */
let noteWindow: Window | null = null;

export function openNoteWindow(bookId: string, page: number): Window | null {
  const url = `/read/${bookId}/${page}/note`;
  // Đặt tên cửa sổ để mở lần 2 là focus lại đúng cửa sổ cũ, không đẻ thêm.
  const features = "popup=yes,width=460,height=760,noopener=no,noreferrer=no";
  const win = window.open(url, WINDOW_NAME, features);
  noteWindow = win;
  win?.focus();
  return win;
}

export function focusNoteWindow(): void {
  noteWindow?.focus();
}

export function closeNoteWindow(): void {
  noteWindow?.close();
  noteWindow = null;
}

export function isNoteWindowClosed(): boolean {
  return noteWindow === null || noteWindow.closed;
}

/** Cửa sổ chính: công bố trang đang đọc để cửa sổ note bám theo. */
export function publishNoteFocus(bookId: string, page: number): void {
  try {
    localStorage.setItem(FOCUS_KEY, JSON.stringify({ bookId, page }));
  } catch {
    /* localStorage bị chặn (chế độ riêng tư...) — bỏ qua, chỉ mất đồng bộ */
  }
}

/**
 * Nghe sửa đổi bài giảng và bảng vẽ từ cửa sổ còn lại. `persist.rehydrate()`
 * đọc lại localStorage và nạp vào store, nên view mode ở cửa sổ kia cập nhật
 * ngay.
 *
 * Riêng bảng vẽ: nạp lại store KHÔNG làm canvas đang mở nhảy theo —
 * Excalidraw chỉ đọc `initialData` lúc khởi tạo (xem `note-draw`). Đúng ý:
 * hai cửa sổ cùng vẽ một trang thì bên nào lưu sau thắng, chứ không giật
 * nét vẽ khỏi tay người đang vẽ.
 */
export function useNoteStoreSync(): void {
  useEffect(() => {
    // Bài giảng nằm ở localStorage nên dùng sự kiện `storage` sẵn có; bảng vẽ
    // đã chuyển sang IndexedDB — kho đó không có sự kiện tương đương nên
    // `idb-storage` tự phát tin qua BroadcastChannel.
    function onStorage(e: StorageEvent) {
      if (e.key !== "kiip-notes-v1") return;
      void useNoteStore.persist.rehydrate();
    }
    window.addEventListener("storage", onStorage);
    const unsubscribe = subscribeToStoreChanges(DRAW_STORAGE_KEY, () => {
      void useDrawStore.persist.rehydrate();
    });
    return () => {
      window.removeEventListener("storage", onStorage);
      unsubscribe();
    };
  }, []);
}

/** Cửa sổ note: bám theo trang mà cửa sổ chính đang đọc. */
export function useFollowNoteFocus(onFocus: (focus: NoteFocus) => void): void {
  useEffect(() => {
    function onStorage(e: StorageEvent) {
      if (e.key !== FOCUS_KEY || !e.newValue) return;
      try {
        onFocus(JSON.parse(e.newValue) as NoteFocus);
      } catch {
        /* giá trị hỏng — bỏ qua */
      }
    }
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [onFocus]);
}
