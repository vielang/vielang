"use client";

import { useCallback, useLayoutEffect, useRef, useState } from "react";
import {
  AlertTriangle,
  ArrowUpRight,
  Eraser,
  Eye,
  EyeOff,
  GripVertical,
  Highlighter,
  Maximize2,
  Minimize2,
  Minus,
  Pen,
  RotateCcw,
  Square,
  Type,
  Undo2,
  X,
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
  { value: "arrow", label: "Mũi tên", Icon: ArrowUpRight },
  { value: "line", label: "Đường thẳng", Icon: Minus },
  { value: "rect", label: "Khung", Icon: Square },
  { value: "text", label: "Chữ", Icon: Type },
  { value: "eraser", label: "Tẩy", Icon: Eraser },
];

/** Chấm chọn cỡ nét — đường kính chỉ để gợi ý, không theo tỉ lệ thật. */
const SIZE_DOTS = [3, 5, 8];

const MARGIN = 8;
/** Chừa chỗ cho thanh điều hướng trang ở đáy khi đặt vị trí mặc định. */
const BOTTOM_BAR_HEIGHT = 76;
/** Xê dịch dưới ngưỡng này thì coi là bấm chứ không phải kéo. */
const DRAG_THRESHOLD = 5;

interface Pos {
  x: number;
  y: number;
}

/** Kẹp trong màn hình theo đúng kích thước hiện tại của thanh. */
function clamp(pos: Pos, width: number, height: number): Pos {
  if (typeof window === "undefined") return pos;
  const maxX = window.innerWidth - width - MARGIN;
  const maxY = window.innerHeight - height - MARGIN;
  return {
    x: Math.min(Math.max(pos.x, MARGIN), Math.max(maxX, MARGIN)),
    y: Math.min(Math.max(pos.y, MARGIN), Math.max(maxY, MARGIN)),
  };
}

