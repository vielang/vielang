"use client";

import Image from "next/image";
import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { getThumbUrl, type Book } from "@/lib/books";
import { getChapters } from "@/lib/chapters";
import { shortBookName } from "@/lib/companion";
import type { BookProgress } from "@/lib/progress-store";

/** Góc học tập chỉ bày bấy nhiêu trang; còn lại xem ở trang danh sách đầy đủ. */
const MAX_SHOWN = 10;

interface Pin {
  book: Book;
  page: number;
}

/**
 * Các trang đã ghim, sách đọc gần nhất lên trước, trong một sách theo số
 * trang. Không có "ghim lúc nào" — đánh dấu chỉ lưu số trang — nên lấy thời
 * điểm đọc sách đó gần nhất làm thứ tự: sách đang học dở thì trang ghim của
 * nó hay cần mở lại nhất.
 */
export function pinnedPages(
  books: readonly Book[],
  progressByBook: Record<string, BookProgress>
): Pin[] {
  return books
    .map((book) => ({ book, progress: progressByBook[book.id] }))
    .filter(({ progress }) => (progress?.bookmarks.length ?? 0) > 0)
    .sort((a, b) => (b.progress!.updatedAt ?? "").localeCompare(a.progress!.updatedAt ?? ""))
    .flatMap(({ book, progress }) =>
      [...progress!.bookmarks].sort((x, y) => x - y).map((page) => ({ book, page }))
    );
}

/** "Bài 1 · 18" — cùng cách ghi với trang danh sách đầy đủ; trang ngoài bài học thì chỉ số trang. */
function pageLabel(book: Book, page: number): string {
  const chapter = getChapters(book.id, book.totalPages).find(
    (ch) => page >= ch.startPage && page <= ch.endPage
  );
  return chapter ? `Bài ${chapter.lesson} · ${page}` : String(page);
}

/**
 * Hàng ảnh thu nhỏ các trang đã đánh dấu, trong Góc học tập — thay cho tab
 * "Đánh dấu" riêng trước đây. Cuộn ngang, bấm là mở đúng trang; danh sách đầy
 * đủ (có nút bỏ ghim) ở `/my/danh-dau`.
 *
 * Chưa ghim trang nào thì không vẽ gì: một khung trống ở đây chỉ chiếm chỗ,
 * cách ghim đã có trong hướng dẫn của trang đọc.
 */
export function BookmarkStrip({
  books,
  progressByBook,
}: {
  books: readonly Book[];
  progressByBook: Record<string, BookProgress>;
}) {
  const pins = pinnedPages(books, progressByBook);
  if (pins.length === 0) return null;

  return (
    <section className="flex flex-col gap-3" aria-labelledby="pins-title">
      <div className="flex items-baseline justify-between gap-2">
        <h2 id="pins-title" className="text-base font-semibold">
          Trang đã đánh dấu
          <span className="ml-1.5 text-sm font-normal text-muted-foreground tabular-nums">
            {pins.length}
          </span>
        </h2>
        <Link
          href="/my/danh-dau"
          className="inline-flex shrink-0 items-center gap-1 text-sm text-primary hover:underline"
        >
          Xem tất cả
          <ArrowRight className="size-3.5" aria-hidden />
        </Link>
      </div>
      <ul className="-mx-4 flex gap-3 overflow-x-auto px-4 pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {pins.slice(0, MAX_SHOWN).map(({ book, page }) => (
          <li key={`${book.id}:${page}`} className="w-24 shrink-0">
            <Link
              href={`/read/${book.id}/${page}`}
              className="group flex flex-col gap-1 focus-visible:outline-none"
            >
              <span className="relative block aspect-[192/250] w-full overflow-hidden rounded-lg border border-border bg-muted transition-shadow group-hover:shadow-md group-focus-visible:ring-2 group-focus-visible:ring-ring">
                <Image
                  src={getThumbUrl(book.id, page)}
                  alt={`${book.titleVi}, trang ${page}`}
                  fill
                  unoptimized
                  sizes="96px"
                  className="object-cover"
                />
                {/* Số trang nằm trên ảnh như ở trang danh sách đầy đủ — để dưới ảnh thì
                    cột ảnh hẹp cắt mất, chỉ còn "Giáo trình Sơ …". */}
                <span className="absolute bottom-1 left-1 rounded bg-background/85 px-1 text-[10px] text-foreground tabular-nums">
                  {pageLabel(book, page)}
                </span>
              </span>
              <span className="line-clamp-2 text-[11px] leading-tight text-muted-foreground">
                {shortBookName(book)}
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </section>
  );
}
