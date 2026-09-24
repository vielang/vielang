"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import type { Book } from "@/lib/books";
import { useProgressStore } from "@/lib/progress-store";
import { useEffectiveNote, useNoteStore } from "@/lib/note-store";
import { getAudioPages, getPageAudio } from "@/lib/audio";
import { buildPlaylist, startIndexFor } from "@/lib/autoplay";
import { startAutoplay, stopAutoplay, useAutoplayStore } from "@/lib/autoplay-player";
import { AutoplayBar } from "@/components/reader/autoplay-bar";
import {
  getAdjacentSpreadAnchor,
  getChapters,
  getSpreadAnchor,
  getSpreadPages,
} from "@/lib/chapters";
import { useReaderPrefsStore } from "@/lib/reader-prefs-store";
import { useIsWideScreen } from "@/lib/use-wide-screen";
import { PageViewer, type PageViewerHandle } from "@/components/reader/page-viewer";
import { ReaderControls } from "@/components/reader/reader-controls";
import { AdjacentPreload } from "@/components/reader/adjacent-preload";
import { AudioWidget } from "@/components/reader/audio-widget";
import { NoteWidget } from "@/components/reader/note-widget";
import { NoteSheet } from "@/components/reader/note-sheet";
import { AnnotationToolbar } from "@/components/reader/annotation-toolbar";
import { RecorderWidget } from "@/components/reader/recorder-widget";
import { ReaderHelp } from "@/components/reader/reader-help";
import { useStudyTracker } from "@/lib/use-study-tracker";
import { nextZoom, useZoomStore } from "@/lib/zoom-store";
import { ZoomBar } from "@/components/reader/zoom-bar";
import { useNoteWidgetStore } from "@/lib/note-widget-store";
import {
  useAnnotationHydration,
  useAnnotationStore,
  useHasAnnotations,
} from "@/lib/annotation-store";
import {
  useHasRecordings,
  useRecordingHydration,
  useRecordingStore,
} from "@/lib/recording-store";
import {
  focusNoteWindow,
  isNoteWindowClosed,
  publishNoteFocus,
  useNoteStoreSync,
} from "@/lib/note-window";

const INTERACTIVE_TAGS = new Set(["INPUT", "TEXTAREA", "SELECT"]);

