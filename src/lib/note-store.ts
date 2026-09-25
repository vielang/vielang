"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

/** Bản note do người dùng tự sửa, đè lên bản gốc trong content/notes. */
export interface LocalNote {
  /** HTML — đúng định dạng `editor.getHTML()` của Tiptap và của bản gốc. */
  html: string;
  updatedAt: string;
}

interface NoteState {
  /** key = `${bookId}:${page}` — xem `noteKey()`. */
  notes: Record<string, LocalNote>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
  saveNote: (bookId: string, page: number, html: string) => void;
  /** Xoá bản sửa cục bộ -> trang quay về bài giảng gốc. */
  resetNote: (bookId: string, page: number) => void;
}

export function noteKey(bookId: string, page: number): string {
  return `${bookId}:${page}`;
}

/**
 * Bài giảng người dùng tự sửa — chỉ lưu trên trình duyệt (localStorage),
 * không có tài khoản/backend, giống `progress-store`. Bản gốc trong
 * content/notes không bao giờ bị đụng tới: store này chỉ giữ phần "đè lên",
 * nên xoá 1 entry là quay về nguyên bản (xem `resetNote`).
 *
 * Lưu HTML thô thay vì JSON của ProseMirror để đọc được kể cả khi không có
 * editor (view mode chỉ render HTML) và để đồng nhất với bản gốc.
 */
export const useNoteStore = create<NoteState>()(
  persist(
    (set) => ({
      notes: {},
      hasHydrated: false,
      setHasHydrated: (v) => set({ hasHydrated: v }),

      saveNote: (bookId, page, html) =>
        set((state) => ({
          notes: {
            ...state.notes,
            [noteKey(bookId, page)]: {
              html,
              updatedAt: new Date().toISOString(),
            },
          },
        })),

      resetNote: (bookId, page) =>
        set((state) => {
          const key = noteKey(bookId, page);
          if (!(key in state.notes)) return state;
          const notes = { ...state.notes };
          delete notes[key];
          return { notes };
        }),
    }),
    {
      name: "kiip-notes-v1",
      storage: createJSONStorage(() => localStorage),
      // Chỉ lưu dữ liệu — không ghi cờ hasHydrated vào localStorage (và vào file sao lưu).
      partialize: (s) => ({ notes: s.notes }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

// Nhiều tab cùng mở thì tab ghi sau không được xoá mất bài giảng tab kia vừa
// sửa — xem `cross-tab-sync`. Trước đây chỉ trình đọc và cửa sổ bài giảng
// riêng tự nghe thay đổi (useNoteStoreSync), hai tab thường thì ghi đè nhau.
syncAcrossTabs(useNoteStore);

/** Tiptap trả về `<p></p>` cho document rỗng — coi như "không có nội dung". */
function isBlankHtml(html: string | null | undefined): boolean {
  if (!html) return true;
  return html.replace(/<[^>]*>/g, "").replace(/&nbsp;/g, " ").trim() === "";
}

export interface EffectiveNote {
  /** Nội dung đang hiển thị: bản sửa cục bộ nếu có, không thì bản gốc. */
  html: string | null;
  /** Người dùng đã sửa trang này (kể cả sửa thành rỗng). */
  isEdited: boolean;
  /** Trang này có bài giảng gốc để "khôi phục" về. */
  hasOriginal: boolean;
  /** Có nội dung để đọc (dùng cho chấm báo trên nút bài giảng). */
  hasContent: boolean;
}

/**
 * Gộp bản gốc (render từ server) với bản sửa cục bộ.
 *
 * Trước khi localStorage rehydrate, `notes` còn rỗng nên kết quả trùng đúng
 * với HTML server render — tránh hydration mismatch. UI phụ thuộc kết quả
 * này vẫn nên chờ `hasHydrated` như các component khác trong app.
 */
export function useEffectiveNote(
  bookId: string,
  page: number,
  originalHtml: string | null
): EffectiveNote {
  const local = useNoteStore((s) => s.notes[noteKey(bookId, page)]);
  const html = local ? local.html : originalHtml;
  return {
    html,
    isEdited: local !== undefined,
    hasOriginal: originalHtml !== null,
    hasContent: !isBlankHtml(html),
  };
}

/**
 * Class dùng chung cho phần thân bài giảng — view mode và vùng soạn thảo của
 * Tiptap phải trông y hệt nhau, nếu không người dùng bấm "Sửa" là chữ nhảy.
 * Bảng có `w-full` + wrapper cuộn ngang riêng (xem globals.css) vì bảng ngữ
 * pháp trong sách khá rộng so với màn hình điện thoại.
 */
export const NOTE_PROSE_CLASS =
  "prose prose-sm dark:prose-invert max-w-none prose-headings:font-heading prose-table:text-sm";
