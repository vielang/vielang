"use client";

import { Fragment } from "react";
import Image from "next/image";
import Link from "next/link";
import {
  BookOpen,
  Bookmark,
  Check,
  Languages,
  ListChecks,
  NotebookText,
  Volume2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { getThumbUrl } from "@/lib/books";
import { useProgressStore } from "@/lib/progress-store";
import { answersHidden, useReaderPrefsStore } from "@/lib/reader-prefs-store";
import type { Chapter } from "@/lib/chapters";
import { useIsClient } from "@/lib/use-is-client";

function chapterAnchor(lesson: number): string {
  return `bai-${lesson}`;
}

/**
 * Thanh nhảy nhanh tới từng bài — sticky ngay dưới header, cần thiết khi
 * sách có 130-250 trang (16-18 bài) nên cuộn tay để tìm 1 bài cụ thể rất
 * chậm. Ẩn hẳn nếu sách không có dữ liệu bài học (chapters rỗng).
 */
function ChapterNav({ chapters }: { chapters: Chapter[] }) {
  if (chapters.length === 0) return null;
  return (
    <nav
      aria-label="Nhảy nhanh tới bài học"
      className="sticky top-14 z-30 -mx-4 flex gap-1.5 overflow-x-auto border-b border-border bg-background/95 px-4 py-2 backdrop-blur supports-[backdrop-filter]:bg-background/80"
    >
      {chapters.map((ch) => (
        <a
          key={ch.lesson}
          href={`#${chapterAnchor(ch.lesson)}`}
          onClick={(e) => {
            // Cuộn mượt CHỈ cho thao tác bấm chip trong cùng trang này — cố
            // ý không dùng CSS scroll-behavior:smooth toàn cục, vì nó cũng
            // áp cho lúc mới điều hướng tới đây kèm hash (vd đóng trang đọc
            // trả về đúng bài đang đọc dở), lúc đó phải tới thẳng vị trí
            // ngay, không cuộn từ đầu trang xuống.
            e.preventDefault();
            const el = document.getElementById(chapterAnchor(ch.lesson));
            el?.scrollIntoView({ behavior: "smooth", block: "start" });
            history.pushState(null, "", `#${chapterAnchor(ch.lesson)}`);
          }}
          className="shrink-0 rounded-full border border-border px-2.5 py-1 text-xs font-medium tabular-nums text-muted-foreground transition-colors hover:border-primary hover:text-foreground"
        >
          Bài {ch.lesson}
        </a>
      ))}
    </nav>
  );
}

export function PageGrid({
  bookId,
  totalPages,
  chapters,
  notePages,
  audioPages,
  translatedPages,
  grammarPages,
  answerPages,
}: {
  bookId: string;
  totalPages: number;
  /** Ranh giới bài học (xem lib/chapters.ts) — [] nếu sách không xác định được */
  chapters: Chapter[];
  /** Số trang có sẵn bài giảng (xem lib/notes.ts) */
  notePages: number[];
  /** Số trang có sẵn audio (xem lib/audio.ts) */
  audioPages: number[];
  /** Số trang có sẵn bản dịch (xem lib/page-translation.ts) */
  translatedPages: number[];
  /** Số trang có sẵn nghĩa ngữ pháp (xem lib/page-grammar.ts) */
  grammarPages: number[];
  /** Số trang có đáp án sách (xem lib/page-answers.ts) */
  answerPages: number[];
}) {
  const books = useProgressStore((s) => s.books);
  const toggleBookmark = useProgressStore((s) => s.toggleBookmark);
  const readPages = new Set(books[bookId]?.readPages ?? []);
  const bookmarks = new Set(books[bookId]?.bookmarks ?? []);
  const notedPages = new Set(notePages);
  const audioedPages = new Set(audioPages);
  const translatedSet = new Set(translatedPages);
  const grammarSet = new Set(grammarPages);
  // Tắt chấm đáp án của cuốn này thì huy hiệu cũng thôi hiện — nó hứa một thứ
  // người dùng sẽ không thấy khi mở trang ra. Chỉ đọc sau khi đã ở trình
  // duyệt, cùng lý do với `AnswersToggle` (tránh lệch hydration).
  const isClient = useIsClient();
  const answersOff = useReaderPrefsStore((s) => answersHidden(s, bookId));
  const answerSet = new Set(isClient && answersOff ? [] : answerPages);
  const chapterByStartPage = new Map(chapters.map((ch) => [ch.startPage, ch]));

  const pages = Array.from({ length: totalPages }, (_, i) => i + 1);

  return (
    <div className="flex flex-col gap-4">
      <ChapterNav chapters={chapters} />

      <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
        {pages.map((page) => {
          const isRead = readPages.has(page);
          const isBookmarked = bookmarks.has(page);
          const hasNote = notedPages.has(page);
          const hasAudio = audioedPages.has(page);
          const hasTranslation = translatedSet.has(page);
          const hasGrammar = grammarSet.has(page);
          const hasAnswers = answerSet.has(page);
          const chapter = chapterByStartPage.get(page);

          return (
            <Fragment key={page}>
              {chapter && (
                <div
                  id={chapterAnchor(chapter.lesson)}
                  className="col-span-full flex items-baseline gap-2 pt-3 pb-0.5 scroll-mt-28 first:pt-0"
                >
                  <span className="text-sm font-semibold tracking-tight">
                    Bài {chapter.lesson}
                  </span>
                  <span className="text-xs text-muted-foreground tabular-nums">
                    trang {chapter.startPage}–{chapter.endPage}
                  </span>
                  <span className="h-px flex-1 bg-border" aria-hidden />
                </div>
              )}
              <div className="group relative">
                <Link
                  href={`/read/${bookId}/${page}`}
                  // Không tải trước: lưới có hàng trăm ô, để mặc định là mỗi ô
                  // hiện trong màn hình tự xin trước trang đọc của nó — vài
                  // chục request (mỗi cái máy chủ phải dựng một trang) mỗi lần
                  // mở sách, trong khi người ta chỉ bấm một ô.
                  prefetch={false}
                  className="focus-visible:ring-ring block overflow-hidden rounded-lg border border-border bg-muted transition-shadow hover:shadow-md focus-visible:ring-2 focus-visible:outline-none"
                >
                  <div className="relative aspect-[192/250] w-full">
                    <Image
                      src={getThumbUrl(bookId, page)}
                      alt={`Trang ${page}`}
                      fill
                      unoptimized
                      sizes="(min-width: 1024px) 15vw, (min-width: 640px) 22vw, 30vw"
                      className={cn(
                        "object-cover transition-opacity",
                        isRead && "opacity-70"
                      )}
                      loading={page <= 12 ? "eager" : "lazy"}
                    />
                    {isRead && (
                      <span className="absolute right-1 bottom-1 rounded-full bg-primary p-0.5 text-primary-foreground shadow">
                        <Check className="size-3" aria-hidden />
                      </span>
                    )}
                    {(hasNote || hasAudio || hasTranslation || hasGrammar || hasAnswers) && (
                      <div className="absolute top-1 left-1 flex gap-1">
                        {hasNote && (
                          <span
                            className="rounded-full bg-background/95 p-1 text-primary shadow ring-1 ring-border"
                            title="Có bài giảng"
                          >
                            <NotebookText className="size-3" aria-hidden />
                          </span>
                        )}
                        {hasAudio && (
                          <span
                            className="rounded-full bg-background/95 p-1 text-primary shadow ring-1 ring-border"
                            title="Có audio"
                          >
                            <Volume2 className="size-3" aria-hidden />
                          </span>
                        )}
                        {hasTranslation && (
                          <span
                            className="rounded-full bg-background/95 p-1 text-primary shadow ring-1 ring-border"
                            title="Có bản dịch tiếng Việt"
                          >
                            <Languages className="size-3" aria-hidden />
                          </span>
                        )}
                        {/* Cùng biểu tượng với chấm trên trang đọc: người
                            dùng thấy ở đây rồi mở trang ra là nhận ra ngay
                            phải tìm cái gì. */}
                        {hasGrammar && (
                          <span
                            className="rounded-full bg-background/95 p-1 text-primary shadow ring-1 ring-border"
                            title="Có giải nghĩa ngữ pháp"
                          >
                            <BookOpen className="size-3" aria-hidden />
                          </span>
                        )}
                        {hasAnswers && (
                          <span
                            className="rounded-full bg-background/95 p-1 text-primary shadow ring-1 ring-border"
                            title="Có đáp án sách"
                          >
                            <ListChecks className="size-3" aria-hidden />
                          </span>
                        )}
                      </div>
                    )}
                    <span className="absolute bottom-1 left-1.5 rounded bg-background/80 px-1 text-[10px] tabular-nums text-foreground">
                      {page}
                    </span>
                  </div>
                </Link>

                <button
                  type="button"
                  aria-label={isBookmarked ? "Bỏ đánh dấu trang" : "Đánh dấu trang"}
                  onClick={() => toggleBookmark(bookId, page)}
                  // Luôn hiện, chỉ mờ đi khi chưa đánh dấu. Trước đây nút
                  // này `opacity-0` và chỉ hiện khi rê chuột — trên điện
                  // thoại không có rê chuột nên nó VÔ HÌNH MÀ VẪN BẤM ĐƯỢC:
                  // chạm hụt vào góc phải trên một trang là đánh dấu nhầm mà
                  // không hiểu vì sao.
                  className={cn(
                    "absolute top-1 right-1 rounded-full bg-background/90 p-1.5 shadow transition-opacity",
                    isBookmarked
                      ? "text-primary opacity-100"
                      : "text-muted-foreground opacity-60 hover:opacity-100 focus-visible:opacity-100"
                  )}
                >
                  <Bookmark
                    className="size-4"
                    fill={isBookmarked ? "currentColor" : "none"}
                    aria-hidden
                  />
                </button>
              </div>
            </Fragment>
          );
        })}
      </div>
    </div>
  );
}
