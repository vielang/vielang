"use client";

import { ExternalLink, Minimize2 } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { NotePanel } from "@/components/reader/note-panel";
import { useEffectiveNote } from "@/lib/note-store";
import { useNoteWidgetStore } from "@/lib/note-widget-store";
import { openNoteWindow } from "@/lib/note-window";

/**
 * Bài giảng ở chế độ toàn màn hình — hợp với điện thoại, nơi panel nổi quá
 * chật. Đổi qua lại với panel nổi bằng nút thu nhỏ; nội dung dùng chung
 * `NotePanel` nên hai chế độ không lệch hành vi.
 */
export function NoteSheet({
  bookId,
  page,
  originalContent,
}: {
  bookId: string;
  page: number;
  /** HTML bài giảng gốc (content/notes); null = trang này chưa biên soạn. */
  originalContent: string | null;
}) {
  const mode = useNoteWidgetStore((s) => s.mode);
  const setMode = useNoteWidgetStore((s) => s.setMode);
  const { isEdited } = useEffectiveNote(bookId, page, originalContent);

  return (
    <Sheet
      open={mode === "fullscreen"}
      onOpenChange={(next) => {
        if (!next) setMode("closed");
      }}
    >
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
            Giải thích nội dung trang này bằng tiếng Việt.
          </SheetDescription>
          {/* Nút đóng mặc định của SheetContent nằm ở góc phải trên, nên chừa
              chỗ cho nó bằng `pr-10`. */}
          <div className="absolute top-3 right-10 flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => {
                if (openNoteWindow(bookId, page)) setMode("popped");
              }}
              aria-label="Mở ra cửa sổ riêng"
              title="Mở ra cửa sổ riêng"
            >
              <ExternalLink className="size-4" aria-hidden />
            </Button>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => setMode("floating")}
              aria-label="Thu về panel nổi"
              title="Thu về panel nổi"
            >
              <Minimize2 className="size-4" aria-hidden />
            </Button>
          </div>
        </SheetHeader>

        <NotePanel bookId={bookId} page={page} originalContent={originalContent} />
      </SheetContent>
    </Sheet>
  );
}
