"use client";

import { useLayoutEffect, useRef, useState } from "react";
import {
  AlertTriangle,
  ArrowUpRight,
  Eraser,
  Eye,
  EyeOff,
  GripHorizontal,
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
import { Separator } from "@/components/ui/separator";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
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
import { drawToolbarAnchor } from "@/lib/widget-dock";
import { clampToViewport, useDraggable } from "@/lib/use-draggable";
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

/**
 * Thanh công cụ vẽ: một cột dọc ở mép phải, kéo thả được, thu lại được.
 *
 * Chỉ 5 nút nằm ngoài — công cụ đang cầm, hoàn tác, thêm, thu nhỏ, thoát.
 * Bảng chọn công cụ/màu/cỡ và hai việc với ảnh gốc nằm trong hai popover.
 *
 * CẢ THÂN THANH là vùng kéo, không riêng tay nắm: tay nắm chỉ cao chừng
 * 16px, người dùng theo phản xạ sẽ túm vào giữa thanh mà kéo. Tay nắm giữ
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
  const activeTool = TOOLS.find((t) => t.value === tool) ?? TOOLS[0];
  const ToolIcon = activeTool.Icon;

  const ref = useRef<HTMLDivElement>(null);

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
      const next = clampToViewport(store.toolbarPos ?? drawToolbarAnchor(w, h), w, h);
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

  // Kéo thả: đặt tay vào bất kỳ đâu trên thanh, kể cả trên nút — xem
  // `use-draggable`. Chạm mà không kéo thì bung lại thanh đang thu nhỏ.
  const dragHandlers = useDraggable({
    ref,
    pos,
    setPos,
    onTap: () => {
      if (collapsed) setCollapsed(false);
    },
    // Kéo thanh đi thì đóng popover — để mở thì nó nhảy theo thanh từng khung
    // hình, nhìn rất giật.
    onDragStart: () => setPanel("none"),
  });

  if (!active) return null;

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
            className="flex size-11 items-center justify-center rounded-full bg-neutral-900/80 text-white ring-1 ring-white/15 shadow-lg backdrop-blur-md transition-transform active:scale-95"
          >
            <ToolIcon className="size-5" style={{ color }} aria-hidden />
          </button>
        ) : (
          <div
            {...dragHandlers}
            data-testid="annotation-toolbar-bar"
            className="flex cursor-grab flex-col items-center gap-0.5 rounded-full bg-neutral-900/80 p-1 text-white ring-1 ring-white/15 shadow-lg backdrop-blur-md active:cursor-grabbing"
          >
            <GripHorizontal className="my-0.5 size-4 shrink-0 text-white/40" aria-hidden />

            <Popover
              open={panel === "tools"}
              onOpenChange={(open) => setPanel(open ? "tools" : "none")}
            >
              <Tooltip>
                <TooltipTrigger asChild>
                  <PopoverTrigger asChild>
                    <BarButton
                      label={`Công cụ: ${activeTool.label}`}
                      pressed={panel === "tools"}
                    >
                      <ToolIcon
                        className="size-4.5"
                        style={{ color: panel === "tools" ? undefined : color }}
                        aria-hidden
                      />
                    </BarButton>
                  </PopoverTrigger>
                </TooltipTrigger>
                <TooltipContent side="left">{activeTool.label}</TooltipContent>
              </Tooltip>
              <PopoverContent side="left" align="start" className="w-60">
                <ToolPanel
                  tool={tool}
                  color={color}
                  size={size}
                  onPickTool={(t) => {
                    setTool(t);
                    // Chọn xong công cụ là muốn vẽ ngay — đóng luôn. Còn màu
                    // và cỡ thì để mở, thường chỉnh cả hai một lượt.
                    setPanel("none");
                  }}
                  onPickColor={setColor}
                  onPickSize={setSize}
                />
              </PopoverContent>
            </Popover>

            <Tooltip>
              <TooltipTrigger asChild>
                <BarButton
                  label={`Hoàn tác nét cuối (trang ${target})`}
                  disabled={count === 0}
                  onClick={() => undo(bookId, target)}
                >
                  <Undo2 className="size-4.5" aria-hidden />
                </BarButton>
              </TooltipTrigger>
              <TooltipContent side="left">Hoàn tác nét cuối</TooltipContent>
            </Tooltip>

            <Popover
              open={panel === "more"}
              onOpenChange={(open) => setPanel(open ? "more" : "none")}
            >
              <Tooltip>
                <TooltipTrigger asChild>
                  <PopoverTrigger asChild>
                    <BarButton
                      label="Ảnh gốc và khôi phục"
                      pressed={panel === "more" || peeking}
                    >
                      <MoreHorizontal className="size-4.5" aria-hidden />
                    </BarButton>
                  </PopoverTrigger>
                </TooltipTrigger>
                <TooltipContent side="left">Ảnh gốc</TooltipContent>
              </Tooltip>
              <PopoverContent side="left" align="start" className="w-56">
                <div className="flex flex-col gap-0.5">
                  <PanelRow
                    onClick={() => setPeeking(!peeking)}
                    pressed={peeking}
                    Icon={peeking ? EyeOff : Eye}
                    label={peeking ? "Hiện lại nét vẽ" : "Xem ảnh gốc"}
                  />
                  <PanelRow
                    onClick={() => {
                      setPanel("none");
                      setConfirmClear(true);
                    }}
                    disabled={count === 0}
                    Icon={RotateCcw}
                    label={`Khôi phục ảnh gốc trang ${target}`}
                  />
                </div>
              </PopoverContent>
            </Popover>

            <Separator className="my-0.5 w-5 bg-white/15" />

            <Tooltip>
              <TooltipTrigger asChild>
                <BarButton label="Thu nhỏ thanh công cụ" onClick={() => setCollapsed(true)}>
                  <Minus className="size-4.5" aria-hidden />
                </BarButton>
              </TooltipTrigger>
              <TooltipContent side="left">Thu nhỏ</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <BarButton label="Thoát chế độ vẽ (Esc)" onClick={() => setActive(false)}>
                  <X className="size-4.5" aria-hidden />
                </BarButton>
              </TooltipTrigger>
              <TooltipContent side="left">Thoát chế độ vẽ</TooltipContent>
            </Tooltip>
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

/**
 * Nút trên thanh. Nhận và chuyển tiếp mọi prop còn lại để
 * `PopoverTrigger asChild` / `TooltipTrigger asChild` gắn được ref và các
 * thuộc tính điều khiển của chúng vào đúng thẻ <button> bên dưới.
 */
function BarButton({
  label,
  children,
  pressed,
  ...props
}: {
  label: string;
  children: React.ReactNode;
  pressed?: boolean;
} & React.ComponentProps<typeof Button>) {
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={label}
      aria-pressed={pressed}
      className={cn(
        "size-9 rounded-full text-white hover:bg-white/15 hover:text-white",
        pressed && "bg-white text-neutral-900 hover:bg-white hover:text-neutral-900",
        "disabled:opacity-30"
      )}
      {...props}
    >
      {children}
    </Button>
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
      <ToggleGroup
        type="single"
        value={tool}
        onValueChange={(v) => v && onPickTool(v as AnnotationTool)}
        variant="outline"
        className="grid grid-cols-4"
      >
        {TOOLS.map(({ value, label, Icon }) => (
          <ToggleGroupItem key={value} value={value} aria-label={label} title={label}>
            <Icon className="size-4.5" aria-hidden />
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      {/* Tẩy không có màu hay cỡ nét để chọn — bày ra là bày cái vô dụng. */}
      {tool !== "eraser" && (
        <>
          <Separator />
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
                  "size-7 rounded-full ring-offset-2 ring-offset-popover transition-transform",
                  color === c ? "scale-110 ring-2 ring-foreground" : "ring-1 ring-border"
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
                  size === i ? "bg-accent" : "hover:bg-accent/50"
                )}
              >
                <span
                  className="rounded-full bg-foreground"
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
    <Button
      variant={pressed ? "secondary" : "ghost"}
      onClick={onClick}
      disabled={disabled}
      aria-pressed={pressed}
      className="h-9 w-full justify-start gap-2 px-2 font-normal"
    >
      <Icon className="size-4 shrink-0" aria-hidden />
      {label}
    </Button>
  );
}
