"use client";

import { useState } from "react";
import Link from "next/link";
import { Check, Download, ListChecks, MoreVertical, RotateCcw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { Book } from "@/lib/books";
import { cn } from "@/lib/utils";
import {
  OfflineDownloadError,
  OfflineDownloadProgress,
  useOfflineDownload,
} from "@/components/library/offline-download";
import { useAnswersShown } from "@/components/library/answers-toggle";

const ITEM = "w-full justify-start";

/**
 * Menu ⋮ gom các việc ít dùng của trang chi tiết sách — CHỈ trên điện thoại.
 *
 * Hàng nút trên màn hẹp tràn sang hai, ba dòng, đẩy lưới trang xuống; mà
 * người ta vào đây chủ yếu để bấm "Đọc tiếp". Nên chỉ giữ ngoài những thứ
 * bấm thường xuyên (Đọc tiếp, nghe tự động), còn lại vào menu — giống menu ⋮
 * trong trình đọc.
 *
 * Hai thứ cố ý KHÔNG nằm kín trong menu:
 * - Tiến độ đang tải offline: hiện ngay cạnh nút ⋮, đang tải mà phải mở menu
 *   mới biết tới đâu thì như không có.
 * - Hộp xác nhận (xoá bản offline, hết chỗ): vẽ ngoài menu. Menu đóng lại
 *   ngay khi chọn mục, hộp mà nằm trong đó thì biến mất theo.
 */
export function BookActionsMenu({
  book,
  startedReading,
  hasAnswers,
  className,
}: {
  book: Book;
  /** Đã đọc dở — lúc đó mới có mục "Đọc lại từ trang 1". */
  startedReading: boolean;
  hasAnswers: boolean;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const download = useOfflineDownload(book);
  const answers = useAnswersShown(book.id);

  /** Chọn mục xong thì đóng menu rồi mới làm, như menu ⋮ của trình đọc. */
  function select(action: () => void) {
    return () => {
      setOpen(false);
      action();
    };
  }

  return (
    <div className={cn("flex flex-col items-end gap-1", className)}>
      <div className="flex items-center gap-1">
        {download.ready && download.state === "downloading" && (
          <OfflineDownloadProgress download={download} />
        )}
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button variant="ghost" size="icon" className="size-9" aria-label="Thêm tuỳ chọn cho sách này">
              <MoreVertical className="size-5" aria-hidden />
            </Button>
          </PopoverTrigger>
          <PopoverContent align="end" className="w-60 p-1">
            <div className="flex flex-col">
              {startedReading && (
                <Button asChild variant="ghost" size="sm" className={ITEM}>
                  <Link href={`/read/${book.id}/1`} onClick={() => setOpen(false)}>
                    <RotateCcw className="size-4" aria-hidden />
                    Đọc lại từ trang 1
                  </Link>
                </Button>
              )}

              {download.ready && download.state === "idle" && (
                <Button
                  variant="ghost"
                  size="sm"
                  className={ITEM}
                  disabled={download.busyWithOther}
                  onClick={select(download.requestDownload)}
                >
                  <Download className="size-4" aria-hidden />
                  Tải offline · {download.sizeLabel}
                </Button>
              )}
              {download.ready && download.state === "downloading" && (
                <Button variant="ghost" size="sm" className={ITEM} onClick={select(download.cancel)}>
                  <X className="size-4" aria-hidden />
                  Huỷ tải về ({download.percent}%)
                </Button>
              )}
              {download.ready && download.state === "downloaded" && (
                <Button
                  variant="ghost"
                  size="sm"
                  className={ITEM}
                  disabled={download.busyWithOther}
                  onClick={select(download.requestRemove)}
                >
                  <Check className="size-4 text-primary" aria-hidden />
                  Đã tải offline · bấm để xoá
                </Button>
              )}

              {hasAnswers && (
                // Không đóng menu khi bật/tắt: chữ đổi ngay tại chỗ là lời xác
                // nhận, đóng luôn thì người dùng không thấy mình vừa đổi gì.
                <Button
                  variant="ghost"
                  size="sm"
                  className={ITEM}
                  aria-pressed={answers.shown}
                  onClick={answers.toggle}
                >
                  <ListChecks className="size-4" aria-hidden />
                  Đáp án: {answers.shown ? "bật" : "tắt"}
                </Button>
              )}
            </div>
          </PopoverContent>
        </Popover>
      </div>
      <OfflineDownloadError error={download.error} />
      {download.dialogs}
    </div>
  );
}
