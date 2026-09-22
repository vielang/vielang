"use client";

import Link from "next/link";
import { cn } from "@/lib/utils";
import type { Book } from "@/lib/books";
import { getChapters } from "@/lib/chapters";
import { shortBookName } from "@/lib/companion";
import { getBookProgress, resumePage, type BookProgress } from "@/lib/progress-store";
import { HEAT_CLASSES } from "@/components/my/heat";

/** 0–4, cùng thang với lịch học: chưa học / ít / một phần / gần xong / xong. */
function lessonLevel(done: number, total: number): 0 | 1 | 2 | 3 | 4 {
  if (done === 0 || total === 0) return 0;
  if (done >= total) return 4;
  const ratio = done / total;
  return ratio < 1 / 3 ? 1 : ratio < 2 / 3 ? 2 : 3;
}

export interface BookRow {
  book: Book;
  progress: BookProgress;
  studied: number[];
}

/** Sách đã từng mở, gần đây nhất lên đầu. */
export function booksInProgress(
  books: readonly Book[],
  progressByBook: Record<string, BookProgress>,
  studiedByBook: Record<string, number[]>
): BookRow[] {
  return books
    .map((book) => ({
      book,
      progress: getBookProgress(progressByBook, book.id),
      studied: studiedByBook[book.id] ?? [],
    }))
    .filter((r) => r.progress.readPages.length > 0 || r.studied.length > 0)
    .sort((a, b) => b.progress.updatedAt.localeCompare(a.progress.updatedAt));
}

/**
 * Tiến độ một cuốn: số trang ĐÃ HỌC (ở lại đọc đủ lâu) và một hàng ô, mỗi ô
 * một bài, đậm theo phần trang đã học của bài đó — nhìn là biết bài nào còn
 * dở. Cả thẻ bấm được để học tiếp đúng trang đang dở.
 */
export function BookProgressCard({ row }: { row: BookRow }) {
  const { book, progress, studied } = row;
  const studiedSet = new Set(studied);
  const percent = Math.round((studied.length / book.totalPages) * 100);
  const chapters = getChapters(book.id, book.totalPages);
  const resume = resumePage(progress);

  return (
    <div className="flex flex-col gap-3 rounded-xl border border-border p-4">
      <div className="flex items-baseline justify-between gap-3">
        <h3 className="min-w-0 truncate font-medium" title={book.titleVi}>
          {shortBookName(book)}
        </h3>
        <span className="shrink-0 text-sm tabular-nums text-muted-foreground">
          {studied.length}/{book.totalPages} trang
        </span>
      </div>

      <div
        className="h-1.5 overflow-hidden rounded-full bg-muted"
        role="progressbar"
        aria-valuenow={percent}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={`Đã học ${percent}% cuốn này`}
      >
        <div className="h-full rounded-full bg-primary" style={{ width: `${percent}%` }} />
      </div>

      {chapters.length > 0 && (
        <div className="flex flex-wrap gap-1" aria-label="Tiến độ từng bài">
          {chapters.map((ch) => {
            const total = ch.endPage - ch.startPage + 1;
            let done = 0;
            for (let p = ch.startPage; p <= ch.endPage; p++) if (studiedSet.has(p)) done++;
            const level = lessonLevel(done, total);
            return (
              <Link
                key={ch.lesson}
                href={`/books/${book.id}#bai-${ch.lesson}`}
                title={`Bài ${ch.lesson}: đã học ${done}/${total} trang`}
                aria-label={`Bài ${ch.lesson}: đã học ${done}/${total} trang`}
                className={cn(
                  "flex size-6 items-center justify-center rounded-md text-[10px] tabular-nums transition-opacity hover:opacity-80",
                  HEAT_CLASSES[level],
                  level >= 3 ? "text-primary-foreground" : "text-foreground/60"
                )}
              >
                {ch.lesson}
              </Link>
            );
          })}
        </div>
      )}

      <Link
        href={`/read/${book.id}/${resume}`}
        className="self-start text-sm font-medium text-primary underline-offset-4 hover:underline"
      >
        Học tiếp trang {resume} →
      </Link>
    </div>
  );
}
