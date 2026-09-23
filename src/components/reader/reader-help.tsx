"use client";

import { useEffect } from "react";
import {
  Bookmark,
  Hand,
  Keyboard,
  Languages,
  ListChecks,
  Mic,
  Mouse,
  MoveHorizontal,
  NotebookText,
  Pen,
  Pointer,
  Volume2,
  ZoomIn,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { useReaderPrefsStore } from "@/lib/reader-prefs-store";

/**
 * Thao tác bằng tay. Đây là phần đáng nói nhất: không thao tác nào trong số
 * này có dấu hiệu gì trên màn hình, nên người mở sách lần đầu sẽ không bao
 * giờ tự tìm ra.
 */
const GESTURES: { Icon: typeof Hand; what: string; how: string }[] = [
  { Icon: Pointer, what: "Chạm mép trái / mép phải", how: "lật về trang trước / sang trang sau" },
  { Icon: Pointer, what: "Chạm giữa trang", how: "ẩn hoặc hiện thanh công cụ" },
  { Icon: MoveHorizontal, what: "Vuốt ngang", how: "lật sang trang trước / trang sau" },
  {
    Icon: ZoomIn,
    what: "Chụm 2 ngón, hoặc chạm 2 lần",
    how: "phóng to để soi chữ — chạm 2 lần nữa là về như cũ. Đang phóng thì kéo để xem quanh trang, kéo tới mép rồi kéo tiếp là lật trang; mức phóng giữ nguyên khi lật",
  },
  {
    Icon: Mouse,
    what: "Trên máy tính: lăn chuột",
    how: "lật trang; Ctrl (hoặc ⌘) + lăn để phóng to; đang phóng thì lăn để cuộn trong trang, Shift + lăn để cuộn ngang",
  },
  {
    Icon: Languages,
    what: "Chạm vào một đoạn chữ Hàn",
    how: "hiện bản dịch tiếng Việt của đoạn đó — kéo bong bóng đi chỗ khác được",
  },
  {
    Icon: ListChecks,
    what: "Chạm vào dấu danh sách cạnh bài tập",
    how: "xem đáp án của sách rồi tự chấm — mình sẽ gợi ý bài nên ôn ở Góc học tập",
  },
  { Icon: Hand, what: "Kéo các bảng công cụ", how: "đặt chúng vào chỗ vừa tay" },
];

const TOOLS: { Icon: typeof Hand; label: string; desc: string }[] = [
  { Icon: NotebookText, label: "Bài giảng", desc: "giải thích trang này bằng tiếng Việt, sửa được" },
  { Icon: Pen, label: "Vẽ", desc: "khoanh, gạch chân, viết chú thích lên trang" },
  { Icon: Mic, label: "Ghi âm", desc: "đọc to rồi nghe lại để so cách phát âm" },
  { Icon: Volume2, label: "Audio", desc: "bài nghe của trang, nếu có" },
  { Icon: Bookmark, label: "Đánh dấu", desc: "ghim trang để quay lại sau" },
];

const SHORTCUTS: { keys: string[]; what: string }[] = [
  { keys: ["←", "→"], what: "Lật trang" },
  { keys: ["Home", "End"], what: "Trang đầu / trang cuối" },
  { keys: ["N"], what: "Bài giảng" },
  { keys: ["D"], what: "Bật/tắt chế độ vẽ" },
  { keys: ["+", "−"], what: "Phóng to / thu nhỏ" },
  { keys: ["0"], what: "Về 100%" },
  { keys: ["B"], what: "Đánh dấu trang" },
  { keys: ["?"], what: "Mở lại bảng này" },
  { keys: ["Esc"], what: "Đóng sách" },
];

/**
 * Bảng "cách dùng" của trang đọc.
 *
 * Tự hiện MỘT LẦN ở lần mở sách đầu tiên, sau đó chỉ mở lại qua nút `?` ở
 * thanh dưới hoặc phím `?`. Trang đọc giấu gần hết thao tác đi để nhường
 * chỗ cho ảnh sách — đổi lại thì phải nói ra một lượt, nếu không người dùng
 * chỉ biết mỗi hai nút mũi tên.
 *
 * Cờ "đã xem" chỉ được ghi lúc ĐÓNG chứ không phải lúc mở: lật trang ngay
 * khi bảng vừa bật lên thì lần sau vẫn được xem lại.
 */
export function ReaderHelp({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const markSeen = useReaderPrefsStore((s) => s.markReaderHelpSeen);

  // Đọc cờ trong effect chứ không lúc render: store này nạp đồng bộ từ
  // localStorage nên đọc lúc render sẽ lệch với HTML server render.
  useEffect(() => {
    if (!useReaderPrefsStore.getState().hasSeenReaderHelp) onOpenChange(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ hỏi đúng một lần lúc mount
  }, []);

  function close() {
    markSeen();
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <DialogContent className="max-h-[85dvh] overflow-y-auto sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Cách dùng trang đọc</DialogTitle>
          <DialogDescription>
            Trang đọc để trống cho ảnh sách, nên hầu hết thao tác nằm ở cử chỉ
            tay. Mở lại bảng này bất cứ lúc nào bằng nút{" "}
            <span className="font-medium text-foreground">?</span> ở thanh dưới.
          </DialogDescription>
        </DialogHeader>

        <section className="flex flex-col gap-2.5">
          {GESTURES.map(({ Icon, what, how }) => (
            <div key={what} className="flex items-start gap-2.5">
              <Icon className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
              <p className="text-sm leading-snug">
                <span className="font-medium">{what}</span>{" "}
                <span className="text-muted-foreground">— {how}</span>
              </p>
            </div>
          ))}
        </section>

        <Separator />

        <section className="flex flex-col gap-2">
          <h3 className="text-xs font-medium text-muted-foreground">
            Các nút trên thanh công cụ
          </h3>
          <div className="grid gap-2 sm:grid-cols-2">
            {TOOLS.map(({ Icon, label, desc }) => (
              <div key={label} className="flex items-start gap-2">
                <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden />
                <p className="text-sm leading-snug">
                  <span className="font-medium">{label}</span>
                  <span className="block text-xs text-muted-foreground">{desc}</span>
                </p>
              </div>
            ))}
          </div>
        </section>

        <Separator />

        <section className="flex flex-col gap-2">
          <h3 className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <Keyboard className="size-3.5" aria-hidden />
            Phím tắt (khi dùng máy tính)
          </h3>
          <dl className="grid gap-1.5">
            {SHORTCUTS.map(({ keys, what }) => (
              <div key={what} className="flex items-center justify-between gap-3">
                <dt className="text-sm text-muted-foreground">{what}</dt>
                <dd className="flex gap-1">
                  {keys.map((k) => (
                    <kbd
                      key={k}
                      className="rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[11px] leading-none"
                    >
                      {k}
                    </kbd>
                  ))}
                </dd>
              </div>
            ))}
          </dl>
        </section>

        <DialogFooter>
          <Button onClick={close} className="w-full sm:w-auto">
            Đã hiểu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
