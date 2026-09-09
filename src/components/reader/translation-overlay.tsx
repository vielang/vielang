"use client";

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
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
            className="absolute cursor-help"
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          />
        );
      })}

      {active && (
        <TranslationBubble
          key={active.region.id}
          text={active.region.vi}
          x={active.x}
          y={active.y}
          onClose={() => setActive(null)}
        />
      )}
    </>
  );
}

/**
 * Bong bóng bản dịch, hiện ngay dưới chỗ bấm.
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

  // Canh giữa theo điểm bấm rồi kéo vào trong nếu tràn mép; không đủ chỗ phía
  // dưới thì lật lên trên. Đo sau khi render vì chiều cao phụ thuộc độ dài chữ.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
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
  }, [x, y, text]);

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
      className="fixed z-50 max-w-[min(24rem,calc(100vw-1rem))] rounded-lg bg-black/75 px-3 py-2 text-sm leading-snug whitespace-pre-line text-white shadow-lg backdrop-blur-sm"
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
