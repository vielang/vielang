"use client";

import Image from "next/image";
import Link from "next/link";
import { BookOpen } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import type { Book } from "@/lib/books";
import { getThumbUrl } from "@/lib/books";
import {
  getBookProgress,
  percentRead,
  resumePage,
  useProgressStore,
} from "@/lib/progress-store";

export function BookCard({ book }: { book: Book }) {
  const books = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);
  const progress = getBookProgress(books, book.id);
  const percent = percentRead(progress, book.totalPages);
  const startedReading = progress.readPages.length > 0;
  const target = startedReading ? resumePage(progress) : 1;

  return (
    <div className="group flex flex-col overflow-hidden rounded-xl border border-border bg-card transition-shadow hover:shadow-md">
      <Link
        href={`/books/${book.id}`}
        className="focus-visible:ring-ring relative aspect-[192/250] block w-full overflow-hidden bg-muted focus-visible:outline-none focus-visible:ring-2"
      >
        <Image
          src={getThumbUrl(book.id, 1)}
          alt={`Bìa sách ${book.titleVi}`}
          fill
          sizes="(min-width: 768px) 220px, 45vw"
          className="object-cover transition-transform duration-300 group-hover:scale-[1.03]"
        />
        <Badge className="absolute top-2 left-2 shadow-sm" variant="secondary">
          Cấp {book.level}
        </Badge>
      </Link>

      <div className="flex flex-1 flex-col gap-2 p-3">
        <Link href={`/books/${book.id}`} className="hover:underline">
          <h3 className="text-sm leading-tight font-medium">{book.titleVi}</h3>
        </Link>
        <p className="font-korean -mt-1 text-xs text-muted-foreground">
          {book.titleKo}
        </p>

        <div className="mt-auto flex flex-col gap-2 pt-2">
          {hasHydrated ? (
            <>
              <div className="flex items-center justify-between text-xs">
                <span className="flex items-center gap-1 text-muted-foreground">
                  <BookOpen className="size-3.5" aria-hidden />
                  {startedReading
                    ? `Trang ${target}/${book.totalPages}`
                    : `${book.totalPages} trang`}
                </span>
                <span className="tabular-nums text-muted-foreground">
                  {percent}%
                </span>
              </div>
              <Progress value={percent} className="h-1.5" />
            </>
          ) : (
            <Skeleton className="h-1.5 w-full rounded-full" />
          )}

          <Button asChild size="sm" className="mt-1 w-full">
            <Link href={`/read/${book.id}/${target}`}>
              {startedReading ? "Đọc tiếp" : "Bắt đầu đọc"}
            </Link>
          </Button>
        </div>
      </div>
    </div>
  );
}
