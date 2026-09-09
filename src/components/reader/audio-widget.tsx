"use client";

import { useEffect, useRef } from "react";
import { GripVertical, Volume2, X } from "lucide-react";
import type { AudioTrack } from "@/lib/audio";
import { cn } from "@/lib/utils";
import { useAudioWidgetStore } from "@/lib/audio-widget-store";

const SIZE = 48; // đường kính nút tròn lúc thu nhỏ
const PANEL_WIDTH = 256;
const MARGIN = 10;
const DRAG_THRESHOLD = 5;

interface Pos {
  x: number;
  y: number;
}

function defaultPosition(): Pos {
  if (typeof window === "undefined") return { x: MARGIN, y: MARGIN };
  return {
    x: window.innerWidth - SIZE - MARGIN,
    // Đặt cao hơn toolbar dưới (~64px) một chút để không đè lên nhau.
    y: window.innerHeight - SIZE - 84,
  };
}

function clamp(pos: Pos, width: number): Pos {
  if (typeof window === "undefined") return pos;
  const maxX = window.innerWidth - width - MARGIN;
  const maxY = window.innerHeight - SIZE - MARGIN;
  return {
    x: Math.min(Math.max(pos.x, MARGIN), Math.max(maxX, MARGIN)),
    y: Math.min(Math.max(pos.y, MARGIN), Math.max(maxY, MARGIN)),
  };
}

/**
 * Widget audio nổi, kéo thả được tới bất kỳ vị trí nào trên màn hình đọc,
 * mặc định thu nhỏ thành 1 nút tròn để không che nội dung ảnh sách — bấm
 * vào mới mở rộng ra nghe. Vị trí/trạng thái thu-phóng giữ nguyên khi
 * chuyển trang (xem lib/audio-widget-store.ts).
 */
export function AudioWidget({ tracks }: { tracks: AudioTrack[] }) {
  const storedPosition = useAudioWidgetStore((s) => s.position);
  const setStoredPosition = useAudioWidgetStore((s) => s.setPosition);
  const collapsed = useAudioWidgetStore((s) => s.collapsed);
  const setCollapsed = useAudioWidgetStore((s) => s.setCollapsed);
  const activeType = useAudioWidgetStore((s) => s.activeType);
  const setActiveType = useAudioWidgetStore((s) => s.setActiveType);

  const drag = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    originX: number;
    originY: number;
    moved: boolean;
  } | null>(null);

  // Lần đầu tiên (chưa từng kéo) — chốt vị trí mặc định (dựa vào kích
  // thước màn hình, chỉ biết được sau khi mount) vào store, để các trang
  // sau (remount) dùng chung 1 giá trị ổn định thay vì tính lại mỗi lần.
  // Render trước khi effect này chạy (SSR + lần vẽ đầu) dùng fallback cố
  // định {MARGIN,MARGIN} — không phụ thuộc window nên không lệch hydration.
  useEffect(() => {
    if (!storedPosition) setStoredPosition(defaultPosition());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const pos = storedPosition ?? { x: MARGIN, y: MARGIN };
  const active = tracks.find((t) => t.type === activeType) ?? tracks[0];
  const width = collapsed ? SIZE : PANEL_WIDTH;

  function onPointerDown(e: React.PointerEvent) {
    // Nút "Thu nhỏ" nằm ngay trong thanh kéo: bấm vào nó thì đừng bắt đầu
    // kéo. setPointerCapture ở đây sẽ chuyển hết pointer event về thanh kéo,
    // nút không nhận được `pointerup` nên `click` không bao giờ bắn.
    // (Nút tròn lúc thu nhỏ thì ngược lại — chính nó là vùng kéo, và nó tự
    // xử lý "nhấn mà không kéo" ở onPointerUp bên dưới.)
    if (!collapsed && (e.target as HTMLElement).closest("button")) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      originX: pos.x,
      originY: pos.y,
      moved: false,
    };
  }

  function onPointerMove(e: React.PointerEvent) {
    const d = drag.current;
    if (!d || d.pointerId !== e.pointerId) return;
    const dx = e.clientX - d.startX;
    const dy = e.clientY - d.startY;
    if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) d.moved = true;
    setStoredPosition(clamp({ x: d.originX + dx, y: d.originY + dy }, width));
  }

  function onPointerUp(e: React.PointerEvent) {
    const d = drag.current;
    if (!d || d.pointerId !== e.pointerId) return;
    drag.current = null;
    // Nhấn (không kéo) trên nút tròn lúc thu nhỏ -> mở rộng ra.
    if (!d.moved && collapsed) setCollapsed(false);
  }

  if (tracks.length === 0) return null;

  return (
    <div
      className="fixed z-30 touch-none select-none"
      style={{ left: pos.x, top: pos.y, width }}
    >
      {collapsed ? (
        <button
          type="button"
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
          aria-label="Mở audio trang này"
          className="flex size-12 items-center justify-center rounded-full bg-black/70 text-white shadow-lg backdrop-blur transition-transform active:scale-95"
        >
          <Volume2 className="size-5" aria-hidden />
        </button>
      ) : (
        <div className="overflow-hidden rounded-xl bg-black/80 text-white shadow-lg backdrop-blur">
          {/* Thanh kéo — chỉ vùng này chịu trách nhiệm drag, để không cấn
              vào thanh trượt/nút play của thẻ audio bên dưới. */}
          <div
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onPointerCancel={onPointerUp}
            className="flex cursor-grab items-center justify-between px-2 py-1.5 active:cursor-grabbing"
          >
            <GripVertical className="size-4 text-white/50" aria-hidden />
            <span className="text-xs font-medium text-white/80">Audio</span>
            <button
              type="button"
              onClick={() => setCollapsed(true)}
              aria-label="Thu nhỏ"
              className="rounded p-0.5 hover:bg-white/10"
            >
              <X className="size-4" aria-hidden />
            </button>
          </div>

          <div className="flex flex-col gap-1.5 px-2.5 pb-2.5">
            {tracks.length > 1 && (
              <div className="flex justify-center gap-1.5">
                {tracks.map((t) => (
                  <button
                    key={t.type}
                    type="button"
                    onClick={() => setActiveType(t.type)}
                    className={cn(
                      "rounded-full px-2 py-0.5 text-[11px] transition-colors",
                      t.type === active.type
                        ? "bg-primary"
                        : "bg-white/10 hover:bg-white/20"
                    )}
                  >
                    {t.label}
                  </button>
                ))}
              </div>
            )}
            {/* key={active.url}: reset trạng thái phát khi đổi track/trang */}
            <audio key={active.url} controls src={active.url} className="h-8 w-full" />
          </div>
        </div>
      )}
    </div>
  );
}
