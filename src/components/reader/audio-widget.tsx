"use client";

import { useEffect, useRef } from "react";
import { GripVertical, Volume2, X } from "lucide-react";
import type { AudioTrack } from "@/lib/audio";
import { cn } from "@/lib/utils";
import { useAudioWidgetStore } from "@/lib/audio-widget-store";
import { AUDIO_WIDGET_SIZE, audioAnchor } from "@/lib/widget-dock";

// Đường kính nút tròn lúc thu nhỏ. Các widget nổi khác neo theo số này nên
// nó nằm ở bến đỗ chung — xem widget-dock.
const SIZE = AUDIO_WIDGET_SIZE;
const PANEL_WIDTH = 256;
/** Chiều cao panel: thanh kéo + thẻ audio. */
const PANEL_BASE_HEIGHT = 74;
/** Hàng chọn track (hoặc hàng chọn trang) cao bằng nhau, chỉ xuất hiện khi cần. */
const TRACK_ROW_HEIGHT = 28;

function panelHeightFor(trackCount: number, showPageSelector: boolean): number {
  return (
    PANEL_BASE_HEIGHT +
    (trackCount > 1 ? TRACK_ROW_HEIGHT : 0) +
    (showPageSelector ? TRACK_ROW_HEIGHT : 0)
  );
}
const MARGIN = 10;
const DRAG_THRESHOLD = 5;

interface Pos {
  x: number;
  y: number;
}

function defaultCollapsedPosition(): Pos {
  if (typeof window === "undefined") return { x: MARGIN, y: MARGIN };
  return audioAnchor();
}

/**
 * Mở panel ngay tại chỗ nút tròn đang đứng, nhưng bung sang TRÁI và LÊN TRÊN
 * (neo theo mép phải-dưới của nút) — nút mặc định nằm góc dưới phải nên panel
 * cũng mở ra ở góc dưới phải, không đè vào giữa trang sách.
 *
 * Neo theo mép phải-dưới chứ không giữ nguyên (x, y) là điểm mấu chốt: bề
 * rộng nhảy 48 -> 256, giữ nguyên x thì panel tràn hẳn khỏi mép phải, mang
 * theo cả nút "Thu nhỏ" ra ngoài màn hình.
 */
function panelPositionFrom(collapsedPos: Pos, panelHeight: number): Pos {
  return clamp(
    {
      x: collapsedPos.x + SIZE - PANEL_WIDTH,
      y: collapsedPos.y + SIZE - panelHeight,
    },
    PANEL_WIDTH,
    panelHeight
  );
}

/**
 * Kẹp trong màn hình theo ĐÚNG kích thước của trạng thái hiện tại. Trước đây
 * chiều cao luôn tính bằng SIZE và clamp chỉ chạy lúc kéo, nên mở panel ở sát
 * mép phải là nó tràn hẳn ra ngoài — mang theo cả nút thu nhỏ, thành ra mở
 * rồi không đóng lại được.
 */
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
 * Widget audio nổi, kéo thả được tới bất kỳ vị trí nào trên màn hình đọc,
 * mặc định thu nhỏ thành 1 nút tròn để không che nội dung ảnh sách — bấm
 * vào mới mở rộng ra nghe. Vị trí/trạng thái thu-phóng giữ nguyên khi
 * chuyển trang (xem lib/audio-widget-store.ts).
 */
