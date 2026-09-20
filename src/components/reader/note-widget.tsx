"use client";

import { useEffect, useRef } from "react";
import { ExternalLink, GripVertical, Maximize2, X } from "lucide-react";
import { NotePanel } from "@/components/reader/note-panel";
import { openNoteWindow } from "@/lib/note-window";
import {
  useNoteWidgetStore,
  type Position,
  type Size,
} from "@/lib/note-widget-store";
import { notePanelAnchor, notePanelSize } from "@/lib/widget-dock";

const HEADER_HEIGHT = 36;
const MIN_WIDTH = 260;
const MIN_HEIGHT = 220;
const MARGIN = 10;

/**
 * Phần bắt buộc phải còn nhìn thấy khi kéo panel ra khỏi màn hình. Cố ý cho
 * panel thò ra ngoài mép (khác `audio-widget` vốn giữ nguyên trong màn) —
 * đây chính là cách "cất" panel để lấy lại chỗ đọc sách mà vẫn nắm được
 * thanh tiêu đề kéo vào lại.
 */
const MIN_VISIBLE = 64;

function defaultSize(): Size {
  if (typeof window === "undefined") return { width: MIN_WIDTH, height: MIN_HEIGHT };
  return notePanelSize();
}

function defaultPosition(): Position {
  if (typeof window === "undefined") return { x: MARGIN, y: MARGIN };
  return notePanelAnchor();
}

function clampPosition(pos: Position, size: Size): Position {
  if (typeof window === "undefined") return pos;
  return {
    // Cho phép âm / vượt quá bề rộng: panel thò ra ngoài mép trái-phải, chỉ
    // giữ lại MIN_VISIBLE px để còn nắm kéo vào.
    x: Math.min(
      Math.max(pos.x, MIN_VISIBLE - size.width),
      window.innerWidth - MIN_VISIBLE
    ),
    // Mép trên không cho âm (mất luôn thanh kéo thì không lôi lại được),
    // mép dưới thì cho thò xuống.
    y: Math.min(Math.max(pos.y, 0), window.innerHeight - HEADER_HEIGHT),
  };
}

function clampSize(size: Size): Size {
  if (typeof window === "undefined") return size;
  return {
    width: Math.min(Math.max(size.width, MIN_WIDTH), window.innerWidth),
    height: Math.min(Math.max(size.height, MIN_HEIGHT), window.innerHeight),
  };
}

/**
 * Panel bài giảng nổi trên trang sách — kéo thả, đổi cỡ, và bật ra cửa sổ
 * trình duyệt riêng được. Vị trí/kích thước giữ nguyên khi lật trang (xem
 * lib/note-widget-store.ts).
 */
