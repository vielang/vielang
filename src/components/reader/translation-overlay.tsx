"use client";

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Languages, X } from "lucide-react";
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
        const isOpen = active?.region.id === region.id;
        const what = region.label ?? "đoạn này";
        return (
          <button
            key={region.id}
            type="button"
            data-translate-region
            onClick={(e) =>
              setActive(isOpen ? null : { region, x: e.clientX, y: e.clientY })
            }
            aria-label={isOpen ? `Đóng bản dịch: ${what}` : `Xem bản dịch: ${what}`}
            aria-expanded={isOpen}
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
                thì cả tính năng này tàng hình.

                Đặt NGAY DƯỚI vùng, canh giữa: người ta đọc hết đoạn rồi mới
                cần bản dịch, nên chấm nằm ở chỗ mắt vừa dừng lại. Canh giữa
                thay vì nép vào góc để nó không đụng chữ của cột bên cạnh —
                nhiều trang có hai cột sát nhau.

                `top-full` chứ không phải `-bottom-*`: lệch âm chỉ đẩy chấm
                ra một phần, phần còn lại vẫn nằm đè lên dòng cuối của đoạn.
                Neo mép TRÊN của chấm vào mép DƯỚI của vùng thì nó ra hẳn
                ngoài, không che chữ nào.

                Đang mở thì chấm đổi thành dấu X. Bấm ra ngoài vốn đã đóng
                được (xem hiệu ứng trong TranslationBubble), nhưng không có
                gì nói ra điều đó — đổi icon là cách rẻ nhất để người dùng
                thấy có đường đóng, ngay tại chỗ họ vừa bấm để mở.

                KHÔNG nền, chỉ nét mực đen — để trang giữ được dáng bản in.
                Đổi lại icon phải tự lo tương phản: nó nằm trên đủ thứ nền
                của ảnh scan, có trang là bảng nền đen (vd trang 189) thì nét
                đen trơn biến mất hẳn. Viền sáng quanh nét giải quyết việc đó,
                cùng cách `audio-widget` đang dùng cho chữ của nó.

                Màu cố định chứ KHÔNG dùng token theme: ảnh trang sách lúc
                nào cũng là giấy in sáng, kể cả khi app đang ở chế độ tối —
                dùng `text-foreground` thì chế độ tối sẽ lật nét thành trắng
                và mất hút trên giấy trắng. */}
            <span
              className="absolute top-full left-1/2 mt-1 flex size-5 -translate-x-1/2 items-center justify-center text-neutral-900 transition-transform group-hover:scale-110"
              style={{
                filter:
                  "drop-shadow(0 0 1.5px rgb(255 255 255)) drop-shadow(0 0 1.5px rgb(255 255 255))",
              }}
              aria-hidden
            >
              {isOpen ? <X className="size-4" /> : <Languages className="size-4" />}
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
      if (ref.current?.contains(e.target as Node)) return;
      // Bấm trúng chính chấm dịch thì ĐỪNG đóng ở đây — để cái nút đó tự
      // quyết (đóng vùng đang mở, hay chuyển sang vùng khác).
      //
      // Đóng ở đây là hỏng: pointerdown chạy trước click, nên tới lúc click
      // bắn thì React đã render lại với trạng thái "đang đóng", và nút lại
      // mở đúng vùng vừa đóng. Nhìn ra ngoài thì thành bấm dấu X mà không
      // có gì xảy ra.
      if ((e.target as HTMLElement).closest?.("[data-translate-region]")) return;
      onClose();
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
