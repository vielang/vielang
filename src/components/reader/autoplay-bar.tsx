"use client";

import { useCallback, useLayoutEffect, useRef } from "react";
import { useRouter } from "next/navigation";
import {
  GripVertical,
  LocateFixed,
  Pause,
  Play,
  SkipBack,
  SkipForward,
  X,
} from "lucide-react";
import { getSpreadAnchor } from "@/lib/chapters";
import { cn } from "@/lib/utils";
import { clampToViewport, useDraggable, type DragPos } from "@/lib/use-draggable";
import { autoplayBarAnchor } from "@/lib/widget-dock";
import {
  autoplayNext,
  autoplayPrev,
  stopAutoplay,
  toggleAutoplayPause,
  useAutoplayStore,
} from "@/lib/autoplay-player";

const BTN =
  "flex size-9 shrink-0 items-center justify-center rounded-full hover:bg-white/10 active:scale-95";

/**
 * Thanh điều khiển nghe tự động, kéo thả được tới bất kỳ đâu trên màn hình.
 *
 * Luôn hiện suốt phiên nghe, kể cả khi thanh công cụ đã ẩn: người dùng cần
 * biết đang nghe bài nào và dừng được bất cứ lúc nào. Trong lúc này widget
 * audio thường tự ẩn (xem `AudioWidget`) để không có 2 nguồn tiếng.
 *
 * Chưa kéo lần nào thì đứng ở chỗ mặc định và bám theo thanh lật trang (xem
 * `autoplayBarAnchor`). Kéo đi rồi thì đứng yên đó, qua cả các lần lật trang.
 */
export function AutoplayBar({
  bookId,
  toolbarVisible,
}: {
  bookId: string;
  /** Thanh lật trang đang hiện — quyết định chỗ mặc định, xem `autoplayBarAnchor`. */
  toolbarVisible: boolean;
}) {
  const router = useRouter();
  const active = useAutoplayStore((s) => s.bookId === bookId);
  const item = useAutoplayStore((s) => s.queue[s.index]);
  const index = useAutoplayStore((s) => s.index);
  const total = useAutoplayStore((s) => s.queue.length);
  const status = useAutoplayStore((s) => s.status);
  const visiblePages = useAutoplayStore((s) => s.visiblePages);
  const double = useAutoplayStore((s) => s.double);
  const pos = useAutoplayStore((s) => s.barPos);
  const moved = useAutoplayStore((s) => s.barMoved);
  const setBarPos = useAutoplayStore((s) => s.setBarPos);

  const ref = useRef<HTMLDivElement>(null);
  const moveTo = useCallback((p: DragPos) => setBarPos(p, true), [setBarPos]);
  const dragHandlers = useDraggable({ ref, pos, setPos: moveTo });
  const shown = active && item !== undefined;

  // Đặt chỗ mặc định (khi chưa kéo) rồi kẹp lại trong màn hình mỗi khi thanh
  // đổi cỡ (hiện/ẩn nút "Về trang đang phát", tên bài dài ngắn) hoặc cửa sổ
  // đổi cỡ — cùng cách với bảng ghi âm.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    function fit() {
      if (!el) return;
      const { offsetWidth: w, offsetHeight: h } = el;
      if (w === 0) return;
      const s = useAutoplayStore.getState();
      const base = s.barMoved && s.barPos ? s.barPos : autoplayBarAnchor(w, h, toolbarVisible);
      const next = clampToViewport(base, w, h);
      if (next.x !== s.barPos?.x || next.y !== s.barPos?.y) s.setBarPos(next, s.barMoved);
    }
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, [shown, toolbarVisible]);

  if (!shown) return null;
  const offPage = !visiblePages.includes(item.page);

  return (
    <div
      ref={ref}
      role="region"
      aria-label="Nghe tự động"
      {...dragHandlers}
      style={{
        left: pos?.x ?? 0,
        top: pos?.y ?? 0,
        // Lượt vẽ đầu chưa đo được cỡ nên chưa biết đặt đâu — ẩn đi một nhịp
        // thay vì loé lên ở góc trên trái.
        visibility: pos ? undefined : "hidden",
      }}
      className={cn(
        "fixed z-30 flex max-w-[calc(100vw-1rem)] cursor-grab touch-none items-center gap-1 rounded-full bg-black/75 py-1 pr-1 pl-1 text-white shadow-lg backdrop-blur select-none active:cursor-grabbing",
        // Trượt theo thanh lật trang lúc còn ở chỗ mặc định; đang kéo mà có
        // hiệu ứng thì thanh lết theo ngón tay chậm một nhịp.
        !moved && "transition-[top] duration-200"
      )}
    >
      <GripVertical className="size-4 shrink-0 text-white/40" aria-hidden />
      <div className="min-w-0 pr-1">
        <div className="max-w-[9rem] truncate text-xs font-medium">{item.label}</div>
        <div className="text-[11px] text-white/70 tabular-nums">
          Trang {item.page} · {index + 1}/{total}
        </div>
      </div>
      {offPage && (
        <button
          type="button"
          className={BTN}
          aria-label="Về trang đang phát"
          title="Về trang đang phát"
          onClick={() =>
            router.push(`/read/${bookId}/${double ? getSpreadAnchor(bookId, item.page) : item.page}`)
          }
        >
          <LocateFixed className="size-4" aria-hidden />
        </button>
      )}
      <button type="button" className={BTN} aria-label="Bài trước" onClick={autoplayPrev}>
        <SkipBack className="size-4" aria-hidden />
      </button>
      <button
        type="button"
        className={BTN}
        aria-label={status === "playing" ? "Tạm dừng" : "Phát"}
        onClick={toggleAutoplayPause}
      >
        {status === "playing" ? (
          <Pause className="size-5" aria-hidden />
        ) : (
          <Play className="size-5" aria-hidden />
        )}
      </button>
      <button type="button" className={BTN} aria-label="Bài sau" onClick={autoplayNext}>
        <SkipForward className="size-4" aria-hidden />
      </button>
      <button type="button" className={BTN} aria-label="Tắt nghe tự động" onClick={stopAutoplay}>
        <X className="size-4" aria-hidden />
      </button>
    </div>
  );
}
