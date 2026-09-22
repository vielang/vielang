"use client";

import { Minus, Plus } from "lucide-react";
import { Slider } from "@/components/ui/slider";
import { cn } from "@/lib/utils";
import { ZOOM_BAR_TOP } from "@/lib/widget-dock";
import { MAX_ZOOM, MIN_ZOOM, ZOOM_STEP, nextZoom, useZoomStore } from "@/lib/zoom-store";

/**
 * Thanh phóng to / thu nhỏ đứng dọc ở mép phải — CHỈ trên màn rộng.
 *
 * Vì sao cần: trên máy tính, lăn chuột đã dành cho việc di chuyển vùng xem
 * (xem `wheel` trong `page-viewer`), nên người dùng chuột thường chỉ còn mỗi
 * bấm đúp để phóng to — không có cách nào chỉnh vừa phải. Điện thoại thì
 * không cần: chụm hai ngón vốn đã tự nhiên, còn màn hình quá chật cho thêm
 * một thanh.
 *
 * Ẩn/hiện cùng thanh công cụ (chạm giữa trang), để trang sách vẫn gọn khi
 * đang đọc. Bấm vào con số phần trăm là về 100%.
 */
export function ZoomBar({
  visible,
  onZoomTo,
}: {
  visible: boolean;
  onZoomTo: (scale: number) => void;
}) {
  const scale = useZoomStore((s) => s.scale);
  const percent = Math.round(scale * 100);
  const atMin = scale <= MIN_ZOOM + 0.001;
  const atMax = scale >= MAX_ZOOM - 0.001;

  return (
    <div
      role="group"
      aria-label="Phóng to, thu nhỏ trang"
      style={{ top: ZOOM_BAR_TOP }}
      className={cn(
        "fixed right-2 z-20 flex flex-col items-center gap-1 rounded-full bg-neutral-900/80 px-1 py-1.5 text-white shadow-lg ring-1 ring-white/15 backdrop-blur-md transition-opacity duration-200",
        visible ? "opacity-100" : "pointer-events-none opacity-0"
      )}
    >
      <button
        type="button"
        onClick={() => onZoomTo(nextZoom(scale, 1))}
        disabled={atMax}
        aria-label="Phóng to"
        title="Phóng to (+)"
        className="flex size-9 items-center justify-center rounded-full transition-colors hover:bg-white/15 disabled:opacity-35"
      >
        <Plus className="size-4" aria-hidden />
      </button>

      <Slider
        orientation="vertical"
        min={MIN_ZOOM * 100}
        max={MAX_ZOOM * 100}
        step={ZOOM_STEP * 100}
        value={[percent]}
        onValueChange={([v]) => onZoomTo(v / 100)}
        aria-label="Mức phóng to"
        className="!min-h-28 h-28 py-1 [&_[data-slot=slider-range]]:bg-white [&_[data-slot=slider-thumb]]:border-white [&_[data-slot=slider-track]]:bg-white/25"
      />

      <button
        type="button"
        onClick={() => onZoomTo(nextZoom(scale, -1))}
        disabled={atMin}
        aria-label="Thu nhỏ"
        title="Thu nhỏ (−)"
        className="flex size-9 items-center justify-center rounded-full transition-colors hover:bg-white/15 disabled:opacity-35"
      >
        <Minus className="size-4" aria-hidden />
      </button>

      <button
        type="button"
        onClick={() => onZoomTo(MIN_ZOOM)}
        aria-label={`Đang phóng ${percent}% — bấm để về 100%`}
        title="Về 100% (0)"
        className="w-10 rounded-full py-1 text-[11px] font-medium tabular-nums transition-colors hover:bg-white/15"
      >
        {percent}%
      </button>
    </div>
  );
}
