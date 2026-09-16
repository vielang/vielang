"use client";

import { useCallback, useEffect } from "react";
import { useRouter } from "next/navigation";
import { Badge } from "@/components/ui/badge";
import { NotePanel } from "@/components/reader/note-panel";
import { useEffectiveNote } from "@/lib/note-store";
import { useNoteWidgetStore } from "@/lib/note-widget-store";
import {
  useFollowNoteFocus,
  useNoteStoreSync,
  type NoteFocus,
} from "@/lib/note-window";
import type { Book } from "@/lib/books";

/**
 * Nội dung của cửa sổ bài giảng riêng (route /read/<book>/<page>/note).
 *
 * Cửa sổ này chạy độc lập với cửa sổ chính — chỉ nối với nhau qua
 * localStorage (xem lib/note-window.ts): lật trang bên cửa sổ chính thì
 * bên này tự chuyển bài giảng theo, và sửa bên nào bên kia cũng thấy.
 */
export function NoteWindowView({
  book,
  page,
  originalContent,
}: {
  book: Book;
  page: number;
  originalContent: string | null;
}) {
  const router = useRouter();
  const setEditing = useNoteWidgetStore((s) => s.setEditing);
  const { isEdited } = useEffectiveNote(book.id, page, originalContent);

  useNoteStoreSync();

  const follow = useCallback(
    (focus: NoteFocus) => {
      if (focus.bookId === book.id && focus.page === page) return;
      // `replace`: cửa sổ note bám theo cửa sổ chính, không phải điều hướng
      // do người dùng chủ động nên đừng làm dày lịch sử back/forward.
      router.replace(`/read/${focus.bookId}/${focus.page}/note`);
    },
    [router, book.id, page]
  );
  useFollowNoteFocus(follow);

  // Cửa sổ riêng thì gần như luôn để sửa — nhưng đây là store dùng chung với
  // cửa sổ chính (module-scope, mỗi cửa sổ 1 bản riêng) nên chỉ ảnh hưởng
  // chính cửa sổ này.
  useEffect(() => {
    setEditing(true);
  }, [setEditing]);

  return (
    <div className="flex h-dvh flex-col bg-background">
      <header className="flex shrink-0 items-center gap-2 border-b border-border px-3 py-2">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium">Bài giảng — Trang {page}</p>
          <p className="truncate text-xs text-muted-foreground">{book.titleVi}</p>
        </div>
        {isEdited && (
          <Badge variant="secondary" className="font-normal">
            Đã sửa
          </Badge>
        )}
      </header>

      <NotePanel
        bookId={book.id}
        pages={[page]}
        noteContentByPage={{ [page]: originalContent }}
      />
    </div>
  );
}
