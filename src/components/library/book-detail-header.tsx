"use client";

import Link from "next/link";
import { Headphones, RotateCcw } from "lucide-react";
import { buildPlaylist } from "@/lib/autoplay";
import {
  continueAutoplay,
  findResumePoint,
  useAutoplayResumeStore,
} from "@/lib/autoplay-player";
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
import { BookActionsMenu } from "@/components/library/book-actions-menu";

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

  // Chỗ nghe dở (nếu có) — chờ nạp xong localStorage mới hiện, tránh lệch
  // với HTML server render.
  const listenPosition = useAutoplayResumeStore((s) => s.positions[book.id]);
  const listenHydrated = useAutoplayResumeStore((s) => s.hasHydrated);
  let resumeListenPage: number | null = null;
  if (hasAudio && listenHydrated && listenPosition) {
    const queue = buildPlaylist(book.id);
    const point = findResumePoint(queue, listenPosition);
    if (point) resumeListenPage = queue[point.index].page;
  }
  const listenLabel = resumeListenPage ? `Nghe tiếp — trang ${resumeListenPage}` : "Nghe tự động";

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

      {/* Điện thoại: chỉ giữ ngoài [Đọc tiếp] [🎧] [⋮], phần còn lại vào menu
          (xem BookActionsMenu). Màn rộng đủ chỗ nên bày hết ra như cũ. */}
      <div className="flex items-start gap-2 sm:flex-wrap sm:items-center">
        <Button asChild size="sm" className="h-9 min-w-0 flex-1 sm:h-7 sm:flex-none">
          <Link href={`/read/${book.id}/${target}`}>
            <span className="truncate">
              {startedReading ? `Đọc tiếp — trang ${target}` : "Bắt đầu đọc"}
            </span>
          </Link>
        </Button>

        {startedReading && (
          <Tooltip>
            <TooltipTrigger asChild>
              {/* Chỉ icon: việc phụ, mà để cả chữ thì hàng nút tràn sang
                  dòng thứ hai. Điện thoại thì nằm trong menu ⋮. */}
              <Button
                asChild
                variant="ghost"
                size="icon"
                className="hidden sm:inline-flex"
                aria-label="Đọc lại từ trang 1"
              >
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
          //
          // Điện thoại: chỉ icon nhưng vẫn để NGOÀI menu — tính năng này mà
          // giấu vào ⋮ thì gần như không ai tìm ra.
          <Button
            size="sm"
            variant="outline"
            className="size-9 shrink-0 p-0 sm:h-7 sm:w-auto sm:px-2.5"
            onClick={() => continueAutoplay(book.id)}
            aria-label={listenLabel}
            title={listenLabel}
          >
            <Headphones className="size-4" aria-hidden />
            <span className="hidden sm:inline">{listenLabel}</span>
          </Button>
        )}

        <span className="hidden sm:contents">
          <OfflineDownload book={book} />
          {hasAnswers && <AnswersToggle bookId={book.id} />}
        </span>

        <BookActionsMenu
          book={book}
          startedReading={startedReading}
          hasAnswers={hasAnswers}
          className="shrink-0 sm:hidden"
        />
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
