"use client";

import { useCallback, useRef, type RefObject } from "react";

export interface DragPos {
  x: number;
  y: number;
}

/** Lề tối thiểu giữ widget nổi cách mép màn hình. */
export const DRAG_MARGIN = 8;

/** Xê dịch dưới ngưỡng này thì coi là bấm chứ không phải kéo. */
const DRAG_THRESHOLD = 5;

/** Kẹp widget trong màn hình theo đúng kích thước hiện tại của nó. */
export function clampToViewport(pos: DragPos, width: number, height: number): DragPos {
  if (typeof window === "undefined") return pos;
  const maxX = window.innerWidth - width - DRAG_MARGIN;
  const maxY = window.innerHeight - height - DRAG_MARGIN;
  return {
    x: Math.min(Math.max(pos.x, DRAG_MARGIN), Math.max(maxX, DRAG_MARGIN)),
    y: Math.min(Math.max(pos.y, DRAG_MARGIN), Math.max(maxY, DRAG_MARGIN)),
  };
}

/**
 * Kéo thả một widget nổi, đặt tay vào BẤT KỲ ĐÂU trên nó — kể cả ngay trên
 * một cái nút.
 *
 * Widget nổi trong app này (thanh vẽ, bảng ghi âm) gần như toàn là nút, chừa
 * ra mỗi cái tay nắm bé xíu; bắt người dùng nhắm trúng chỗ đó mới kéo được
 * thì coi như không kéo được. Nên bấm và kéo chỉ phân biệt được SAU khi tay
 * nhúc nhích: dưới ngưỡng thì cứ để `click` chạy như thường, vượt ngưỡng mới
 * bắt đầu dời và nuốt cú `click` ở cuối — nếu không, kéo xong là vô tình bấm
 * luôn cái nút vừa đặt ngón lên.
 *
 * Nghe `pointermove`/`pointerup` trên `window` chứ không dùng
 * `setPointerCapture`: bắt con trỏ ngay từ `pointerdown` thì nút không bao
 * giờ nhận được `pointerup` nên `click` không bắn (xem comment trong
 * `audio-widget` — nó đã dính đúng bẫy này), còn bắt muộn thì có lúc con trỏ
 * đã rời khỏi widget và mất luôn các sự kiện ở giữa.
 */
export function useDraggable({
  ref,
  pos,
  setPos,
  onTap,
  onDragStart,
}: {
  ref: RefObject<HTMLElement | null>;
  pos: DragPos | null;
  setPos: (p: DragPos) => void;
  /** Chạm mà KHÔNG kéo — vd bung lại widget đang thu nhỏ. */
  onTap?: () => void;
  /** Bắt đầu kéo thật — vd đóng popover đang mở cho khỏi nhảy theo. */
  onDragStart?: () => void;
}) {
  /** Vừa kéo xong: nuốt cú `click` sắp tới, xem `onClickCapture`. */
  const suppressClick = useRef(false);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      // Cử chỉ mới bắt đầu thì mọi thứ còn treo của cử chỉ trước là rác —
      // xoá đi, bảo đảm cú chạm này luôn tới được cái nút bên dưới.
      suppressClick.current = false;
      const pointerId = e.pointerId;
      const startX = e.clientX;
      const startY = e.clientY;
      const origin = pos ?? { x: DRAG_MARGIN, y: DRAG_MARGIN };
      let moved = false;

      function onMove(ev: PointerEvent) {
        if (ev.pointerId !== pointerId) return;
        const dx = ev.clientX - startX;
        const dy = ev.clientY - startY;
        if (!moved) {
          if (Math.abs(dx) <= DRAG_THRESHOLD && Math.abs(dy) <= DRAG_THRESHOLD) return;
          moved = true;
          onDragStart?.();
        }
        const el = ref.current;
        if (!el) return;
        setPos(
          clampToViewport(
            { x: origin.x + dx, y: origin.y + dy },
            el.offsetWidth,
            el.offsetHeight
          )
        );
      }

      function onUp(ev: PointerEvent) {
        if (ev.pointerId !== pointerId) return;
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
        window.removeEventListener("pointercancel", onUp);
        if (moved) {
          suppressClick.current = true;
          // Chuột kéo xong VẪN sinh ra một cú `click`, còn ngón tay kéo xong
          // thì KHÔNG — trình duyệt chỉ bắn `click` khi coi cú chạm là một
          // cái chạm, không phải một cú vuốt. Không hẹn giờ dọn thì trên
          // điện thoại cờ này nằm lại mãi và nuốt mất lần chạm kế tiếp: đúng
          // cái cảnh "kéo xong phải bấm hai lần nút mới ăn".
          //
          // Hẹn 0ms là đủ và an toàn: `click` đi cùng một lượt xử lý với
          // `pointerup`, nên bộ đếm giờ luôn chạy SAU nó, mà cũng không sống
          // đủ lâu để đụng vào lần chạm sau.
          setTimeout(() => {
            suppressClick.current = false;
          }, 0);
        } else onTap?.();
      }

      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
      window.addEventListener("pointercancel", onUp);
    },
    [pos, setPos, onTap, onDragStart, ref]
  );

  /**
   * Nhả tay sau khi kéo vẫn sinh ra một cú `click` trên cái nút nằm dưới
   * ngón — nuốt nó ở pha bắt.
   */
  const onClickCapture = useCallback((e: React.MouseEvent) => {
    if (!suppressClick.current) return;
    suppressClick.current = false;
    e.preventDefault();
    e.stopPropagation();
  }, []);

  return { onPointerDown, onClickCapture };
}
