"use client";

import Image from "next/image";
import Link from "next/link";
import { Bookmark, BookmarkX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { getThumbUrl, type Book } from "@/lib/books";
import { getChapters } from "@/lib/chapters";
import { useProgressStore } from "@/lib/progress-store";
import { EmptyState } from "@/components/layout/empty-state";

/**
 * Danh sách trang đã đánh dấu, gom theo sách.
 *
 * Trước đây đánh dấu là tính năng chỉ ghi không đọc: có tới ba chỗ để bấm
 * ghim (phím `b`, nút trong trang đọc, nút trong lưới trang) nhưng không màn
 * hình nào liệt kê ra. Ghim xong rồi tự đi tìm lại.
 *
 * Kèm luôn nút bỏ ghim ngay tại đây — chỗ tự nhiên nhất để dọn là chỗ nhìn
 * thấy cả danh sách.
 */
export function BookmarkList({ books }: { books: readonly Book[] }) {
  const progressByBook = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);
  const toggleBookmark = useProgressStore((s) => s.toggleBookmark);

  // Đánh dấu nằm ở localStorage nên server không biết gì — giữ chỗ bằng
  // khung xám thay vì nhảy từ "trống" sang "có" sau khi nạp.
  if (!hasHydrated) {
    return (
      <div className="flex flex-col gap-3">
        <Skeleton className="h-5 w-40" />
        <Skeleton className="h-28 w-full rounded-xl" />
      </div>
    );
  }

  const groups = books
    .map((book) => ({
      book,
      pages: [...(progressByBook[book.id]?.bookmarks ?? [])].sort((a, b) => a - b),
    }))
    .filter(({ pages }) => pages.length > 0);

  if (groups.length === 0) {
    return (
      <EmptyState
        dashed
        icon={Bookmark}
        title="Chưa đánh dấu trang nào"
        description={
          <>
            Khi đang đọc, bấm biểu tượng dấu trang trên thanh công cụ (hoặc phím{" "}
            <kbd className="rounded border border-border bg-muted px-1 font-mono text-xs">B</kbd>)
            để ghim lại trang cần xem kỹ. Chúng sẽ nằm ở đây.
          </>
        }
        action={
          <Button asChild variant="outline">
            <Link href="/">Về thư viện</Link>
          </Button>
        }
      />
    );
  }

  return (
    <div className="flex flex-col gap-8">
      {groups.map(({ book, pages }) => {
        const chapters = getChapters(book.id, book.totalPages);
        return (
          <section key={book.id} className="flex flex-col gap-3">
            <div className="flex items-baseline gap-2 border-b border-border pb-2">
              <h2 className="text-base font-semibold tracking-tight">
                <Link href={`/books/${book.id}`} className="hover:underline">
                  {book.titleVi}
                </Link>
              </h2>
              <span className="text-xs tabular-nums text-muted-foreground">
                {pages.length} trang
              </span>
            </div>

            <ul className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
              {pages.map((page) => {
                const chapter = chapters.find(
                  (ch) => page >= ch.startPage && page <= ch.endPage
                );
                return (
                  <li key={page} className="group relative">
                    <Link
                      href={`/read/${book.id}/${page}`}
                      className="focus-visible:ring-ring block overflow-hidden rounded-lg border border-border bg-muted transition-shadow hover:shadow-md focus-visible:ring-2 focus-visible:outline-none"
                    >
                      <div className="relative aspect-[192/250] w-full">
                        <Image
                          src={getThumbUrl(book.id, page)}
                          alt={`Trang ${page}`}
                          fill
                          unoptimized
                          sizes="(min-width: 1024px) 15vw, (min-width: 640px) 22vw, 30vw"
                          className="object-cover"
                        />
                        <span className="absolute bottom-1 left-1.5 rounded bg-background/85 px-1 text-[10px] tabular-nums text-foreground">
                          {chapter ? `Bài ${chapter.lesson} · ` : ""}
                          {page}
                        </span>
                      </div>
                    </Link>

                    {/* Luôn hiện, không giấu sau hover: cảm ứng không có rê
                        chuột — cùng lý do đã sửa ở lưới trang. */}
                    <button
                      type="button"
                      onClick={() => toggleBookmark(book.id, page)}
                      aria-label={`Bỏ đánh dấu trang ${page}`}
                      title="Bỏ đánh dấu"
                      className="absolute top-1 right-1 rounded-full bg-background/90 p-1.5 text-primary shadow transition-colors hover:text-destructive"
                    >
                      <BookmarkX className="size-4" aria-hidden />
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
