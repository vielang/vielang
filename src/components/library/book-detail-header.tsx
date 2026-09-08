"use client";

import Link from "next/link";
import { BookOpen, RotateCcw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import type { Book } from "@/lib/books";
import {
  getBookProgress,
  percentRead,
  resumePage,
  useProgressStore,
} from "@/lib/progress-store";

export function BookDetailHeader({ book }: { book: Book }) {
  const books = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);
  const progress = getBookProgress(books, book.id);
  const percent = percentRead(progress, book.totalPages);
  const startedReading = progress.readPages.length > 0;
  const target = startedReading ? resumePage(progress) : 1;

  return (
    <div className="flex flex-col gap-4 border-b border-border pb-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <Badge variant="secondary" className="mb-2">
            Cấp {book.level}
          </Badge>
          <h1 className="text-xl font-semibold tracking-tight sm:text-2xl">
            {book.titleVi}
          </h1>
          <p className="font-korean mt-1 text-sm text-muted-foreground">
            {book.titleKo}
          </p>
        </div>
      </div>

      {hasHydrated && (
        <div className="flex flex-col gap-1.5 sm:max-w-xs">
          <div className="flex items-center justify-between text-xs">
            <span className="flex items-center gap-1 text-muted-foreground">
              <BookOpen className="size-3.5" aria-hidden />
              {book.totalPages} trang
            </span>
            <span className="tabular-nums text-muted-foreground">
              {percent}% đã đọc
            </span>
          </div>
          <Progress value={percent} className="h-1.5" />
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button asChild>
          <Link href={`/read/${book.id}/${target}`}>
            {startedReading ? `Đọc tiếp — trang ${target}` : "Bắt đầu đọc"}
          </Link>
        </Button>
        {startedReading && (
          <Button asChild variant="outline">
            <Link href={`/read/${book.id}/1`}>
              <RotateCcw className="size-4" aria-hidden />
              Đọc từ đầu
            </Link>
          </Button>
        )}
      </div>
    </div>
  );
}
