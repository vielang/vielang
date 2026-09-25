"use client";

import { useState, type ReactNode } from "react";
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
import {
  AUDIO_CAN_BE_CACHED,
  estimateBytes,
  formatBytes,
} from "@/lib/offline-books";
import { hasFreeSlot, useDownloadStore, useOfflineBooks } from "@/lib/download-store";

/**
 * Nói đúng thứ gói tải về thật sự có. Hứa cả bài nghe trong khi nó không
 * được lưu là lời hứa suông, và người dùng chỉ phát hiện lúc mất mạng.
 */
const PART_LABEL = AUDIO_CAN_BE_CACHED
  ? "phần (trang sách và bài nghe)"
  : "trang sách";

export interface OfflineDownloadState {
  /** Đã đọc xong danh sách sách đã tải — chưa xong thì đừng vẽ gì. */
  ready: boolean;
  state: "idle" | "downloading" | "downloaded";
  percent: number;
  done: number;
  total: number;
  /** Đang tải cuốn KHÁC — mọi nút tải/xoá của cuốn này tạm khoá. */
  busyWithOther: boolean;
  error: string | null;
  sizeLabel: string;
  /** Bấm "tải về": hết chỗ thì mở hộp hỏi xoá cuốn đang chiếm chỗ. */
  requestDownload: () => void;
  /** Bấm "xoá bản offline": mở hộp xác nhận. */
  requestRemove: () => void;
  cancel: () => void;
  /**
   * Hai hộp xác nhận. Bên gọi đặt ở chỗ SỐNG LÂU hơn nút bấm — nút nằm
   * trong menu ⋮ thì menu đóng là nút biến mất, hộp mà nằm chung trong đó
   * cũng mất theo ngay khi vừa mở.
   */
  dialogs: ReactNode;
}

/**
 * Logic tải sách về đọc offline, tách khỏi phần vẽ nút: màn rộng bày thành
 * nút (`OfflineDownload`), điện thoại gom vào menu ⋮ (`BookActionsMenu`).
 *
 * Bốn trạng thái, mỗi trạng thái một hành động rõ ràng: chưa tải (tải về),
 * đang tải (huỷ), đã tải (xoá), và hết chỗ (xoá cuốn kia trước).
 *
 * "Hết chỗ" không phải lỗi nên không báo đỏ — chỉ nói thẳng cuốn nào đang
 * chiếm chỗ và cho xoá ngay tại đó, thay vì bắt người dùng tự đi tìm.
 */