export function NoteWidget({
  bookId,
  pages,
  noteContentByPage,
}: {
  bookId: string;
  /** 1 hoặc 2 trang tuỳ chế độ xem — xem NotePanel. */
  pages: number[];
  noteContentByPage: Record<number, string | null>;
}) {
  const mode = useNoteWidgetStore((s) => s.mode);
  const setMode = useNoteWidgetStore((s) => s.setMode);
  const storedPosition = useNoteWidgetStore((s) => s.position);
  const setPosition = useNoteWidgetStore((s) => s.setPosition);
  const storedSize = useNoteWidgetStore((s) => s.size);
  const setSize = useNoteWidgetStore((s) => s.setSize);

  const drag = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    origin: Position | Size;
  } | null>(null);

  // Chốt vị trí/kích thước mặc định (phụ thuộc kích thước màn hình, chỉ biết
  // được sau khi mount) vào store ở lần mở đầu tiên, để các lần remount sau
  // dùng chung 1 giá trị ổn định. Cùng cách làm với audio-widget.
  useEffect(() => {
    if (storedSize && storedPosition) return;
    const size = storedSize ?? defaultSize();
    if (!storedSize) setSize(size);
    if (!storedPosition) setPosition(defaultPosition());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (mode !== "floating" || !storedPosition || !storedSize) return null;

  const pos = storedPosition;
  const size = storedSize;

  function onDragDown(e: React.PointerEvent) {
    // Bấm vào nút NẰM TRONG thanh tiêu đề thì không bắt đầu kéo. Nếu vẫn
    // setPointerCapture ở đây, mọi pointer event sau đó bị chuyển hết về
    // thanh tiêu đề — nút không bao giờ nhận `pointerup` nên `click` không
    // bao giờ bắn, và các nút mở cửa sổ/phóng to/đóng thành vô tác dụng.
    if ((e.target as HTMLElement).closest("button")) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      origin: pos,
    };
  }

  function onDragMove(e: React.PointerEvent) {
    const d = drag.current;
    if (!d || d.pointerId !== e.pointerId) return;
    const origin = d.origin as Position;
    setPosition(
      clampPosition(
        { x: origin.x + (e.clientX - d.startX), y: origin.y + (e.clientY - d.startY) },
        size
      )
    );
  }

  function onResizeDown(e: React.PointerEvent) {
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    drag.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      origin: size,
    };
  }

  function onResizeMove(e: React.PointerEvent) {
    const d = drag.current;
    if (!d || d.pointerId !== e.pointerId) return;
    const origin = d.origin as Size;
    setSize(
      clampSize({
        width: origin.width + (e.clientX - d.startX),
        height: origin.height + (e.clientY - d.startY),
      })
    );
  }

  function onPointerUp(e: React.PointerEvent) {
    if (drag.current?.pointerId === e.pointerId) drag.current = null;
  }

  // z-[60]: phải nằm trên cả tooltip (z-50). Tooltip của nút bài giảng trên
  // thanh công cụ hiện ra đúng chỗ panel mọc lên, để thấp hơn là nó vừa che
  // vừa chặn click xuống thanh tiêu đề panel.
  return (
    <div
      className="fixed z-[60] flex flex-col overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-2xl"
      style={{ left: pos.x, top: pos.y, width: size.width, height: size.height }}
    >
      {/* Thanh kéo — chỉ vùng này chịu trách nhiệm drag, để không cấn vào
          thanh cuộn / toolbar của editor bên dưới. */}
      <div
        onPointerDown={onDragDown}
        onPointerMove={onDragMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        className="flex shrink-0 cursor-grab touch-none select-none items-center gap-1 border-b border-border bg-muted/60 px-2 py-1.5 active:cursor-grabbing"
        style={{ height: HEADER_HEIGHT }}
      >
        <GripVertical className="size-4 shrink-0 text-muted-foreground" aria-hidden />
        <span className="min-w-0 flex-1 truncate text-xs font-medium">
          Bài giảng — Trang {pages.length === 2 ? `${pages[0]}–${pages[1]}` : pages[0]}
        </span>
        <button
          type="button"
          onClick={() => {
            // Cửa sổ riêng chỉ xem 1 trang (khung hẹp, không hợp 2 trang) —
            // luôn mở trang trái/anchor.
            if (openNoteWindow(bookId, pages[0])) setMode("popped");
          }}
          aria-label="Mở ra cửa sổ riêng"
          title="Mở ra cửa sổ riêng"
          className="rounded p-1 hover:bg-muted"
        >
          <ExternalLink className="size-3.5" aria-hidden />
        </button>
        <button
          type="button"
          onClick={() => setMode("fullscreen")}
          aria-label="Phóng to toàn màn hình"
          title="Phóng to toàn màn hình"
          className="rounded p-1 hover:bg-muted"
        >
          <Maximize2 className="size-3.5" aria-hidden />
        </button>
        <button
          type="button"
          onClick={() => setMode("closed")}
          aria-label="Đóng bài giảng"
          className="rounded p-1 hover:bg-muted"
        >
          <X className="size-3.5" aria-hidden />
        </button>
      </div>

      <NotePanel bookId={bookId} pages={pages} noteContentByPage={noteContentByPage} />

      {/* Tay cầm đổi cỡ ở góc dưới phải. */}
      <div
        onPointerDown={onResizeDown}
        onPointerMove={onResizeMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        role="separator"
        aria-label="Đổi kích thước bài giảng"
        className="absolute right-0 bottom-0 size-4 cursor-nwse-resize touch-none bg-[linear-gradient(135deg,transparent_50%,var(--color-border)_50%)]"
      />
    </div>
  );
}
