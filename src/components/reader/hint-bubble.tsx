"use client";

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

/** Khoảng cách từ điểm bấm xuống đỉnh bong bóng, và lề tối thiểu với mép màn hình. */
const GAP = 12;
const MARGIN = 8;

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
 * Bong bóng chú giải nhỏ, hiện ngay dưới chỗ bấm và KÉO ĐI ĐƯỢC — che mất
 * đúng chỗ đang cần đối chiếu là chuyện thường xảy ra, kéo sang chỗ khác là
 * xong.
 *
 * Dùng chung cho bản dịch đoạn văn và định nghĩa ngữ pháp. Cả hai đều là
 * "chạm vào chấm trên trang, đọc vài dòng, đóng" — tách hai bản riêng thì
 * phần định vị và kéo thả tinh vi ở dưới sẽ trôi khỏi nhau.
 *
 * Cả bong bóng là vùng kéo, không có thanh tiêu đề hay tay cầm riêng: thêm
 * thanh vào thì mất luôn cái gọn gàng. Nút/liên kết bên trong vẫn bấm được —
 * nhấn trúng chúng thì KHÔNG bắt đầu kéo (xem `onPointerDown`). Có nút bên
 * trong thì truyền `role="dialog"` kèm `label`: "tooltip" là thứ chỉ để đọc,
 * trình đọc màn hình sẽ không cho người dùng đi vào các nút đó.
 *
 * Dựng qua portal ra `document.body` chứ KHÔNG render tại chỗ: lớp phủ nằm
 * trong cây đã bị `react-zoom-pan-pinch` gắn `transform`, mà phần tử tổ tiên
 * có `transform` sẽ trở thành gốc toạ độ cho con `position: fixed`. Render
 * tại chỗ thì bong bóng vừa phóng to theo ảnh vừa lệch vị trí khi người dùng
 * zoom/pan.
 */
export function HintBubble({
  children,
  x,
  y,
  onClose,
  ownTriggerSelector,
  role = "tooltip",
  label,
}: {
  children: React.ReactNode;
  x: number;
  y: number;
  onClose: () => void;
  /**
   * Bộ chọn của CHÍNH những cái chấm mở ra bong bóng này.
   *
   * Bấm trúng chúng thì bong bóng không tự đóng, để cái nút đó tự quyết
   * (đóng chấm đang mở, hay chuyển sang chấm khác). Xem lý do đầy đủ ở
   * hiệu ứng bên dưới.
   */
  ownTriggerSelector: string;
  role?: "tooltip" | "dialog";
  /** Tên cho `role="dialog"` (aria-label). */
  label?: string;
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
  }, [x, y, children, dragged]);

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
    // Nhấn trúng nút bên trong thì để nút nhận cú bấm, đừng bắt đầu kéo:
    // `setPointerCapture` sẽ dồn mọi sự kiện về bong bóng, và cú click không
    // bao giờ tới được nút.
    if ((e.target as HTMLElement).closest?.("button, a, input, select, textarea")) return;
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
      if ((e.target as HTMLElement).closest?.(ownTriggerSelector)) return;
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
  }, [onClose, ownTriggerSelector]);

  return createPortal(
    <div
      ref={ref}
      role={role}
      aria-label={label}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      className="fixed z-50 max-w-[min(24rem,calc(100vw-1rem))] cursor-grab touch-none rounded-lg bg-black/75 px-3 py-2 text-sm leading-snug text-white shadow-lg backdrop-blur-sm select-none active:cursor-grabbing"
      style={
        pos
          ? { left: pos.left, top: pos.top }
          : // Lần vẽ đầu chỉ để đo — giấu đi để không thấy nó nhảy vị trí.
            { left: 0, top: 0, visibility: "hidden" }
      }
    >
      {children}
    </div>,
    document.body
  );
}