export function useOfflineDownload(book: Book): OfflineDownloadState {
  const { books: offlineBooks, hasLoaded } = useOfflineBooks();
  const active = useDownloadStore((s) => s.active);
  const error = useDownloadStore((s) => s.error);
  const start = useDownloadStore((s) => s.start);
  const cancel = useDownloadStore((s) => s.cancel);
  const remove = useDownloadStore((s) => s.remove);
  const clearError = useDownloadStore((s) => s.clearError);

  const [limitOpen, setLimitOpen] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState(false);

  const isDownloaded = offlineBooks.includes(book.id);
  const isDownloading = active?.bookId === book.id;
  const occupant = offlineBooks.find((id) => id !== book.id);
  const occupantBook = occupant ? getBook(occupant) : undefined;
  const full = !hasFreeSlot(offlineBooks);
  const sizeLabel = formatBytes(estimateBytes(book));
  const done = isDownloading ? active.progress.done : 0;
  const total = isDownloading ? active.progress.total : 0;

  const dialogs = (
    <>
      <Dialog open={confirmRemove} onOpenChange={setConfirmRemove}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Xoá bản offline của {book.titleVi}?</DialogTitle>
            <DialogDescription>
              Giải phóng khoảng {sizeLabel} và cho phép tải cuốn khác về. Tiến độ
              đọc, ghi chú, nét vẽ và bản ghi âm đều KHÔNG bị đụng tới — chỉ xoá
              ảnh đã tải.
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

      <Dialog open={limitOpen} onOpenChange={setLimitOpen}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <WifiOff className="size-4 text-muted-foreground" aria-hidden />
              Đã đủ số sách tải về
            </DialogTitle>
            <DialogDescription>
              Mỗi lần chỉ giữ được một cuốn trên máy, vì mỗi cuốn nặng cỡ {sizeLabel}.
              Hiện{" "}
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

  return {
    // Cache API chỉ có ở trình duyệt — chờ đọc xong rồi mới vẽ, nếu không nút
    // nhảy từ "tải về" sang "đã tải" ngay trước mắt.
    ready: hasLoaded,
    state: isDownloading ? "downloading" : isDownloaded ? "downloaded" : "idle",
    percent: total > 0 ? Math.round((done / total) * 100) : 0,
    done,
    total,
    busyWithOther: active !== null && !isDownloading,
    error,
    sizeLabel,
    requestDownload: () => {
      clearError();
      if (full) setLimitOpen(true);
      else void start(book);
    },
    requestRemove: () => setConfirmRemove(true),
    cancel,
    dialogs,
  };
}

/**
 * Đang tải: phần trăm kèm nút huỷ.
 *
 * Chỉ phần trăm, không kèm thanh tiến độ. Cả hàng nút vừa khít màn điện
 * thoại; thêm một thanh 80px nữa là nó tụt xuống dòng mới ngay giữa lúc đang
 * tải, rồi lại nhảy lên khi tải xong. Vả lại ngay phía trên đã có một thanh
 * tiến độ ĐỌC rồi — hai thanh cạnh nhau chỉ tổ khiến người ta nhìn nhầm cái
 * này ra cái kia.
 */
export function OfflineDownloadProgress({ download }: { download: OfflineDownloadState }) {
  return (
    <div
      className="flex items-center gap-1.5 text-xs text-muted-foreground"
      title={`Đang tải ${download.done}/${download.total} ${PART_LABEL}`}
    >
      <Loader2 className="size-3.5 shrink-0 animate-spin" aria-hidden />
      <span className="tabular-nums" aria-label={`Đang tải ${download.percent}%`}>
        {download.percent}%
      </span>
      <Button
        variant="ghost"
        size="icon"
        className="size-7"
        onClick={download.cancel}
        aria-label="Huỷ tải về"
      >
        <X className="size-3.5" aria-hidden />
      </Button>
    </div>
  );
}

/** Lỗi lần tải gần nhất — dòng chữ đỏ nhỏ ngay dưới nút. */
export function OfflineDownloadError({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <p className="flex items-start gap-1.5 text-xs text-destructive">
      <AlertTriangle className="mt-px size-3.5 shrink-0" aria-hidden />
      {error}
    </p>
  );
}

/** Nút tải sách về đọc offline, bày thẳng trên hàng nút (màn rộng). */
export function OfflineDownload({ book }: { book: Book }) {
  const download = useOfflineDownload(book);
  if (!download.ready) return null;

  if (download.state === "downloading") {
    return <OfflineDownloadProgress download={download} />;
  }

  if (download.state === "downloaded") {
    return (
      <>
        <Tooltip>
          <TooltipTrigger asChild>
            {/* Nhãn "đã tải" và nút xoá gộp làm một: cùng nói về một thứ, mà
                tách ra thì ăn mất hai chỗ trên hàng nút. */}
            <Button
              variant="ghost"
              size="sm"
              onClick={download.requestRemove}
              disabled={download.busyWithOther}
            >
              <Check className="size-4 text-primary" aria-hidden />
              Đã tải offline
            </Button>
          </TooltipTrigger>
          <TooltipContent>Đọc được khi mất mạng — bấm để xoá</TooltipContent>
        </Tooltip>
        {download.dialogs}
      </>
    );
  }

  return (
    <>
      <div className="flex flex-col gap-2">
        {/* Dung lượng nằm ngay trong nhãn nút thay vì một dòng riêng — người
            dùng mạng di động cần biết trước khi bấm, nhưng không cần cả một
            dòng cho nó. */}
        <Button
          variant="outline"
          size="sm"
          disabled={download.busyWithOther}
          onClick={download.requestDownload}
        >
          <Download className="size-4" aria-hidden />
          Tải offline · {download.sizeLabel}
        </Button>
        <OfflineDownloadError error={download.error} />
      </div>
      {download.dialogs}
    </>
  );
}
