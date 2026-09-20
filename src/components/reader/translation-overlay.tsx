"use client";

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Languages } from "lucide-react";
import type { TranslationRegion } from "@/lib/page-translation";

/** Khoảng cách từ điểm bấm xuống đỉnh bong bóng, và lề tối thiểu với mép màn hình. */
const GAP = 12;
const MARGIN = 8;

interface ActiveBubble {
  region: TranslationRegion;
  /** Toạ độ điểm bấm, theo hệ toạ độ màn hình (clientX/clientY). */
  x: number;
  y: number;
}

/**
 * Vùng bấm xem bản dịch, đặt đè lên ảnh trang.
 *
 * Vùng KHÔNG hiển thị gì cả — không viền, không nhãn, không nền. Trang sách
 * phải trông y như bản in; người dùng bấm vào đoạn nào thì bản dịch của đoạn
 * đó hiện ra, thế thôi.
 *
 * Component này PHẢI nằm trong đúng khung ảnh thật (không phải khung chứa):
 * ảnh dùng `object-contain` nên có viền trống hai bên, đặt sai khung là toạ
 * độ lệch hết. Xem `page-viewer.tsx` — nó tự đo khung ảnh rồi mới render
 * component này bên trong.
 *
 * Mỗi vùng gắn `data-translate-region` để `page-viewer` biết cử chỉ bắt đầu
 * trên vùng dịch mà không tính là "chạm để ẩn/hiện thanh công cụ".
 */
export function TranslationOverlay({ regions }: { regions: TranslationRegion[] }) {
  const [active, setActive] = useState<ActiveBubble | null>(null);

  if (regions.length === 0) return null;

  return (
    <>
      {regions.map((region) => {
        const [x, y, w, h] = region.rect;
        return (
          <button
            key={region.id}
            type="button"
            data-translate-region
            onClick={(e) => setActive({ region, x: e.clientX, y: e.clientY })}
            aria-label={`Xem bản dịch: ${region.label ?? "đoạn này"}`}
            className="group absolute cursor-help"
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          >
            {/* Dấu hiệu DUY NHẤT cho biết đoạn này có bản dịch. Vùng bấm vẫn
                trong suốt như cũ để trang giữ nguyên dáng bản in — chỉ một
                chấm nhỏ ở góc, đủ để người ta biết mà chạm vào. Không có nó
                thì cả tính năng này tàng hình. */}
            <span
              className="absolute -top-1.5 -right-1.5 flex size-5 items-center justify-center rounded-full bg-primary/85 text-primary-foreground shadow-sm ring-1 ring-background/70 transition-transform group-hover:scale-110"
              aria-hidden
            >
              <Languages className="size-3" />
            </span>
          </button>
        );
      })}

      {active && (
        // key gồm cả toạ độ bấm: mở lại cùng một vùng ở chỗ khác thì bong bóng
        // dựng mới, xoá vị trí người dùng đã kéo lần trước.
        <TranslationBubble
          key={`${active.region.id}:${active.x}:${active.y}`}
          text={active.region.vi}
          x={active.x}
          y={active.y}
          onClose={() => setActive(null)}
        />
      )}
    </>
  );
}

/** Kéo dưới ngưỡng này thì coi là bấm nhầm, không tính là di chuyển. */
const DRAG_THRESHOLD = 4;

function clampToViewport(
  left: number,
  top: number,
  width: number,
  height: number
): { left: number; top: number } {
  return {
    left: Math.min(Math.max(left, MARGIN), window.innerWidth - width - MARGIN),
    top: Math.min(Math.max(top, MARGIN), window.innerHeight - height - MARGIN),
  };
}

/**
 * Bong bóng bản dịch, hiện ngay dưới chỗ bấm và KÉO ĐI ĐƯỢC — che mất đúng
 * đoạn đang cần đối chiếu là chuyện thường xảy ra, kéo sang chỗ khác là xong.
 *
 * Cả bong bóng là vùng kéo, không có thanh tiêu đề hay tay cầm riêng: thêm
 * thanh vào thì mất luôn cái gọn gàng, mà nội dung ở đây cũng chẳng có gì để
 * bấm bên trong.
 *
 * Dựng qua portal ra `document.body` chứ KHÔNG render tại chỗ: lớp phủ nằm
 * trong cây đã bị `react-zoom-pan-pinch` gắn `transform`, mà phần tử tổ tiên
 * có `transform` sẽ trở thành gốc toạ độ cho con `position: fixed`. Render
 * tại chỗ thì bong bóng vừa phóng to theo ảnh vừa lệch vị trí khi người dùng
 * zoom/pan.
 */