export function AudioWidget({
  pages,
  tracksByPage,
}: {
  /** 1 hoặc 2 trang tuỳ chế độ xem — xem NotePanel (cùng ý tưởng). */
  pages: number[];
  tracksByPage: Record<number, AudioTrack[]>;
}) {
  const panelPosition = useAudioWidgetStore((s) => s.position);
  const setPanelPosition = useAudioWidgetStore((s) => s.setPosition);
  const collapsedPosition = useAudioWidgetStore((s) => s.collapsedPosition);
  const setCollapsedPosition = useAudioWidgetStore((s) => s.setCollapsedPosition);
  const collapsed = useAudioWidgetStore((s) => s.collapsed);
  const setCollapsed = useAudioWidgetStore((s) => s.setCollapsed);
  const activeType = useAudioWidgetStore((s) => s.activeType);
  const setActiveType = useAudioWidgetStore((s) => s.setActiveType);
  const storedSide = useAudioWidgetStore((s) => s.side);
  const setSide = useAudioWidgetStore((s) => s.setSide);

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
    if (!collapsedPosition) setCollapsedPosition(defaultCollapsedPosition());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const leftPage = pages[0];
  const rightPage = pages[1];
  const leftTracks = tracksByPage[leftPage] ?? [];
  const rightTracks = rightPage !== undefined ? tracksByPage[rightPage] ?? [] : [];
  const hasLeft = leftTracks.length > 0;
  const hasRight = rightTracks.length > 0;

  // Chỉ hiện dải chọn "Trang N" khi CẢ 2 trang đều có audio — 1 trang có, 1
  // trang không thì chả có gì để chọn, hiện thẳng trang có audio luôn.
  const showPageSelector = hasLeft && hasRight;
  const side = showPageSelector
    ? storedSide
    : hasLeft
      ? "left"
      : "right";
  const tracks = side === "right" ? rightTracks : leftTracks;

  const pos =
    (collapsed ? collapsedPosition : panelPosition) ?? { x: MARGIN, y: MARGIN };
  const active = tracks.find((t) => t.type === activeType) ?? tracks[0];
  const panelHeight = panelHeightFor(tracks.length, showPageSelector);
  const width = collapsed ? SIZE : PANEL_WIDTH;
  const height = collapsed ? SIZE : panelHeight;
  const setPos = collapsed ? setCollapsedPosition : setPanelPosition;

  function expand() {
    setPanelPosition(panelPositionFrom(pos, panelHeight));
    setCollapsed(false);
  }

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
    setPos(clamp({ x: d.originX + dx, y: d.originY + dy }, width, height));
  }

  function onPointerUp(e: React.PointerEvent) {
    const d = drag.current;
    if (!d || d.pointerId !== e.pointerId) return;
    drag.current = null;
    // Nhấn (không kéo) trên nút tròn lúc thu nhỏ -> bung ra tại chỗ.
    if (!d.moved && collapsed) expand();
  }

  if (!hasLeft && !hasRight) return null;

  // z-[55]: trên tooltip (z-50) để panel canh giữa không bị tooltip che, nhưng
  // dưới panel bài giảng (z-[60]) khi cả hai cùng mở.
  return (
    <div
      className="fixed z-[55] touch-none select-none"
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
        // Nền trong suốt: thẻ <audio> của trình duyệt đã có nền riêng của nó,
        // bọc thêm 1 hộp đen nữa là thừa và che mất trang sách. Đổi lại phần
        // chữ/icon phải tự lo tương phản — dùng drop-shadow để vẫn đọc được
        // cả khi nằm trên vùng trắng của trang.
        <div className="overflow-hidden rounded-xl bg-transparent text-white">
          {/* Thanh kéo — chỉ vùng này chịu trách nhiệm drag, để không cấn
              vào thanh trượt/nút play của thẻ audio bên dưới. */}
          <div
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onPointerCancel={onPointerUp}
            className="flex cursor-grab items-center justify-between px-2 py-1.5 drop-shadow-[0_1px_2px_rgb(0_0_0/0.9)] active:cursor-grabbing"
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
            {showPageSelector && (
              <div className="flex justify-center gap-1.5 drop-shadow-[0_1px_2px_rgb(0_0_0/0.9)]">
                {pages.map((p, i) => {
                  const s = i === 0 ? "left" : "right";
                  return (
                    <button
                      key={p}
                      type="button"
                      onClick={() => setSide(s)}
                      className={cn(
                        "rounded-full px-2 py-0.5 text-[11px] transition-colors",
                        side === s ? "bg-primary" : "bg-black/70 hover:bg-black/85"
                      )}
                    >
                      Trang {p}
                    </button>
                  );
                })}
              </div>
            )}
            {tracks.length > 1 && (
              <div className="flex justify-center gap-1.5">
                {tracks.map((t) => (
                  <button
                    key={t.type}
                    type="button"
                    onClick={() => setActiveType(t.type)}
                    className={cn(
                      // Nền panel trong suốt nên các chip phải tự có nền đục,
                      // không dựa vào hộp bọc như trước (bg-white/10 trên nền
                      // trắng của trang sách là mất hút).
                      "rounded-full px-2 py-0.5 text-[11px] transition-colors",
                      t.type === active.type
                        ? "bg-primary"
                        : "bg-black/70 hover:bg-black/85"
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
