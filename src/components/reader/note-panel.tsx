"use client";

import { useCallback, useState } from "react";
import dynamic from "next/dynamic";
import { Eye, Pencil, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { NOTE_PROSE_CLASS, useEffectiveNote, useNoteStore } from "@/lib/note-store";
import { useNoteWidgetStore, type NoteSide } from "@/lib/note-widget-store";
import { countItems, getPageQuiz } from "@/lib/quiz";
import { QuizPanel } from "@/components/reader/quiz-panel";
import { cn } from "@/lib/utils";

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
 *
 * `pages` có 1 phần tử (chế độ 1 trang / cửa sổ riêng) hoặc 2 phần tử [trái,
 * phải] (chế độ 2 trang) — có 2 trang thì thêm 1 dải chọn "Trang N" phía
 * trên, mỗi trang lại có tối đa 2 tab con (Bài giảng/Bài tập) như cũ, nên
 * tổng cộng tối đa 4 tổ hợp chọn được khi trang nào cũng có bài tập.
 */
export function NotePanel({
  bookId,
  pages,
  noteContentByPage,
}: {
  bookId: string;
  pages: number[];
  /** HTML bài giảng gốc theo từng trang trong `pages`; null = trang chưa biên soạn. */
  noteContentByPage: Record<number, string | null>;
}) {
  const leftPage = pages[0];
  const rightPage = pages[1] ?? pages[0];

  const leftNote = useEffectiveNote(bookId, leftPage, noteContentByPage[leftPage] ?? null);
  const rightNote = useEffectiveNote(
    bookId,
    rightPage,
    noteContentByPage[rightPage] ?? null
  );
  const leftQuiz = getPageQuiz(bookId, leftPage);
  const rightQuiz = pages.length === 2 ? getPageQuiz(bookId, rightPage) : [];

  const resetNote = useNoteStore((s) => s.resetNote);
  const editing = useNoteWidgetStore((s) => s.editing);
  const setEditing = useNoteWidgetStore((s) => s.setEditing);
  const storedTab = useNoteWidgetStore((s) => s.tab);
  const setTab = useNoteWidgetStore((s) => s.setTab);
  const storedSide = useNoteWidgetStore((s) => s.side);
  const setSide = useNoteWidgetStore((s) => s.setSide);

  const side: NoteSide = pages.length === 2 ? storedSide : "left";
  const page = side === "right" ? rightPage : leftPage;
  const { html, isEdited, hasOriginal, hasContent } =
    side === "right" ? rightNote : leftNote;
  const sections = side === "right" ? rightQuiz : leftQuiz;

  // Trang không có bài tập thì không hiện tab đó — đỡ bày ra 1 tab rỗng. Nếu
  // đang đứng ở tab bài tập mà đổi sang trang/side không có bài tập thì rơi
  // về bài giảng, không kẹt ở màn trống.
  const hasQuiz = sections.length > 0;
  const tab = hasQuiz ? storedTab : "note";

  // Bấm "Khôi phục bản gốc" lúc đang sửa: remount editor để nó nạp lại nội
  // dung gốc — Tiptap không tự theo dõi prop `content` sau khi khởi tạo.
  const [editorNonce, setEditorNonce] = useState(0);

  const handleReset = useCallback(() => {
    resetNote(bookId, page);
    setEditorNonce((n) => n + 1);
  }, [resetNote, bookId, page]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {pages.length === 2 && (
        <div role="tablist" className="flex gap-1 border-b border-border px-3 pt-2">
          <PageTabButton
            selected={side === "left"}
            onSelect={() => setSide("left")}
            page={leftPage}
            hasContent={leftNote.hasContent || leftQuiz.length > 0}
          />
          <PageTabButton
            selected={side === "right"}
            onSelect={() => setSide("right")}
            page={rightPage}
            hasContent={rightNote.hasContent || rightQuiz.length > 0}
          />
        </div>
      )}

      {hasQuiz && (
        <div role="tablist" className="flex gap-1 border-b border-border px-3">
          <TabButton
            selected={tab === "note"}
            onSelect={() => setTab("note")}
            label="Bài giảng"
          />
          <TabButton
            selected={tab === "quiz"}
            onSelect={() => setTab("quiz")}
            label="Bài tập"
            badge={countItems(sections)}
          />
        </div>
      )}

      {tab === "quiz" ? (
        <QuizPanel bookId={bookId} page={page} sections={sections} />
      ) : (
        // Đổi tab/trang thì editor unmount — an toàn vì phần gõ dở được ghi
        // nốt ở cleanup của nó (xem note-editor), và lúc quay lại nó nạp
        // đúng bản vừa lưu.
        <>
          <div className="flex items-center gap-2 px-3 py-2">
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
              {/* key gồm cả trang: lật trang/đổi side lúc đang soạn thì editor
                  nạp lại nội dung trang mới, và bản đang gõ dở của trang cũ
                  được ghi nốt ở cleanup của editor cũ (xem note-editor). */}
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
                // Nội dung chỉ đến từ content/notes (do mình biên soạn) hoặc
                // từ chính localStorage của người dùng — không có nguồn bên
                // thứ ba nên không cần sanitize.
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
        </>
      )}
    </div>
  );
}

function PageTabButton({
  selected,
  onSelect,
  page,
  hasContent,
}: {
  selected: boolean;
  onSelect: () => void;
  page: number;
  hasContent: boolean;
}) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={selected}
      onClick={onSelect}
      className={cn(
        "relative rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
        selected
          ? "bg-primary text-primary-foreground"
          : "bg-muted text-muted-foreground hover:text-foreground"
      )}
    >
      Trang {page}
      {hasContent && (
        <span
          className={cn(
            "absolute -right-0.5 -top-0.5 size-1.5 rounded-full",
            selected ? "bg-primary-foreground" : "bg-sky-400"
          )}
          aria-hidden
        />
      )}
    </button>
  );
}

function TabButton({
  selected,
  onSelect,
  label,
  badge,
}: {
  selected: boolean;
  onSelect: () => void;
  label: string;
  badge?: number;
}) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={selected}
      onClick={onSelect}
      className={cn(
        "-mb-px flex items-center gap-1.5 border-b-2 px-2 py-2 text-sm transition-colors",
        selected
          ? "border-primary font-medium text-foreground"
          : "border-transparent text-muted-foreground hover:text-foreground"
      )}
    >
      {label}
      {badge !== undefined && (
        <span className="rounded-full bg-muted px-1.5 text-[11px] tabular-nums text-muted-foreground">
          {badge}
        </span>
      )}
    </button>
  );
}