/**
 * Thanh công cụ vẽ: kéo thả tới bất kỳ chỗ nào trên màn, và thu lại thành
 * một nút nhỏ khi cần chỗ đọc.
 *
 * Luôn hiện khi còn đang vẽ — khác thanh công cụ đọc vốn ẩn/hiện theo chạm,
 * vì trong chế độ vẽ thì chạm là vẽ chứ không còn bật/tắt thanh nữa. Ẩn nốt
 * thanh này thì người dùng kẹt trong chế độ vẽ mà không có đường ra; thu nhỏ
 * mới là cách lấy lại chỗ.
 *
 * Kích thước KHÔNG cố định (thanh tự xuống dòng theo bề rộng màn, và ẩn bảng
 * màu khi chọn tẩy) nên phải đo bằng ResizeObserver rồi kẹp lại — đặt hằng số
 * cứng thì đổi cỡ màn hay đổi công cụ là thanh thò ra ngoài, mang theo cả nút
 * thoát.
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
  const peeking = useAnnotationStore((s) => s.peeking);
  const setPeeking = useAnnotationStore((s) => s.setPeeking);
  const undo = useAnnotationStore((s) => s.undo);
  const clearPage = useAnnotationStore((s) => s.clearPage);
  const lastPage = useAnnotationStore((s) => s.lastPage);
  const quotaExceeded = useAnnotationStore((s) => s.quotaExceeded);
  const pos = useAnnotationStore((s) => s.toolbarPos);
  const setPos = useAnnotationStore((s) => s.setToolbarPos);
  const collapsed = useAnnotationStore((s) => s.toolbarCollapsed);
  const setCollapsed = useAnnotationStore((s) => s.setToolbarCollapsed);

  // Trang mà "hoàn tác"/"khôi phục" nói tới: trang vừa chạm bút, hoặc trang
  // trái khi chưa chạm trang nào. Ở chế độ 1 trang thì luôn là chính nó.
  const target = pages.includes(lastPage ?? NaN) ? (lastPage as number) : pages[0];
  const count = useAnnotationStore(
    (s) => s.strokes[annotationKey(bookId, target)]?.length ?? 0
  );

  const [confirmClear, setConfirmClear] = useState(false);
  const color = tool === "highlighter" ? highlighterColor : penColor;
  const ToolIcon = TOOLS.find((t) => t.value === tool)?.Icon ?? Pen;

  const ref = useRef<HTMLDivElement>(null);
  const drag = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    origin: Pos;
    moved: boolean;
  } | null>(null);

  /**
   * Đặt vị trí mặc định ở lần mở đầu, rồi kẹp lại mỗi khi thanh đổi cỡ (thu
   * nhỏ, đổi công cụ, xoay máy). Đọc vị trí từ `getState()` chứ không từ
   * closure — effect này chỉ chạy lúc mount nên biến trong closure sẽ cũ.
   */
  // Chạy lại theo `active`: lúc mount thì chế độ vẽ còn tắt nên thanh chưa
  // render, chưa có gì để đo.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;

    function fit() {
      if (!el) return;
      const { offsetWidth: w, offsetHeight: h } = el;
      if (w === 0) return;
      const store = useAnnotationStore.getState();
      const current =
        store.toolbarPos ??
        ({
          x: (window.innerWidth - w) / 2,
          y: window.innerHeight - h - BOTTOM_BAR_HEIGHT,
        } as Pos);
      const next = clamp(current, w, h);
      if (next.x !== store.toolbarPos?.x || next.y !== store.toolbarPos?.y) {
        store.setToolbarPos(next);
      }
    }

    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, [active]);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      // Lúc mở rộng, chỉ tay nắm mới kéo — bấm vào nút mà setPointerCapture
      // thì mọi pointer event dồn hết về tay nắm, nút không nhận `pointerup`
      // nên `click` không bao giờ bắn. Lúc thu nhỏ thì cả nút là vùng kéo,
      // và nó tự phân biệt bấm/kéo ở `onPointerUp`.
      if (!collapsed && (e.target as HTMLElement).closest("button")) return;
      e.currentTarget.setPointerCapture(e.pointerId);
      drag.current = {
        pointerId: e.pointerId,
        startX: e.clientX,
        startY: e.clientY,
        origin: pos ?? { x: MARGIN, y: MARGIN },
        moved: false,
      };
    },
    [collapsed, pos]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      const d = drag.current;
      if (!d || d.pointerId !== e.pointerId || !ref.current) return;
      const dx = e.clientX - d.startX;
      const dy = e.clientY - d.startY;
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) d.moved = true;
      setPos(
        clamp(
          { x: d.origin.x + dx, y: d.origin.y + dy },
          ref.current.offsetWidth,
          ref.current.offsetHeight
        )
      );
    },
    [setPos]
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      const d = drag.current;
      if (!d || d.pointerId !== e.pointerId) return;
      drag.current = null;
      // Nhấn (không kéo) lên nút đã thu nhỏ -> bung ra tại chỗ.
      if (!d.moved && collapsed) setCollapsed(false);
    },
    [collapsed, setCollapsed]
  );

  if (!active) return null;

  const dragHandlers = {
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onPointerCancel: onPointerUp,
  };

  return (
    <>
      {/* z-30: trên thanh đọc (z-20) nhưng dưới panel bài giảng (z-[60]), để
          mở bài giảng ra vẫn kéo panel đè lên được.
          Chưa đo xong thì giấu đi một khung hình, đỡ thấy nó nhảy vị trí. */}
      <div
        ref={ref}
        className="fixed z-30 touch-none select-none"
        style={{
          left: pos?.x ?? 0,
          top: pos?.y ?? 0,
          opacity: pos ? 1 : 0,
        }}
      >
        {collapsed ? (
          <button
            type="button"
            {...dragHandlers}
            aria-label="Mở lại thanh công cụ vẽ"
            title="Mở lại thanh công cụ vẽ"
            className="flex items-center gap-1.5 rounded-full border border-white/15 bg-neutral-900/90 py-1.5 pr-2.5 pl-2 text-white shadow-2xl backdrop-blur"
          >
            <GripVertical className="size-4 text-white/50" aria-hidden />
            <ToolIcon className="size-4.5" aria-hidden />
            {tool !== "eraser" && (
              <span
                className="size-3 rounded-full border border-white/30"
                style={{ backgroundColor: color }}
                aria-hidden
              />
            )}
            <Maximize2 className="size-3.5 text-white/60" aria-hidden />
          </button>
        ) : (
          // `flex-wrap` chứ không phải cuộn ngang: 7 công cụ + màu + cỡ + các
          // nút hành động là quá rộng cho màn điện thoại, mà giấu nút sau một
          // thanh cuộn thì người dùng không biết là còn nữa.
          <div className="flex max-w-[min(34rem,calc(100vw-1rem))] flex-wrap items-center justify-center gap-1 rounded-2xl border border-white/15 bg-neutral-900/90 p-1.5 text-white shadow-2xl backdrop-blur">
            <span
              {...dragHandlers}
              role="separator"
              aria-label="Kéo để đổi chỗ thanh công cụ"
              title="Kéo để đổi chỗ"
              className="flex h-9 cursor-grab items-center px-0.5 text-white/50 active:cursor-grabbing"
            >
              <GripVertical className="size-4" aria-hidden />
            </span>

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
            {/* Hé xem ảnh gốc: giấu tạm hết nét để đối chiếu với trang sạch.
                Đứng ngay cạnh nút khôi phục vì hai nút là một cặp — xem thử
                trước, rồi mới quyết định có xoá thật hay không. */}
            <button
              type="button"
              onClick={() => setPeeking(!peeking)}
              aria-label={peeking ? "Hiện lại nét vẽ" : "Xem ảnh gốc, giấu tạm nét vẽ"}
              aria-pressed={peeking}
              title={peeking ? "Hiện lại nét vẽ" : "Xem ảnh gốc"}
              className={cn(
                "flex size-9 shrink-0 items-center justify-center rounded-full transition-colors",
                peeking ? "bg-white text-neutral-900" : "hover:bg-white/10"
              )}
            >
              {peeking ? (
                <EyeOff className="size-4.5" aria-hidden />
              ) : (
                <Eye className="size-4.5" aria-hidden />
              )}
            </button>
            <button
              type="button"
              onClick={() => setConfirmClear(true)}
              disabled={count === 0}
              aria-label={`Khôi phục ảnh gốc trang ${target}`}
              title={`Khôi phục ảnh gốc (trang ${target})`}
              className="flex size-9 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-transparent"
            >
              <RotateCcw className="size-4.5" aria-hidden />
            </button>

            <Divider />

            <button
              type="button"
              onClick={() => setCollapsed(true)}
              aria-label="Thu nhỏ thanh công cụ"
              title="Thu nhỏ thanh công cụ"
              className="flex size-9 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-white/10"
            >
              <Minimize2 className="size-4.5" aria-hidden />
            </button>
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
        )}
      </div>

      {quotaExceeded && (
        // Cố định ở đáy chứ không bám theo thanh công cụ: thanh có thể đã bị
        // kéo ra mép hay thu nhỏ, mà báo lỗi thì phải luôn đọc được.
        <div className="pointer-events-none fixed inset-x-0 bottom-16 z-30 flex justify-center px-3">
          <p className="pointer-events-auto flex items-center gap-1.5 rounded-full bg-destructive px-3 py-1.5 text-xs text-white shadow-lg">
            <AlertTriangle className="size-3.5" aria-hidden />
            Chưa lưu được nét vẽ — bộ nhớ trình duyệt bị chặn hoặc đã đầy.
          </p>
        </div>
      )}

      <Dialog open={confirmClear} onOpenChange={setConfirmClear}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Khôi phục ảnh gốc trang {target}?</DialogTitle>
            <DialogDescription>
              Trang sẽ trở lại đúng như lúc chưa vẽ gì. {count} nét bị xoá và không
              lấy lại được — muốn xem thử trang sạch thôi thì dùng nút “Xem ảnh gốc”.
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
              Khôi phục ảnh gốc
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
