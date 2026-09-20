"use client";

import Image from "next/image";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { getThumbUrl, type Book } from "@/lib/books";
import {
  getBookProgress,
  percentRead,
  resumePage,
  useProgressStore,
} from "@/lib/progress-store";

/** Nhiều hơn chừng này thì hết là "lối tắt" mà thành một thư viện thứ hai. */
const MAX_ITEMS = 3;

/**
 * Dải "Đọc tiếp" trên đầu thư viện.
 *
 * Quay lại chỗ đang đọc dở là việc người dùng làm nhiều nhất, mà trước đây
 * tốn ba bước: mở thư viện, tìm đúng cuốn, bấm "Đọc tiếp". Dải này rút
 * xuống một lần chạm, và đi thẳng tới đúng số trang.
 *
 * Xếp theo lần đọc gần nhất (`updatedAt`) chứ không theo tên hay cấp độ:
 * cuốn vừa gấp lại luôn là cuốn muốn mở ra tiếp.
 *
 * Chỉ nhận sách của đúng ngôn ngữ đang xem — đọc dở sách tiếng Hàn thì
 * không việc gì phải hiện ở trang tiếng Anh.
 */
export function ContinueReading({ books }: { books: readonly Book[] }) {
  const progressByBook = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);

  // Tiến độ nằm ở localStorage nên server không biết gì — chờ nạp xong mới
  // vẽ, nếu không HTML hai bên lệch nhau.
  if (!hasHydrated) return null;

  const items = books
    .map((book) => ({ book, progress: getBookProgress(progressByBook, book.id) }))
    .filter(({ progress }) => progress.readPages.length > 0)
    .sort((a, b) => b.progress.updatedAt.localeCompare(a.progress.updatedAt))
    .slice(0, MAX_ITEMS);

  if (items.length === 0) return null;

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-base font-semibold tracking-tight">Đọc tiếp</h2>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {items.map(({ book, progress }) => {
          const page = resumePage(progress);
          const percent = percentRead(progress, book.totalPages);
          return (
            <Link
              key={book.id}
              href={`/read/${book.id}/${page}`}
              className="group focus-visible:ring-ring flex items-center gap-3 rounded-xl border border-border bg-card p-2.5 transition-shadow hover:shadow-md focus-visible:ring-2 focus-visible:outline-none"
            >
              <div className="relative aspect-[192/250] w-12 shrink-0 overflow-hidden rounded-md bg-muted">
                <Image
                  src={getThumbUrl(book.id, page)}
                  alt=""
                  fill
                  unoptimized
                  sizes="48px"
                  className="object-cover"
                />
              </div>
              <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                <p className="truncate text-sm leading-tight font-medium">
                  {book.titleVi}
                </p>
                <p className="text-xs tabular-nums text-muted-foreground">
                  Trang {page}/{book.totalPages} · {percent}%
                </p>
                <Progress value={percent} className="h-1" />
              </div>
              <ChevronRight
                className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                aria-hidden
              />
            </Link>
          );
        })}
      </div>
    </section>
  );
}
