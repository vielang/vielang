"use client";

import { useRouter } from "next/navigation";
import { LocateFixed, Pause, Play, SkipBack, SkipForward, X } from "lucide-react";
import { getSpreadAnchor } from "@/lib/chapters";
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
 * Thanh điều khiển nghe tự động, nằm giữa đáy ngay trên thanh lật trang.
 *
 * Luôn hiện suốt phiên nghe, kể cả khi thanh công cụ đã ẩn: người dùng cần
 * biết đang nghe bài nào và dừng được bất cứ lúc nào. Trong lúc này widget
 * audio thường tự ẩn (xem `AudioWidget`) để không có 2 nguồn tiếng.
 */
export function AutoplayBar({ bookId }: { bookId: string }) {
  const router = useRouter();
  const active = useAutoplayStore((s) => s.bookId === bookId);
  const item = useAutoplayStore((s) => s.queue[s.index]);
  const index = useAutoplayStore((s) => s.index);
  const total = useAutoplayStore((s) => s.queue.length);
  const status = useAutoplayStore((s) => s.status);
  const visiblePages = useAutoplayStore((s) => s.visiblePages);
  const double = useAutoplayStore((s) => s.double);

  if (!active || !item) return null;
  const offPage = !visiblePages.includes(item.page);

  return (
    <div className="fixed inset-x-0 bottom-[calc(max(0.5rem,env(safe-area-inset-bottom))+4rem)] z-30 flex justify-center px-4 pointer-events-none">
      <div
        role="region"
        aria-label="Nghe tự động"
        className="pointer-events-auto flex max-w-full items-center gap-1 rounded-full bg-black/75 py-1 pr-1 pl-3 text-white shadow-lg backdrop-blur"
      >
        <div className="min-w-0 pr-1">
          <div className="truncate text-xs font-medium">{item.label}</div>
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
    </div>
  );
}
