"use client";

import Link from "next/link";
import { Headphones, RotateCcw } from "lucide-react";
import { startAutoplay } from "@/lib/autoplay-player";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { Book } from "@/lib/books";
import {
  getBookProgress,
  percentRead,
  resumePage,
  useProgressStore,
} from "@/lib/progress-store";
import { OfflineDownload } from "@/components/library/offline-download";
import { AnswersToggle } from "@/components/library/answers-toggle";

/**
 * Đầu trang chi tiết sách.
 *
 * Trước đây mỗi dữ kiện một khối riêng: huy hiệu cấp độ, tên, tên tiếng Hàn,
 * rồi ba dòng số (số trang, phần trăm, dung lượng tải). Sáu khối chồng nhau
 * đẩy lưới trang — thứ người ta thật sự vào đây để xem — xuống tít dưới.
 *
 * Giờ mọi con số gom vào MỘT dòng phụ, và chỉ những gì thay đổi theo người
 * dùng mới được chiếm chỗ riêng: thanh tiến độ chỉ hiện khi đã đọc dở, vì
 * thanh 0% không nói thêm điều gì mà vẫn ăn một dòng.
 */
export function BookDetailHeader({
  book,
  hasAnswers = false,
  hasAudio = false,
}: {
  book: Book;
  /** Sách có chấm đáp án không (xem lib/page-answers.ts) — không có thì khỏi hiện công tắc. */
  hasAnswers?: boolean;
  /** Sách có bài nghe không — không có thì khỏi hiện nút nghe tự động. */
  hasAudio?: boolean;
}) {
  const books = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);
  const progress = getBookProgress(books, book.id);
  const percent = percentRead(progress, book.totalPages);
  const startedReading = hasHydrated && progress.readPages.length > 0;
  const target = startedReading ? resumePage(progress) : 1;

  return (
    <div className="flex flex-col gap-3 border-b border-border pb-5">
      <div>
        <h1 className="text-xl font-semibold tracking-tight sm:text-2xl">
          {book.titleVi}
        </h1>
        {book.titleKo && (
          <p className="font-korean mt-0.5 text-sm text-muted-foreground">
            {book.titleKo}
          </p>
        )}
      </div>

      {/* Một dòng cho mọi dữ kiện tĩnh — trước đây là ba dòng rời. */}
      <p className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
        <span>Cấp {book.level}</span>
        <Separator />
        <span className="tabular-nums">{book.totalPages} trang</span>
        {startedReading && (
          <>
            <Separator />
            <span className="tabular-nums">{percent}% đã đọc</span>
          </>
        )}
      </p>

      {startedReading && <Progress value={percent} className="h-1 max-w-xs" />}

      <div className="flex flex-wrap items-center gap-2">
        <Button asChild size="sm">
          <Link href={`/read/${book.id}/${target}`}>
            {startedReading ? `Đọc tiếp — trang ${target}` : "Bắt đầu đọc"}
          </Link>
        </Button>

        {startedReading && (
          <Tooltip>
            <TooltipTrigger asChild>
              {/* Chỉ icon: việc phụ, mà để cả chữ thì hàng nút tràn sang
                  dòng thứ hai trên điện thoại. */}
              <Button asChild variant="ghost" size="icon" aria-label="Đọc lại từ trang 1">
                <Link href={`/read/${book.id}/1`}>
                  <RotateCcw className="size-4" aria-hidden />
                </Link>
              </Button>
            </TooltipTrigger>
            <TooltipContent>Đọc lại từ trang 1</TooltipContent>
          </Tooltip>
        )}

        {hasAudio && (
          // Phát ngay trong click rồi mới lật vào trình đọc — ra khỏi lượt
          // xử lý lần chạm là trình duyệt không cho phát nữa (xem
          // lib/autoplay-player.ts). Lật trang do AutoplayFollower lo.
          <Button size="sm" variant="outline" onClick={() => startAutoplay(book.id)}>
            <Headphones className="size-4" aria-hidden />
            Nghe tự động
          </Button>
        )}

        <OfflineDownload book={book} />

        {hasAnswers && <AnswersToggle bookId={book.id} />}
      </div>
    </div>
  );
}

function Separator() {
  return (
    <span className="text-border" aria-hidden>
      ·
    </span>
  );
}
