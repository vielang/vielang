"use client";

import { useCallback, useState } from "react";
import dynamic from "next/dynamic";
import { Eye, Pencil, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { NOTE_PROSE_CLASS, useEffectiveNote, useNoteStore } from "@/lib/note-store";
import { useNoteWidgetStore } from "@/lib/note-widget-store";

/**
 * Tiptap + ProseMirror nặng và chỉ chạy được ở client — tách khỏi bundle
 * chính, chỉ tải khi người dùng thật sự bấm "Sửa".
 */
const NoteEditor = dynamic(
  () => import("@/components/reader/note-editor").then((m) => m.NoteEditor),
  {
    ssr: false,
    loading: () => (
      <div className="min-h-[40vh] flex-1 animate-pulse rounded-xl border border-border bg-muted/40" />
    ),
  }
);

/**
 * Thân bài giảng — dùng chung cho cả 3 nơi hiển thị: panel nổi
 * (note-widget), sheet toàn màn hình (note-sheet) và cửa sổ riêng
 * (app/read/.../note). Nhờ vậy đổi chế độ hiển thị không làm lệch hành vi
 * đọc/sửa/khôi phục.
 *
 * Trạng thái `editing` lấy từ store module-scope nên giữ nguyên khi đổi chế
 * độ hay lật trang — đang soạn mà lật trang thì vẫn ở chế độ soạn.
 */
export function NotePanel({
  bookId,
  page,
  originalContent,
}: {
  bookId: string;
  page: number;
  /** HTML bài giảng gốc (content/notes); null = trang này chưa biên soạn. */
  originalContent: string | null;
}) {
  const { html, isEdited, hasOriginal, hasContent } = useEffectiveNote(
    bookId,
    page,
    originalContent
  );
  const resetNote = useNoteStore((s) => s.resetNote);
  const editing = useNoteWidgetStore((s) => s.editing);
  const setEditing = useNoteWidgetStore((s) => s.setEditing);

  // Bấm "Khôi phục bản gốc" lúc đang sửa: remount editor để nó nạp lại nội
  // dung gốc — Tiptap không tự theo dõi prop `content` sau khi khởi tạo.
  const [editorNonce, setEditorNonce] = useState(0);

  const handleReset = useCallback(() => {
    resetNote(bookId, page);
    setEditorNonce((n) => n + 1);
  }, [resetNote, bookId, page]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-2 px-3 pb-2">
        <Button
          variant={editing ? "default" : "outline"}
          size="sm"
          onClick={() => setEditing(!editing)}
        >
          {editing ? (
            <>
              <Eye className="size-4" aria-hidden /> Xem
            </>
          ) : (
            <>
              <Pencil className="size-4" aria-hidden />{" "}
              {hasContent ? "Sửa" : "Soạn bài giảng"}
            </>
          )}
        </Button>
        {isEdited && hasOriginal && (
          <Button variant="ghost" size="sm" onClick={handleReset}>
            <RotateCcw className="size-4" aria-hidden /> Khôi phục bản gốc
          </Button>
        )}
      </div>

      {editing ? (
        <div className="flex min-h-0 flex-1 flex-col px-3 pb-3">
          {/* key gồm cả trang: lật trang lúc đang soạn thì editor nạp lại nội
              dung trang mới, và bản đang gõ dở của trang cũ được ghi nốt ở
              cleanup của editor cũ (xem note-editor). */}
          <NoteEditor
            key={`${bookId}:${page}:${editorNonce}`}
            bookId={bookId}
            page={page}
            initialHtml={html ?? ""}
          />
        </div>
      ) : (
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-6">
          {hasContent ? (
            // Nội dung chỉ đến từ content/notes (do mình biên soạn) hoặc từ
            // chính localStorage của người dùng — không có nguồn bên thứ ba
            // nên không cần sanitize.
            <div
              className={NOTE_PROSE_CLASS}
              dangerouslySetInnerHTML={{ __html: html ?? "" }}
            />
          ) : (
            <p className="text-sm text-muted-foreground">
              Trang này chưa có bài giảng — bấm “Soạn bài giảng” để tự viết.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
