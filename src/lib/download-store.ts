"use client";

import { useEffect } from "react";
import { create } from "zustand";
import {
  deleteOfflineBook,
  downloadBook,
  listOfflineBooks,
  type DownloadProgress,
} from "@/lib/offline-books";
import type { Book } from "@/lib/books";

/**
 * Mỗi lần chỉ giữ MỘT cuốn trên máy.
 *
 * Một cuốn cỡ 60MB. Cho tải thoải mái thì vài cuốn là đầy máy người dùng, mà
 * trình duyệt hết chỗ thì nó tự xoá — âm thầm, không báo, và thường xoá
 * đúng lúc cần nhất. Giới hạn rõ ràng kèm nút xoá vẫn hơn là để hệ điều hành
 * quyết hộ.
 */
export const MAX_OFFLINE_BOOKS = 1;

interface DownloadState {
  /** Các cuốn đang có trên máy. Đọc từ Cache API — đó mới là nguồn sự thật. */
  offlineBooks: string[];
  hasLoaded: boolean;
  /** Cuốn đang tải và tiến độ của nó. null = không tải gì. */
  active: { bookId: string; progress: DownloadProgress } | null;
  /** Lỗi của lần tải gần nhất, để UI nói lại cho người dùng. */
  error: string | null;

  refresh: () => Promise<void>;
  start: (book: Book) => Promise<void>;
  cancel: () => void;
  remove: (bookId: string) => Promise<void>;
  clearError: () => void;
}

/** Tham chiếu để huỷ — thuộc về lượt tải đang chạy, không phải state React. */
let controller: AbortController | null = null;

export const useDownloadStore = create<DownloadState>((set, get) => ({
  offlineBooks: [],
  hasLoaded: false,
  active: null,
  error: null,

  refresh: async () => {
    set({ offlineBooks: await listOfflineBooks(), hasLoaded: true });
  },

  start: async (book) => {
    const { offlineBooks, active } = get();
    if (active) return;
    if (offlineBooks.includes(book.id)) return;
    // Chặn ở đây chứ không chỉ ở giao diện: hai tab cùng mở thì nút bên tab
    // kia vẫn đang ở trạng thái cũ.
    if (offlineBooks.length >= MAX_OFFLINE_BOOKS) return;

    controller = new AbortController();
    set({
      active: { bookId: book.id, progress: { done: 0, total: book.totalPages } },
      error: null,
    });

    try {
      const { saved, total } = await downloadBook(book, {
        signal: controller.signal,
        onProgress: (progress) => {
          // Huỷ giữa chừng thì bỏ qua mấy nhịp tiến độ còn bay về sau đó.
          if (get().active?.bookId === book.id) set({ active: { bookId: book.id, progress } });
        },
      });
      if (controller.signal.aborted) {
        // Huỷ thì dọn sạch phần đã tải: một cuốn dở dang vẫn chiếm chỗ mà
        // đọc offline lại hụt trang, tệ hơn là không có gì.
        await deleteOfflineBook(book.id);
      } else if (saved < total) {
        set({
          error: `Tải được ${saved}/${total} trang. Vài trang sẽ không xem được khi mất mạng — thử tải lại.`,
        });
      }
    } catch {
      set({ error: "Không tải được sách. Kiểm tra lại mạng rồi thử lại." });
    } finally {
      controller = null;
      set({ active: null });
      await get().refresh();
    }
  },

  cancel: () => {
    controller?.abort();
  },

  remove: async (bookId) => {
    await deleteOfflineBook(bookId);
    await get().refresh();
  },

  clearError: () => set({ error: null }),
}));

/** Đọc danh sách sách đã tải, sau lần render đầu ở client. */
export function useOfflineBooks(): { books: string[]; hasLoaded: boolean } {
  const books = useDownloadStore((s) => s.offlineBooks);
  const hasLoaded = useDownloadStore((s) => s.hasLoaded);
  useEffect(() => {
    if (!useDownloadStore.getState().hasLoaded) void useDownloadStore.getState().refresh();
  }, []);
  return { books, hasLoaded };
}

/** Còn chỗ để tải thêm cuốn nữa không. */
export function hasFreeSlot(offlineBooks: string[]): boolean {
  return offlineBooks.length < MAX_OFFLINE_BOOKS;
}