export function ReaderView({
  book,
  page,
  noteContentByPage,
}: {
  book: Book;
  page: number;
  /** HTML bài giảng gốc của trang này VÀ trang kế (trang phải khi ở chế độ 2 trang); null = trang chưa biên soạn. */
  noteContentByPage: Record<number, string | null>;
}) {
  const router = useRouter();
  const viewerRef = useRef<PageViewerHandle>(null);
  // Audio của CẢ 2 trang có thể hiện (trang này + trang kế) — AudioWidget tự
  // chọn trang nào có audio để hiện, giống cách NotePanel xử lý noteContentByPage.
  const audioTracksByPage = {
    [page]: getPageAudio(book.id, page),
    [page + 1]: getPageAudio(book.id, page + 1),
  };
  const chapters = getChapters(book.id, book.totalPages);
  const currentChapter = chapters.find(
    (ch) => page >= ch.startPage && page <= ch.endPage
  );

  // Chế độ xem 1/2 trang — tuỳ chọn chung cho mọi sách (persist), nhưng chỉ
  // thực sự bật khi màn hình đủ rộng để 2 trang còn đọc được. `page` (từ URL)
  // luôn được coi là trang trái/anchor của spread khi ở chế độ 2 trang — mọi
  // đường điều hướng trong app (goTo/stepNext/stepPrev/nút đổi chế độ) đều tự
  // chuẩn hoá về đúng anchor trước khi đổi URL, nên bất biến này luôn đúng
  // trừ khi người dùng tự sửa tay số trang trên URL (trường hợp hiếm, tự
  // phục hồi ngay lần điều hướng kế tiếp).
  const pageLayout = useReaderPrefsStore((s) => s.pageLayout);
  const setPageLayout = useReaderPrefsStore((s) => s.setPageLayout);
  const isWideScreen = useIsWideScreen();
  const effectiveDouble = pageLayout === "double" && isWideScreen;
  const pages = effectiveDouble
    ? getSpreadPages(book.id, page, book.totalPages)
    : [page];

  /**
   * Spread liền trước và liền sau, để trình xem vẽ sẵn hai bên mà trượt
   * sang khi lật.
   *
   * Tính bằng ĐÚNG mấy hàm mà stepPrev/stepNext dùng, nên cái hiện ra lúc
   * lật không bao giờ lệch với trang sẽ mở. `getAdjacentSpreadAnchor` tự
   * kẹp ở hai đầu sách và trả lại chính nó, nên so với `page` là biết đã
   * hết trang.
   */
  const spreadAt = (anchor: number) =>
    effectiveDouble ? getSpreadPages(book.id, anchor, book.totalPages) : [anchor];
  const prevAnchor = effectiveDouble
    ? getAdjacentSpreadAnchor(book.id, page, book.totalPages, -1)
    : page - 1;
  const nextAnchor = effectiveDouble
    ? getAdjacentSpreadAnchor(book.id, page, book.totalPages, 1)
    : page + 1;
  const prevPages =
    prevAnchor >= 1 && prevAnchor !== page ? spreadAt(prevAnchor) : null;
  const nextPages =
    nextAnchor <= book.totalPages && nextAnchor !== page
      ? spreadAt(nextAnchor)
      : null;

  // Bài giảng có thể đã được người dùng sửa/tự viết rồi lưu ở localStorage —
  // chờ rehydrate xong mới hiện chấm báo, tránh lệch với HTML server render.
  // Gọi useEffectiveNote cố định cho CẢ 2 trang (page, page+1) dù đang ở chế
  // độ 1 trang — số lần gọi hook phải cố định giữa các lượt render, không
  // được rẽ nhánh theo effectiveDouble.
  const notesHydrated = useNoteStore((s) => s.hasHydrated);
  const { hasContent: hasContentLeft } = useEffectiveNote(
    book.id,
    page,
    noteContentByPage[page] ?? null
  );
  const { hasContent: hasContentRight } = useEffectiveNote(
    book.id,
    page + 1,
    noteContentByPage[page + 1] ?? null
  );
  const noteHasContent =
    notesHydrated &&
    (hasContentLeft || (pages.length === 2 && hasContentRight));

  const markPageRead = useProgressStore((s) => s.markPageRead);
  const toggleBookmark = useProgressStore((s) => s.toggleBookmark);
  const bookmarks = useProgressStore((s) => s.books[book.id]?.bookmarks);
  const isBookmarked = (bookmarks ?? []).includes(page);

  const [toolbarVisible, setToolbarVisible] = useState(true);
  const [jumpOpen, setJumpOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);

  const noteMode = useNoteWidgetStore((s) => s.mode);
  const setNoteMode = useNoteWidgetStore((s) => s.setMode);

  // Nét vẽ tay trên ảnh trang — nạp từ localStorage sau lần render đầu.
  useAnnotationHydration();
  const drawActive = useAnnotationStore((s) => s.active);
  const setDrawActive = useAnnotationStore((s) => s.setActive);
  const drawHasContent = useHasAnnotations(book.id, pages);

  // Ghi âm giọng người dùng theo từng trang — cùng cách nạp với nét vẽ.
  useRecordingHydration();
  const recordOpen = useRecordingStore((s) => s.open);
  const setRecordOpen = useRecordingStore((s) => s.setOpen);
  const recordHasContent = useHasRecordings(book.id, pages);

  const toggleDraw = useCallback(() => {
    // Bật chế độ vẽ mà thanh công cụ đang ẩn thì không còn đường tắt lại:
    // trong chế độ vẽ, chạm lên trang là vẽ chứ không hiện thanh nữa.
    setDrawActive(!drawActive);
    if (!drawActive) setToolbarVisible(true);
  }, [drawActive, setDrawActive]);

  // Sửa bài giảng ở cửa sổ note riêng thì cửa sổ này phải thấy ngay.
  useNoteStoreSync();

  // Công bố trang đang đọc để cửa sổ note riêng bám theo khi lật trang.
  useEffect(() => {
    publishNoteFocus(book.id, page);
  }, [book.id, page]);

  // Người dùng đóng cửa sổ note bằng nút X của trình duyệt: không có sự kiện
  // nào báo về, nên hỏi `window.closed` theo nhịp để trả panel về trạng thái
  // đóng — nếu không nút bài giảng sẽ mãi focus vào 1 cửa sổ đã chết.
  useEffect(() => {
    if (noteMode !== "popped") return;
    const id = setInterval(() => {
      if (isNoteWindowClosed()) setNoteMode("closed");
    }, 1000);
    return () => clearInterval(id);
  }, [noteMode, setNoteMode]);

  const toggleNote = useCallback(() => {
    if (noteMode === "popped") {
      focusNoteWindow(); // đang ở cửa sổ riêng — đưa cửa sổ đó lên trước
      return;
    }
    setNoteMode(noteMode === "closed" ? "floating" : "closed");
  }, [noteMode, setNoteMode]);

  const noteLabel =
    noteMode === "popped"
      ? "Bài giảng đang ở cửa sổ riêng"
      : noteMode !== "closed"
        ? "Đóng bài giảng"
        : noteHasContent
          ? "Bài giảng trang này"
          : "Soạn bài giảng cho trang này";

  const goTo = useCallback(
    (target: number) => {
      const clamped = Math.min(Math.max(target, 1), book.totalPages);
      const dest = effectiveDouble ? getSpreadAnchor(book.id, clamped) : clamped;
      if (dest !== page) router.push(`/read/${book.id}/${dest}`);
    },
    [book.id, book.totalPages, page, router, effectiveDouble]
  );

  // Nạp sẵn route hai bên: `router.push` của trang chưa nạp phải chờ tải
  // payload, đúng vào lúc hiệu ứng lật vừa xong nên thấy khựng. Ảnh đã có
  // `AdjacentPreload` lo.
  useEffect(() => {
    for (const p of [...prevPages ?? [], ...nextPages ?? []]) {
      router.prefetch(`/read/${book.id}/${p}`);
    }
  }, [book.id, prevPages, nextPages, router]);

  // Lùi/tiến đúng 1 spread ở chế độ 2 trang (bước 1 hoặc 2 trang tuỳ spread
  // hiện tại dài bao nhiêu), hoặc đúng 1 trang ở chế độ 1 trang.
  const stepNext = useCallback(() => {
    goTo(
      effectiveDouble
        ? getAdjacentSpreadAnchor(book.id, page, book.totalPages, 1)
        : page + 1
    );
  }, [effectiveDouble, book.id, book.totalPages, page, goTo]);

  const stepPrev = useCallback(() => {
    goTo(
      effectiveDouble
        ? getAdjacentSpreadAnchor(book.id, page, book.totalPages, -1)
        : page - 1
    );
  }, [effectiveDouble, book.id, book.totalPages, page, goTo]);

  /**
   * Nút mũi tên và phím mũi tên đi qua cùng một hiệu ứng lật như vuốt tay.
   *
   * Không thì bấm nút là trang nhảy cái rụp còn vuốt thì trượt, hai lối
   * điều hướng cho cùng một việc mà cảm giác khác hẳn nhau.
   *
   * `turn` trả về false khi hết sách hoặc khung chưa đo xong — lúc đó cứ
   * chuyển trang kiểu cũ, thà không có hiệu ứng còn hơn đứng im.
   */
  const turnNext = useCallback(() => {
    if (!viewerRef.current?.turn(1)) stepNext();
  }, [stepNext]);

  const turnPrev = useCallback(() => {
    if (!viewerRef.current?.turn(-1)) stepPrev();
  }, [stepPrev]);

  const togglePageLayout = useCallback(() => {
    const next = pageLayout === "double" ? "single" : "double";
    setPageLayout(next);
    // Bật 2 trang mà đang đứng ở trang phải: chuẩn hoá URL về trang trái của
    // spread luôn — không gọi goTo() vì nó còn đóng gói effectiveDouble của
    // lượt render TRƯỚC khi setPageLayout có hiệu lực (stale closure).
    if (next === "double" && isWideScreen) {
      const anchor = getSpreadAnchor(book.id, page);
      if (anchor !== page) router.push(`/read/${book.id}/${anchor}`);
    }
  }, [pageLayout, setPageLayout, isWideScreen, book.id, page, router]);

  // Ghi nhận đã đọc CẢ 2 trang của spread (nếu đang ở chế độ 2 trang).
  // `toolbarVisible` không cần reset thủ công ở đây — page.tsx render
  // <ReaderView key={page}/>, remount mỗi khi đổi trang nên state cục bộ
  // (toolbar, dialog...) tự về mặc định.
  // Lịch sử học cho My page: thời gian học thật và trang ở lại đủ lâu —
  // khác `markPageRead` ngay dưới, vốn ghi ngay khi trang vừa hiện ra.
  useStudyTracker(book.id, pages);

  const pagesKey = pages.join(",");
  useEffect(() => {
    pages.forEach((p) => markPageRead(book.id, p));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- so sánh theo pagesKey (chuỗi) để khỏi chạy lại mỗi lần render do `pages` là array mới mỗi lượt.
  }, [book.id, pagesKey, markPageRead]);

  // Nghe tự động: báo trang đang hiện lên để bài kế tiếp biết có phải lật
  // trang không (xem AutoplayFollower).
  const hasBookAudio = getAudioPages(book.id).length > 0;
  const autoplayActive = useAutoplayStore((s) => s.bookId === book.id);
  const setAutoplayView = useAutoplayStore((s) => s.setView);
  useEffect(() => {
    setAutoplayView(pages, effectiveDouble);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- như trên, so theo pagesKey.
  }, [pagesKey, effectiveDouble, setAutoplayView]);

  // Bài kế nằm ở spread liền bên thì cho nghe tự động lật bằng đúng hiệu
  // ứng của trình đọc.
  const setAutoplayTurnTo = useAutoplayStore((s) => s.setTurnTo);
  useEffect(() => {
    setAutoplayTurnTo((dest) => {
      if (dest === nextAnchor) turnNext();
      else if (dest === prevAnchor) turnPrev();
      else return false;
      return true;
    });
    return () => setAutoplayTurnTo(null);
  }, [nextAnchor, prevAnchor, turnNext, turnPrev, setAutoplayTurnTo]);

  // Gọi thẳng trong click, không qua effect — xem đầu lib/autoplay-player.ts.
  const toggleAutoplay = useCallback(() => {
    if (autoplayActive) stopAutoplay();
    else startAutoplay(book.id, startIndexFor(buildPlaylist(book.id), page));
  }, [autoplayActive, book.id, page]);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      // Dialog nhảy trang / sheet bài giảng toàn màn hình đang mở (Slider/
      // Input/Esc riêng của Radix) — nhường toàn bộ phím tắt, tránh vừa đóng
      // vừa chuyển trang. Panel nổi thì KHÔNG chặn: mở bài giảng rồi vẫn lật
      // trang bằng phím mũi tên được, đó mới là điểm lợi của nó.
      if (jumpOpen || helpOpen || noteMode === "fullscreen") return;
      const target = e.target as HTMLElement | null;
      if (target && (INTERACTIVE_TAGS.has(target.tagName) || target.isContentEditable))
        return;

      switch (e.key) {
        case "ArrowRight":
          turnNext();
          break;
        case "ArrowLeft":
          turnPrev();
          break;
        case "Home":
          goTo(1);
          break;
        case "End":
          goTo(book.totalPages);
          break;
        case "Escape":
          // Đang vẽ thì Esc là "bỏ cây bút xuống", chưa phải "đóng sách" —
          // thoát hẳn khỏi trang đọc thì bấm Esc lần nữa.
          if (drawActive) {
            setDrawActive(false);
            break;
          }
          router.push(
            currentChapter
              ? `/books/${book.id}#bai-${currentChapter.lesson}`
              : `/books/${book.id}`
          );
          break;
        case "b":
        case "B":
          toggleBookmark(book.id, page);
          break;
        case "n":
        case "N":
          toggleNote();
          break;
        case "d":
        case "D":
          toggleDraw();
          break;
        case "?":
          setHelpOpen(true);
          break;
        // Phóng to/thu nhỏ từng nấc 25% và về 100%. Có Ctrl/⌘ thì là phóng to
        // CẢ TRÌNH DUYỆT — để nguyên cho trình duyệt xử lý.
        case "+":
        case "=":
        case "-":
        case "_":
        case "0": {
          if (e.ctrlKey || e.metaKey) return;
          const scale = useZoomStore.getState().scale;
          const target =
            e.key === "0" ? 1 : nextZoom(scale, e.key === "-" || e.key === "_" ? -1 : 1);
          viewerRef.current?.zoomTo(target);
          break;
        }
        default:
          return;
      }
      e.preventDefault();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    goTo,
    turnNext,
    turnPrev,
    page,
    book.id,
    book.totalPages,
    router,
    currentChapter,
    toggleBookmark,
    jumpOpen,
    helpOpen,
    noteMode,
    toggleNote,
    drawActive,
    setDrawActive,
    toggleDraw,
  ]);

  return (
    <div className="fixed inset-0 overflow-hidden bg-black">
      <AdjacentPreload bookId={book.id} pages={pages} totalPages={book.totalPages} />

      <PageViewer
        ref={viewerRef}
        wide={isWideScreen}
        book={book}
        pages={pages}
        prevPages={prevPages}
        nextPages={nextPages}
        onTap={() => setToolbarVisible((v) => !v)}
        onSwipePrev={stepPrev}
        onSwipeNext={stepNext}
      />

      <AudioWidget pages={pages} tracksByPage={audioTracksByPage} />
      <AutoplayBar bookId={book.id} toolbarVisible={toolbarVisible} />

      <NoteWidget bookId={book.id} pages={pages} noteContentByPage={noteContentByPage} />
      <NoteSheet bookId={book.id} pages={pages} noteContentByPage={noteContentByPage} />

      <AnnotationToolbar bookId={book.id} pages={pages} />
      {/* Ghi âm bám theo trang trái/anchor: một bản ghi thuộc về một trang,
          chứ không thuộc về cả spread. */}
      <RecorderWidget bookId={book.id} page={page} />

      {/* Tự hiện một lần ở lần mở sách đầu tiên — xem ReaderHelp. */}
      <ReaderHelp open={helpOpen} onOpenChange={setHelpOpen} />

      {isWideScreen && (
        <ZoomBar
          visible={toolbarVisible}
          onZoomTo={(scale) => viewerRef.current?.zoomTo(scale)}
        />
      )}
      <ReaderControls
        visible={toolbarVisible}
        book={book}
        page={page}
        pages={pages}
        chapters={chapters}
        currentLesson={currentChapter?.lesson ?? null}
        isBookmarked={isBookmarked}
        jumpOpen={jumpOpen}
        onJumpOpenChange={setJumpOpen}
        noteHasContent={noteHasContent}
        noteLabel={noteLabel}
        onNoteToggle={toggleNote}
        drawActive={drawActive}
        drawHasContent={drawHasContent}
        onDrawToggle={toggleDraw}
        recordOpen={recordOpen}
        recordHasContent={recordHasContent}
        onRecordToggle={() => setRecordOpen(!recordOpen)}
        showAutoplay={hasBookAudio}
        autoplayActive={autoplayActive}
        onAutoplayToggle={toggleAutoplay}
        onPrev={turnPrev}
        onNext={turnNext}
        onJump={goTo}
        onToggleBookmark={() => toggleBookmark(book.id, page)}
        onResetZoom={() => viewerRef.current?.resetZoom()}
        onOpenHelp={() => setHelpOpen(true)}
        showLayoutToggle={isWideScreen}
        pageLayout={pageLayout}
        onTogglePageLayout={togglePageLayout}
      />
    </div>
  );
}