function TranslationBubble({
  text,
  x,
  y,
  onClose,
}: {
  text: string;
  x: number;
  y: number;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const [dragged, setDragged] = useState(false);

  const drag = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    originLeft: number;
    originTop: number;
    moved: boolean;
  } | null>(null);

  // Canh giữa theo điểm bấm rồi kéo vào trong nếu tràn mép; không đủ chỗ phía
  // dưới thì lật lên trên. Đo sau khi render vì chiều cao phụ thuộc độ dài chữ.
  // Bỏ qua khi người dùng đã tự kéo — vị trí họ chọn phải thắng.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el || dragged) return;
    const { width, height } = el.getBoundingClientRect();

    const left = Math.min(
      Math.max(x - width / 2, MARGIN),
      window.innerWidth - width - MARGIN
    );
    const below = y + GAP;
    const top =
      below + height > window.innerHeight - MARGIN
        ? Math.max(MARGIN, y - height - GAP)
        : below;

    setPos({ left, top });
  }, [x, y, text, dragged]);

  /**
   * CHẶN NỔI BỌT ở mọi handler pointer — bắt buộc, không phải cho chắc.
   *
   * Bong bóng dựng qua portal nên nằm ở `document.body`, NHƯNG portal của
   * React vẫn cho sự kiện nổi bọt theo CÂY COMPONENT chứ không theo cây DOM.
   * Không chặn thì `pointerdown`/`pointerup` khi kéo bong bóng vẫn chạy tới
   * handler cử chỉ của `page-viewer`, bị tính là vuốt ngang và lật sang trang
   * kế tiếp ngay giữa lúc người dùng đang kéo.
   */
  function onPointerDown(e: React.PointerEvent) {
    e.stopPropagation();
    const el = ref.current;
    if (!el) return;
    el.setPointerCapture(e.pointerId);
    const rect = el.getBoundingClientRect();
    drag.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      originLeft: rect.left,
      originTop: rect.top,
      moved: false,
    };
  }

  function onPointerMove(e: React.PointerEvent) {
    e.stopPropagation();
    const d = drag.current;
    const el = ref.current;
    if (!d || !el || d.pointerId !== e.pointerId) return;

    const dx = e.clientX - d.startX;
    const dy = e.clientY - d.startY;
    if (!d.moved && Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) {
      return;
    }
    d.moved = true;

    const { width, height } = el.getBoundingClientRect();
    setPos(clampToViewport(d.originLeft + dx, d.originTop + dy, width, height));
    setDragged(true);
  }

  function onPointerUp(e: React.PointerEvent) {
    e.stopPropagation();
    if (drag.current?.pointerId === e.pointerId) drag.current = null;
  }

  // Bấm ra ngoài hoặc Esc thì đóng. Dùng pointerdown ở pha capture để cùng một
  // cú bấm vừa đóng bong bóng hiện tại vừa mở được vùng khác, không phải bấm
  // hai lần.
  useEffect(() => {
    function onPointerDown(e: PointerEvent) {
      if (!ref.current?.contains(e.target as Node)) onClose();
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown, true);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={ref}
      role="tooltip"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      className="fixed z-50 max-w-[min(24rem,calc(100vw-1rem))] cursor-grab touch-none rounded-lg bg-black/75 px-3 py-2 text-sm leading-snug whitespace-pre-line text-white shadow-lg backdrop-blur-sm select-none active:cursor-grabbing"
      style={
        pos
          ? { left: pos.left, top: pos.top }
          : // Lần vẽ đầu chỉ để đo — giấu đi để không thấy nó nhảy vị trí.
            { left: 0, top: 0, visibility: "hidden" }
      }
    >
      {text}
    </div>,
    document.body
  );
}
