"use client";

import { useState } from "react";
import { AlertTriangle, Eraser, Highlighter, Pen, Trash2, Undo2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  annotationKey,
  paletteFor,
  useAnnotationStore,
  type AnnotationTool,
} from "@/lib/annotation-store";
import { cn } from "@/lib/utils";

const TOOLS: { value: AnnotationTool; label: string; Icon: typeof Pen }[] = [
  { value: "pen", label: "Bút", Icon: Pen },
  { value: "highlighter", label: "Bút dạ quang", Icon: Highlighter },
  { value: "eraser", label: "Tẩy", Icon: Eraser },
];

/** Chấm chọn cỡ nét — đường kính chỉ để gợi ý, không theo tỉ lệ thật. */
const SIZE_DOTS = [3, 5, 8];

/**
 * Thanh công cụ vẽ, nổi lên khi bật chế độ vẽ trên trang sách.
 *
 * Đặt sát mép dưới nhưng trên thanh điều hướng trang, và luôn hiện khi còn
 * đang vẽ — khác thanh công cụ đọc vốn ẩn/hiện theo chạm, vì trong chế độ vẽ
 * thì chạm là vẽ chứ không còn bật/tắt thanh nữa. Nếu thanh này cũng ẩn theo
 * thì người dùng kẹt trong chế độ vẽ mà không có đường ra.
 */
export function AnnotationToolbar({ bookId, pages }: { bookId: string; pages: number[] }) {
  const active = useAnnotationStore((s) => s.active);
  const setActive = useAnnotationStore((s) => s.setActive);
  const tool = useAnnotationStore((s) => s.tool);
  const setTool = useAnnotationStore((s) => s.setTool);
  const size = useAnnotationStore((s) => s.size);
  const setSize = useAnnotationStore((s) => s.setSize);
  const penColor = useAnnotationStore((s) => s.penColor);
  const highlighterColor = useAnnotationStore((s) => s.highlighterColor);
  const setColor = useAnnotationStore((s) => s.setColor);
  const undo = useAnnotationStore((s) => s.undo);
  const clearPage = useAnnotationStore((s) => s.clearPage);
  const lastPage = useAnnotationStore((s) => s.lastPage);
  const quotaExceeded = useAnnotationStore((s) => s.quotaExceeded);

  // Trang mà "hoàn tác"/"xoá hết" nói tới: trang vừa chạm bút, hoặc trang
  // trái khi chưa chạm trang nào. Ở chế độ 1 trang thì luôn là chính nó.
  const target = pages.includes(lastPage ?? NaN) ? (lastPage as number) : pages[0];
  const count = useAnnotationStore(
    (s) => s.strokes[annotationKey(bookId, target)]?.length ?? 0
  );

  const [confirmClear, setConfirmClear] = useState(false);
  const color = tool === "highlighter" ? highlighterColor : penColor;

  if (!active) return null;

  return (
    <>
      {/* z-30: trên thanh đọc (z-20) nhưng dưới panel bài giảng (z-[60]), để
          mở bài giảng ra vẫn kéo panel đè lên được. */}
      <div className="pointer-events-none fixed inset-x-0 bottom-16 z-30 flex justify-center px-3">
        <div className="pointer-events-auto flex max-w-full items-center gap-1 overflow-x-auto rounded-full border border-white/15 bg-neutral-900/90 p-1.5 text-white shadow-2xl backdrop-blur">
          {TOOLS.map(({ value, label, Icon }) => (
            <button
              key={value}
              type="button"
              onClick={() => setTool(value)}
              aria-label={label}
              aria-pressed={tool === value}
              title={label}
              className={cn(
                "flex size-9 shrink-0 items-center justify-center rounded-full transition-colors",
                tool === value ? "bg-white text-neutral-900" : "hover:bg-white/10"
              )}
            >
              <Icon className="size-4.5" aria-hidden />
            </button>
          ))}

          <Divider />

          {/* Tẩy không có màu — đổi bảng màu thành cỡ đầu tẩy thì thừa, nên
              chỉ ẩn đi, giữ nguyên phần chọn cỡ ở dưới. */}
          {tool !== "eraser" &&
            paletteFor(tool).map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => setColor(c)}
                aria-label={`Màu ${c}`}
                aria-pressed={color === c}
                title={`Màu ${c}`}
                className={cn(
                  "size-7 shrink-0 rounded-full border-2 transition-transform",
                  color === c ? "scale-110 border-white" : "border-white/25"
                )}
                style={{ backgroundColor: c }}
              />
            ))}

          {tool !== "eraser" && <Divider />}

          {SIZE_DOTS.map((diameter, i) => (
            <button
              key={diameter}
              type="button"
              onClick={() => setSize(i)}
              aria-label={`Cỡ nét ${i + 1}`}
              aria-pressed={size === i}
              className={cn(
                "flex size-8 shrink-0 items-center justify-center rounded-full transition-colors",
                size === i ? "bg-white/20" : "hover:bg-white/10"
              )}
            >
              <span
                className="rounded-full bg-white"
                style={{ width: diameter, height: diameter }}
                aria-hidden
              />
            </button>
          ))}

          <Divider />

          <button
            type="button"
            onClick={() => undo(bookId, target)}
            disabled={count === 0}
            aria-label={`Hoàn tác nét cuối trên trang ${target}`}
            title={`Hoàn tác nét cuối (trang ${target})`}
            className="flex size-9 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-transparent"
          >
            <Undo2 className="size-4.5" aria-hidden />
          </button>
          <button
            type="button"
            onClick={() => setConfirmClear(true)}
            disabled={count === 0}
            aria-label={`Xoá hết nét trên trang ${target}`}
            title={`Xoá hết nét (trang ${target})`}
            className="flex size-9 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-transparent"
          >
            <Trash2 className="size-4.5" aria-hidden />
          </button>

          <Divider />

          <button
            type="button"
            onClick={() => setActive(false)}
            aria-label="Thoát chế độ vẽ"
            title="Thoát chế độ vẽ (Esc)"
            className="flex size-9 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-white/10"
          >
            <X className="size-4.5" aria-hidden />
          </button>
        </div>
      </div>

      {quotaExceeded && (
        <div className="pointer-events-none fixed inset-x-0 bottom-28 z-30 flex justify-center px-3">
          <p className="pointer-events-auto flex items-center gap-1.5 rounded-full bg-destructive px-3 py-1.5 text-xs text-white shadow-lg">
            <AlertTriangle className="size-3.5" aria-hidden />
            Bộ nhớ trình duyệt đã đầy — nét vẽ chưa lưu được.
          </p>
        </div>
      )}

      <Dialog open={confirmClear} onOpenChange={setConfirmClear}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Xoá hết nét vẽ trang {target}?</DialogTitle>
            <DialogDescription>
              {count} nét trên trang này sẽ bị xoá và không lấy lại được. Ảnh trang
              sách thì không đụng tới.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmClear(false)}>
              Huỷ
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                clearPage(bookId, target);
                setConfirmClear(false);
              }}
            >
              Xoá hết
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

function Divider() {
  return <span className="mx-0.5 h-6 w-px shrink-0 bg-white/15" aria-hidden />;
}
