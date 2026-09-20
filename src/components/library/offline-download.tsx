"use client";

import { useState } from "react";
import {
  AlertTriangle,
  Check,
  Download,
  Loader2,
  Trash2,
  WifiOff,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { getBook, type Book } from "@/lib/books";
import { estimateBytes, formatBytes } from "@/lib/offline-books";
import { hasFreeSlot, useDownloadStore, useOfflineBooks } from "@/lib/download-store";

/**
 * Nút tải sách về đọc offline, đặt ở trang chi tiết sách.
 *
 * Bốn trạng thái, mỗi trạng thái một hành động rõ ràng: chưa tải (tải về),
 * đang tải (huỷ), đã tải (xoá), và hết chỗ (xoá cuốn kia trước).
 *
 * "Hết chỗ" không phải lỗi nên không báo đỏ — chỉ nói thẳng cuốn nào đang
 * chiếm chỗ và cho xoá ngay tại đó, thay vì bắt người dùng tự đi tìm.
 */
export function OfflineDownload({ book }: { book: Book }) {
  const { books: offlineBooks, hasLoaded } = useOfflineBooks();
  const active = useDownloadStore((s) => s.active);
  const error = useDownloadStore((s) => s.error);
  const start = useDownloadStore((s) => s.start);
  const cancel = useDownloadStore((s) => s.cancel);
  const remove = useDownloadStore((s) => s.remove);
  const clearError = useDownloadStore((s) => s.clearError);

  const [limitOpen, setLimitOpen] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState(false);

  // Cache API chỉ có ở trình duyệt — chờ đọc xong rồi mới vẽ, nếu không nút
  // nhảy từ "tải về" sang "đã tải" ngay trước mắt.
  if (!hasLoaded) return null;

  const isDownloaded = offlineBooks.includes(book.id);
  const isDownloading = active?.bookId === book.id;
  const busyWithOther = active !== null && !isDownloading;
  const occupant = offlineBooks.find((id) => id !== book.id);
  const occupantBook = occupant ? getBook(occupant) : undefined;

  if (isDownloading) {
    const { done, total } = active.progress;
    return (
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <Loader2 className="size-3.5 shrink-0 animate-spin" aria-hidden />
        <span className="tabular-nums">
          {done}/{total}
        </span>
        <Progress value={(done / total) * 100} className="h-1 w-20" />
        <Button
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={cancel}
          aria-label="Huỷ tải về"
        >
          <X className="size-3.5" aria-hidden />
        </Button>
      </div>
    );
  }

  if (isDownloaded) {
    return (
      <>
        <Tooltip>
          <TooltipTrigger asChild>
            {/* Nhãn "đã tải" và nút xoá gộp làm một: cùng nói về một thứ, mà
                tách ra thì ăn mất hai chỗ trên hàng nút. */}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setConfirmRemove(true)}
              disabled={busyWithOther}
            >
              <Check className="size-4 text-primary" aria-hidden />
              Đã tải offline
            </Button>
          </TooltipTrigger>
          <TooltipContent>Đọc được khi mất mạng — bấm để xoá</TooltipContent>
        </Tooltip>

        <Dialog open={confirmRemove} onOpenChange={setConfirmRemove}>
          <DialogContent className="sm:max-w-sm">
            <DialogHeader>
              <DialogTitle>Xoá bản offline của {book.titleVi}?</DialogTitle>
              <DialogDescription>
                Giải phóng khoảng {formatBytes(estimateBytes(book))} và cho phép tải
                cuốn khác về. Tiến độ đọc, ghi chú, nét vẽ và bản ghi âm đều KHÔNG
                bị đụng tới — chỉ xoá ảnh đã tải.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setConfirmRemove(false)}>
                Huỷ
              </Button>
              <Button
                variant="destructive"
                onClick={() => {
                  void remove(book.id);
                  setConfirmRemove(false);
                }}
              >
                Xoá bản offline
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </>
    );
  }

  const full = !hasFreeSlot(offlineBooks);

  return (
    <>
      <div className="flex flex-col gap-2">
        {/* Dung lượng nằm ngay trong nhãn nút thay vì một dòng riêng — người
            dùng mạng di động cần biết trước khi bấm, nhưng không cần cả một
            dòng cho nó. */}
        <Button
          variant="outline"
          size="sm"
          disabled={busyWithOther}
          onClick={() => {
            clearError();
            if (full) setLimitOpen(true);
            else void start(book);
          }}
        >
          <Download className="size-4" aria-hidden />
          Tải offline · {formatBytes(estimateBytes(book))}
        </Button>

        {error && (
          <p className="flex items-start gap-1.5 text-xs text-destructive">
            <AlertTriangle className="mt-px size-3.5 shrink-0" aria-hidden />
            {error}
          </p>
        )}
      </div>

      <Dialog open={limitOpen} onOpenChange={setLimitOpen}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <WifiOff className="size-4 text-muted-foreground" aria-hidden />
              Đã đủ số sách tải về
            </DialogTitle>
            <DialogDescription>
              Mỗi lần chỉ giữ được một cuốn trên máy, vì mỗi cuốn nặng cỡ{" "}
              {formatBytes(estimateBytes(book))}. Hiện{" "}
              <span className="font-medium text-foreground">
                {occupantBook?.titleVi ?? "một cuốn khác"}
              </span>{" "}
              đang chiếm chỗ — xoá nó đi là tải được {book.titleVi}.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setLimitOpen(false)}>
              Để sau
            </Button>
            {occupant && (
              <Button
                variant="destructive"
                onClick={async () => {
                  await remove(occupant);
                  setLimitOpen(false);
                  void start(book);
                }}
              >
                <Trash2 className="size-4" aria-hidden />
                Xoá rồi tải cuốn này
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
