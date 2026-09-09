"use client";

import { useCallback, useState } from "react";
import dynamic from "next/dynamic";
import { Eye, Pencil, RotateCcw } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { NOTE_PROSE_CLASS, useEffectiveNote, useNoteStore } from "@/lib/note-store";

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

export function NoteSheet({
  open,
  onOpenChange,
  originalContent,
  bookId,
  page,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** HTML bài giảng gốc (content/notes); null = trang này chưa biên soạn. */
  originalContent: string | null;
  bookId: string;
  page: number;
}) {
  const { html, isEdited, hasOriginal, hasContent } = useEffectiveNote(
    bookId,
    page,
    originalContent
  );
  const resetNote = useNoteStore((s) => s.resetNote);

  const [editing, setEditing] = useState(false);
  // Bấm "Khôi phục bản gốc" lúc đang sửa: remount editor để nó nạp lại nội
  // dung gốc — Tiptap không tự theo dõi prop `content` sau khi khởi tạo.
  const [editorNonce, setEditorNonce] = useState(0);

  // Đóng sheet thì quay về chế độ đọc cho lần mở sau. (Đổi trang không cần
  // xử lý: page.tsx render <ReaderView key={page}/> nên cả cây này remount.)
  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) setEditing(false);
      onOpenChange(next);
    },
    [onOpenChange]
  );

  const handleReset = useCallback(() => {
    resetNote(bookId, page);
    setEditorNonce((n) => n + 1);
  }, [resetNote, bookId, page]);

  return (
    <Sheet open={open} onOpenChange={handleOpenChange}>
      {/* Chiều cao phải viết dưới dạng biến thể `data-[side=bottom]:` —
          SheetContent đã có `data-[side=bottom]:h-auto`, class `h-[85vh]`
          trơn thua về độ ưu tiên nên sheet sẽ cao bằng cả bài giảng. */}
      <SheetContent
        side="bottom"
        className="flex flex-col data-[side=bottom]:h-[85vh]"
      >
        <SheetHeader className="pb-2">
          <SheetTitle className="flex items-center gap-2">
            Bài giảng — Trang {page}
            {isEdited && (
              <Badge variant="secondary" className="font-normal">
                Đã sửa
              </Badge>
            )}
          </SheetTitle>
          <SheetDescription>
            {editing
              ? "Sửa trực tiếp; thay đổi tự lưu trên thiết bị này."
              : "Giải thích nội dung trang này bằng tiếng Việt."}
          </SheetDescription>
        </SheetHeader>

        <div className="flex items-center gap-2 px-4 pb-2">
          <Button
            variant={editing ? "default" : "outline"}
            size="sm"
            onClick={() => setEditing((v) => !v)}
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
          <div className="flex min-h-0 flex-1 flex-col px-4 pb-6">
            <NoteEditor
              key={editorNonce}
              bookId={bookId}
              page={page}
              initialHtml={html ?? ""}
            />
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto px-4 pb-8">
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
      </SheetContent>
    </Sheet>
  );
}
