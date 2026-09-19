"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  AlertTriangle,
  ArrowUpRight,
  Eraser,
  Eye,
  EyeOff,
  GripVertical,
  Highlighter,
  Minus,
  MoreHorizontal,
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
/** Còn ít hơn chừng này chỗ phía trên thì mở bảng chọn xuống dưới. */
const PANEL_SPACE = 260;

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
 * Thanh công cụ vẽ: một hàng duy nhất, kéo thả được, thu lại được.
 *
 * Chỉ 6 nút nằm ngoài — công cụ đang cầm, hoàn tác, thêm, thu nhỏ, thoát,
 * cùng tay nắm. Bảng chọn công cụ/màu/cỡ và hai việc với ảnh gốc nằm trong
 * hai bảng bung ra. Trước đây bày hết ~20 nút ra ngoài, trên màn điện thoại
 * nó xuống 2–3 dòng che gần hết chỗ đọc, mà nút thu nhỏ thì lọt tít cuối
 * hàng nên không ai tìm thấy.
 *
 * CẢ THÂN THANH là vùng kéo, không riêng tay nắm: tay nắm chỉ rộng chừng
 * 20px, người dùng theo phản xạ sẽ túm vào giữa thanh mà kéo. Tay nắm giữ
 * lại chỉ để làm dấu hiệu "cái này kéo được".
 *
 * Luôn hiện khi còn đang vẽ — khác thanh công cụ đọc vốn ẩn/hiện theo chạm,
 * vì trong chế độ vẽ thì chạm là vẽ. Ẩn nốt thanh này thì người dùng kẹt
 * trong chế độ vẽ mà không có đường ra; thu nhỏ mới là cách lấy lại chỗ.
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

  const [panel, setPanel] = useState<"none" | "tools" | "more">("none");
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
   * nhỏ, xoay máy). Đọc vị trí từ `getState()` chứ không từ closure — effect
   * chạy lại theo `active` nên biến trong closure sẽ cũ.
   *
   * Chạy lại theo `active` vì lúc mount thì chế độ vẽ còn tắt, thanh chưa
   * render nên chưa có gì để đo.
   */
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;

    function fit() {
      if (!el) return;
      const { offsetWidth: w, offsetHeight: h } = el;
      if (w === 0) return;
      const store = useAnnotationStore.getState();
      const current: Pos = store.toolbarPos ?? {
        x: (window.innerWidth - w) / 2,
        y: window.innerHeight - h - BOTTOM_BAR_HEIGHT,
      };
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

  // Bấm ra ngoài hay bấm Esc thì đóng bảng chọn. Dùng pha bắt (capture) trên
  // `pointerdown`: nếu chờ tới lượt nổi bọt thì cú chạm đó đã rơi xuống lớp
  // vẽ và thành một nét trên trang.
  useEffect(() => {
    if (panel === "none") return;
    function onDown(e: PointerEvent) {
      if (!ref.current?.contains(e.target as Node)) setPanel("none");
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.stopPropagation();
        setPanel("none");
      }
    }
    document.addEventListener("pointerdown", onDown, true);
    document.addEventListener("keydown", onKey, true);
    return () => {
      document.removeEventListener("pointerdown", onDown, true);
      document.removeEventListener("keydown", onKey, true);
    };
  }, [panel]);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      // Bấm vào nút thì đừng bắt đầu kéo: setPointerCapture ở đây sẽ dồn mọi
      // pointer event về thân thanh, nút không nhận được `pointerup` nên
      // `click` không bao giờ bắn. Lúc thu nhỏ thì cả nút là vùng kéo, và nó
      // tự phân biệt bấm/kéo ở `onPointerUp`.
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
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) {
        d.moved = true;
        // Kéo thanh đi thì đóng bảng chọn — để mở thì nó lật lên lật xuống
        // theo vị trí, nhìn rất giật.
        setPanel("none");
      }
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
  // Thanh nằm sát mép trên thì bảng chọn phải bung xuống dưới, không thì nó
  // tràn ra ngoài màn.
  const panelBelow = (pos?.y ?? 0) < PANEL_SPACE;

  return (
    <>
      {/* z-30: trên thanh đọc (z-20) nhưng dưới panel bài giảng (z-[60]).
          Chưa đo xong thì giấu đi một khung hình, đỡ thấy nó nhảy vị trí. */}
      <div
        ref={ref}
        className="fixed z-30 touch-none select-none"
        style={{ left: pos?.x ?? 0, top: pos?.y ?? 0, opacity: pos ? 1 : 0 }}
      >
        {collapsed ? (
          <button
            type="button"
            {...dragHandlers}
            aria-label="Mở lại thanh công cụ vẽ"
            title="Mở lại thanh công cụ vẽ"
            className="flex size-11 items-center justify-center rounded-full border border-white/15 bg-neutral-900/90 text-white shadow-2xl backdrop-blur"
          >
            <ToolIcon className="size-5" style={{ color }} aria-hidden />
          </button>
        ) : (
          <div className="relative">
            {panel !== "none" && (
              <div
                className={cn(
                  "absolute left-0 w-64 max-w-[calc(100vw-1rem)] rounded-2xl border border-white/15 bg-neutral-900/95 p-2 text-white shadow-2xl backdrop-blur",
                  panelBelow ? "top-full mt-2" : "bottom-full mb-2"
                )}
              >
                {panel === "tools" ? (
                  <ToolPanel
                    tool={tool}
                    color={color}
                    size={size}
                    onPickTool={(t) => {
                      setTool(t);
                      // Chọn xong công cụ là muốn vẽ ngay — đóng luôn. Còn
                      // màu và cỡ thì để mở, thường chỉnh cả hai một lượt.
                      setPanel("none");
                    }}
                    onPickColor={setColor}
                    onPickSize={setSize}
                  />
                ) : (
                  <MorePanel
                    peeking={peeking}
                    count={count}
                    target={target}
                    onTogglePeek={() => setPeeking(!peeking)}
                    onRestore={() => {
                      setPanel("none");
                      setConfirmClear(true);
                    }}
                  />
                )}
              </div>
            )}

            <div
              {...dragHandlers}
              data-testid="annotation-toolbar-bar"
              className="flex cursor-grab items-center gap-0.5 rounded-full border border-white/15 bg-neutral-900/90 p-1 pl-0.5 text-white shadow-2xl backdrop-blur active:cursor-grabbing"
            >
              <GripVertical className="size-4 shrink-0 text-white/40" aria-hidden />

              <BarButton
                label={`Công cụ: ${TOOLS.find((t) => t.value === tool)?.label}`}
                pressed={panel === "tools"}
                onClick={() => setPanel(panel === "tools" ? "none" : "tools")}
              >
                <ToolIcon
                  className="size-4.5"
                  style={{ color: panel === "tools" ? undefined : color }}
                  aria-hidden
                />
              </BarButton>

              <BarButton
                label={`Hoàn tác nét cuối (trang ${target})`}
                disabled={count === 0}
                onClick={() => undo(bookId, target)}
              >
                <Undo2 className="size-4.5" aria-hidden />
              </BarButton>

              <BarButton
                label="Ảnh gốc và khôi phục"
                pressed={panel === "more" || peeking}
                onClick={() => setPanel(panel === "more" ? "none" : "more")}
              >
                <MoreHorizontal className="size-4.5" aria-hidden />
              </BarButton>

              <span className="mx-0.5 h-5 w-px shrink-0 bg-white/15" aria-hidden />

              <BarButton label="Thu nhỏ thanh công cụ" onClick={() => setCollapsed(true)}>
                <Minus className="size-4.5" aria-hidden />
              </BarButton>

              <BarButton label="Thoát chế độ vẽ (Esc)" onClick={() => setActive(false)}>
                <X className="size-4.5" aria-hidden />
              </BarButton>
            </div>
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
              lấy lại được — muốn xem thử trang sạch thôi thì dùng “Xem ảnh gốc”.
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

/** Nút tròn trên thanh — cùng một cỡ cho mọi nút để hàng không so le. */
function BarButton({
  label,
  onClick,
  children,
  pressed,
  disabled,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
  pressed?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
      aria-pressed={pressed}
      title={label}
      className={cn(
        "flex size-9 shrink-0 items-center justify-center rounded-full transition-colors",
        pressed ? "bg-white text-neutral-900" : "hover:bg-white/10",
        "disabled:opacity-30 disabled:hover:bg-transparent"
      )}
    >
      {children}
    </button>
  );
}

function ToolPanel({
  tool,
  color,
  size,
  onPickTool,
  onPickColor,
  onPickSize,
}: {
  tool: AnnotationTool;
  color: string;
  size: number;
  onPickTool: (t: AnnotationTool) => void;
  onPickColor: (c: string) => void;
  onPickSize: (i: number) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <div className="grid grid-cols-4 gap-1">
        {TOOLS.map(({ value, label, Icon }) => (
          <button
            key={value}
            type="button"
            onClick={() => onPickTool(value)}
            aria-label={label}
            aria-pressed={tool === value}
            title={label}
            className={cn(
              "flex h-10 items-center justify-center rounded-lg transition-colors",
              tool === value ? "bg-white text-neutral-900" : "hover:bg-white/10"
            )}
          >
            <Icon className="size-4.5" aria-hidden />
          </button>
        ))}
      </div>

      {/* Tẩy không có màu hay cỡ nét để chọn — bày ra là bày cái vô dụng. */}
      {tool !== "eraser" && (
        <>
          <div className="flex items-center justify-between px-1">
            {paletteFor(tool).map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => onPickColor(c)}
                aria-label={`Màu ${c}`}
                aria-pressed={color === c}
                title={`Màu ${c}`}
                className={cn(
                  "size-7 rounded-full border-2 transition-transform",
                  color === c ? "scale-110 border-white" : "border-white/25"
                )}
                style={{ backgroundColor: c }}
              />
            ))}
          </div>
          <div className="flex items-center justify-center gap-2">
            {SIZE_DOTS.map((diameter, i) => (
              <button
                key={diameter}
                type="button"
                onClick={() => onPickSize(i)}
                aria-label={`Cỡ ${i + 1}`}
                aria-pressed={size === i}
                className={cn(
                  "flex size-8 items-center justify-center rounded-full transition-colors",
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
          </div>
        </>
      )}
    </div>
  );
}

function MorePanel({
  peeking,
  count,
  target,
  onTogglePeek,
  onRestore,
}: {
  peeking: boolean;
  count: number;
  target: number;
  onTogglePeek: () => void;
  onRestore: () => void;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <PanelRow
        onClick={onTogglePeek}
        pressed={peeking}
        Icon={peeking ? EyeOff : Eye}
        label={peeking ? "Hiện lại nét vẽ" : "Xem ảnh gốc"}
      />
      <PanelRow
        onClick={onRestore}
        disabled={count === 0}
        Icon={RotateCcw}
        label={`Khôi phục ảnh gốc trang ${target}`}
      />
    </div>
  );
}

function PanelRow({
  onClick,
  Icon,
  label,
  pressed,
  disabled,
}: {
  onClick: () => void;
  Icon: typeof Eye;
  label: string;
  pressed?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-pressed={pressed}
      className={cn(
        "flex items-center gap-2 rounded-lg px-2 py-2 text-left text-sm transition-colors",
        pressed ? "bg-white text-neutral-900" : "hover:bg-white/10",
        "disabled:opacity-30 disabled:hover:bg-transparent"
      )}
    >
      <Icon className="size-4 shrink-0" aria-hidden />
      {label}
    </button>
  );
}
