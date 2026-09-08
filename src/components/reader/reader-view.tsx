"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import type { Book } from "@/lib/books";
import { useProgressStore } from "@/lib/progress-store";
import { PageViewer, type PageViewerHandle } from "@/components/reader/page-viewer";
import { ReaderControls } from "@/components/reader/reader-controls";
import { AdjacentPreload } from "@/components/reader/adjacent-preload";

const INTERACTIVE_TAGS = new Set(["INPUT", "TEXTAREA", "SELECT"]);

export function ReaderView({
  book,
  page,
  noteContent,
}: {
  book: Book;
  page: number;
  /** null = trang này chưa có bài giảng. */
  noteContent: string | null;
}) {
  const router = useRouter();
  const viewerRef = useRef<PageViewerHandle>(null);
  const hasNote = noteContent !== null;

  const markPageRead = useProgressStore((s) => s.markPageRead);
  const toggleBookmark = useProgressStore((s) => s.toggleBookmark);
  const bookmarks = useProgressStore((s) => s.books[book.id]?.bookmarks);
  const isBookmarked = (bookmarks ?? []).includes(page);

  const [toolbarVisible, setToolbarVisible] = useState(true);
  const [jumpOpen, setJumpOpen] = useState(false);
  const [noteOpen, setNoteOpen] = useState(false);

  const goTo = useCallback(
    (target: number) => {
      const clamped = Math.min(Math.max(target, 1), book.totalPages);
      if (clamped !== page) router.push(`/read/${book.id}/${clamped}`);
    },
    [book.id, book.totalPages, page, router]
  );

  // Ghi nhận đã đọc trang này. `toolbarVisible` không cần reset thủ công ở
  // đây — page.tsx render <ReaderView key={page}/>, remount mỗi khi đổi
  // trang nên state cục bộ (toolbar, dialog...) tự về mặc định.
  useEffect(() => {
    markPageRead(book.id, page);
  }, [book.id, page, markPageRead]);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      // Dialog nhảy trang / Sheet bài giảng đang mở (Slider/Input/Esc riêng
      // của Radix) — nhường toàn bộ phím tắt, tránh vừa đóng vừa chuyển trang.
      if (jumpOpen || noteOpen) return;
      const target = e.target as HTMLElement | null;
      if (target && INTERACTIVE_TAGS.has(target.tagName)) return;

      switch (e.key) {
        case "ArrowRight":
          goTo(page + 1);
          break;
        case "ArrowLeft":
          goTo(page - 1);
          break;
        case "Home":
          goTo(1);
          break;
        case "End":
          goTo(book.totalPages);
          break;
        case "Escape":
          router.push(`/books/${book.id}`);
          break;
        case "b":
        case "B":
          toggleBookmark(book.id, page);
          break;
        case "n":
        case "N":
          if (hasNote) setNoteOpen(true);
          break;
        default:
          return;
      }
      e.preventDefault();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    goTo,
    page,
    book.id,
    book.totalPages,
    router,
    toggleBookmark,
    jumpOpen,
    noteOpen,
    hasNote,
  ]);

  return (
    <div className="fixed inset-0 overflow-hidden bg-black">
      <AdjacentPreload bookId={book.id} page={page} totalPages={book.totalPages} />

      <PageViewer
        ref={viewerRef}
        bookId={book.id}
        page={page}
        onTap={() => setToolbarVisible((v) => !v)}
        onSwipePrev={() => goTo(page - 1)}
        onSwipeNext={() => goTo(page + 1)}
      />

      <ReaderControls
        visible={toolbarVisible}
        book={book}
        page={page}
        isBookmarked={isBookmarked}
        jumpOpen={jumpOpen}
        onJumpOpenChange={setJumpOpen}
        hasNote={hasNote}
        noteContent={noteContent}
        noteOpen={noteOpen}
        onNoteOpenChange={setNoteOpen}
        onPrev={() => goTo(page - 1)}
        onNext={() => goTo(page + 1)}
        onJump={goTo}
        onToggleBookmark={() => toggleBookmark(book.id, page)}
        onResetZoom={() => viewerRef.current?.resetZoom()}
      />
    </div>
  );
}
