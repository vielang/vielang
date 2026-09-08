"use client";

import Image from "next/image";
import Link from "next/link";
import { Bookmark, Check, NotebookText } from "lucide-react";
import { cn } from "@/lib/utils";
import { getThumbUrl } from "@/lib/books";
import { useProgressStore } from "@/lib/progress-store";

export function PageGrid({
  bookId,
  totalPages,
  notePages,
}: {
  bookId: string;
  totalPages: number;
  /** Số trang có sẵn bài giảng (xem lib/notes.ts) */
  notePages: number[];
}) {
  const books = useProgressStore((s) => s.books);
  const toggleBookmark = useProgressStore((s) => s.toggleBookmark);
  const readPages = new Set(books[bookId]?.readPages ?? []);
  const bookmarks = new Set(books[bookId]?.bookmarks ?? []);
  const notedPages = new Set(notePages);

  const pages = Array.from({ length: totalPages }, (_, i) => i + 1);

  return (
    <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
      {pages.map((page) => {
        const isRead = readPages.has(page);
        const isBookmarked = bookmarks.has(page);
        const hasNote = notedPages.has(page);
        return (
          <div key={page} className="group relative">
            <Link
              href={`/read/${bookId}/${page}`}
              className="focus-visible:ring-ring block overflow-hidden rounded-lg border border-border bg-muted transition-shadow hover:shadow-md focus-visible:ring-2 focus-visible:outline-none"
            >
              <div className="relative aspect-[192/250] w-full">
                <Image
                  src={getThumbUrl(bookId, page)}
                  alt={`Trang ${page}`}
                  fill
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
                {hasNote && (
                  <span
                    className="absolute top-1 left-1 rounded-full bg-background/90 p-0.5 text-primary shadow"
                    title="Có bài giảng"
                  >
                    <NotebookText className="size-3" aria-hidden />
                  </span>
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
              className={cn(
                "absolute top-1 right-1 rounded-full bg-background/90 p-1 shadow transition-colors",
                isBookmarked
                  ? "text-primary"
                  : "text-muted-foreground opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
              )}
            >
              <Bookmark
                className="size-3.5"
                fill={isBookmarked ? "currentColor" : "none"}
                aria-hidden
              />
            </button>
          </div>
        );
      })}
    </div>
  );
}
