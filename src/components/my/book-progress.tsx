"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import type { Book } from "@/lib/books";
import { getChapters } from "@/lib/chapters";
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
 * Tiến độ một cuốn: số trang ĐÃ HỌC (ở lại đủ lâu) là con số chính, số trang
 * đã xem để phụ. Dưới là một hàng ô, mỗi ô một bài, đậm theo phần trang đã
 * học của bài đó — nhìn là biết bài nào còn bỏ dở.
 */
export function BookProgressCard({ row }: { row: BookRow }) {
  const { book, progress, studied } = row;
  const studiedSet = new Set(studied);
  const percent = Math.round((studied.length / book.totalPages) * 100);
  const chapters = getChapters(book.id, book.totalPages);

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-border p-4">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="font-medium leading-snug">{book.titleVi}</h3>
          <p className="mt-0.5 text-xs text-muted-foreground tabular-nums">
            Đã học {studied.length}/{book.totalPages} trang · đã xem{" "}
            {progress.readPages.length} trang
          </p>
        </div>
        <Button asChild size="sm" variant="outline">
          <Link href={`/read/${book.id}/${resumePage(progress)}`}>
            Đọc tiếp — trang {resumePage(progress)}
          </Link>
        </Button>
      </div>

      <div className="flex items-center gap-3">
        <Progress value={percent} className="h-1.5 flex-1" aria-label={`Đã học ${percent}%`} />
        <span className="w-10 text-right text-sm font-medium tabular-nums">{percent}%</span>
      </div>

      {chapters.length > 0 && (
        <div className="flex flex-wrap gap-1" aria-label="Tiến độ từng bài">
          {chapters.map((ch) => {
            const total = ch.endPage - ch.startPage + 1;
            let done = 0;
            for (let p = ch.startPage; p <= ch.endPage; p++) if (studiedSet.has(p)) done++;
            return (
              <Link
                key={ch.lesson}
                href={`/books/${book.id}#bai-${ch.lesson}`}
                title={`Bài ${ch.lesson}: đã học ${done}/${total} trang`}
                aria-label={`Bài ${ch.lesson}: đã học ${done}/${total} trang`}
                className={cn(
                  "flex h-6 min-w-6 items-center justify-center rounded px-1 text-[10px] tabular-nums transition-opacity hover:opacity-80",
                  HEAT_CLASSES[lessonLevel(done, total)],
                  lessonLevel(done, total) >= 3 ? "text-primary-foreground" : "text-foreground/70"
                )}
              >
                {ch.lesson}
              </Link>
            );
          })}
        </div>
      )}
    </div>
  );
}
