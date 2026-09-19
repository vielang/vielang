"use client";

import { useCallback, useMemo, useRef } from "react";
import {
  HIGHLIGHTER_OPACITY,
  activeWidth,
  useAnnotationStore,
  type Stroke,
} from "@/lib/annotation-store";
import { farEnough, quantize, strokeHit, strokePath } from "@/lib/annotation-geometry";

/**
 * Bề rộng viewBox. Toạ độ lưu theo tỉ lệ [0,1] (xem `annotation-store`), nhân
 * lên số này khi vẽ để chuỗi `d` là số nguyên cho gọn.
 */
const VIEWBOX_WIDTH = 1000;

/** Khoảng cách tối thiểu giữa 2 điểm được ghi, theo phần chiều rộng trang. */
const MIN_POINT_DISTANCE = 0.0025;

/** Bán kính đầu tẩy, theo phần chiều rộng trang. */
const ERASER_RADIUS = 0.012;

/**
 * Lớp nét vẽ tay nằm đè lên ảnh MỘT trang sách.
 *
 * Phải nằm trong ĐÚNG khung ảnh thật chứ không phải khung chứa — ảnh dùng
 * `object-contain` nên có viền trống hai bên, đặt sai khung là nét vẽ trượt
 * khỏi con chữ. `page-viewer` tự đo khung ảnh rồi mới render lớp này, y như
 * cách nó làm với `TranslationOverlay`.
 *
 * Vẽ bằng SVG chứ không phải <canvas>: lớp này nằm trong phần tử bị
 * `react-zoom-pan-pinch` phóng to, canvas raster sẽ vỡ hạt đúng lúc người ta
 * zoom để soi chữ Hàn nhỏ — mà đó mới là lúc cần khoanh chú thích. Bề rộng
 * nét cũng tính theo viewBox nên nét dày mỏng đúng tỉ lệ trang ở mọi mức zoom.
 *
 * Tắt chế độ vẽ thì lớp vẫn hiện nét cũ nhưng `pointer-events: none` — nét vẽ
 * thành một phần của trang, không cản chạm/vuốt/bấm vùng dịch.
 */
export function AnnotationLayer({
  bookId,
  page,
  aspectRatio,
}: {
  bookId: string;
  page: number;
  aspectRatio: number;
}) {
  const strokes = useAnnotationStore((s) => s.strokes[`${bookId}:${page}`]);
  const active = useAnnotationStore((s) => s.active);
  const tool = useAnnotationStore((s) => s.tool);
  const size = useAnnotationStore((s) => s.size);
  const penColor = useAnnotationStore((s) => s.penColor);
  const highlighterColor = useAnnotationStore((s) => s.highlighterColor);
  const addStroke = useAnnotationStore((s) => s.addStroke);
  const eraseStrokes = useAnnotationStore((s) => s.eraseStrokes);
  const touchPage = useAnnotationStore((s) => s.touchPage);

  // Khung ảnh luôn đúng tỉ lệ trang (page-viewer tự tính) nên viewBox khớp
  // sẵn với phần tử: tỉ lệ thu phóng của x và y bằng nhau, nét không bị méo.
  const vbHeight = VIEWBOX_WIDTH / aspectRatio;
  const color = tool === "highlighter" ? highlighterColor : penColor;
  const width = activeWidth(tool, size);

  const svgRef = useRef<SVGSVGElement>(null);
  // Nét đang vẽ ghi thẳng vào thuộc tính `d` thay vì qua state: pointermove
  // bắn rất dày, render lại cây React ở mỗi điểm là giật tay.
  const liveRef = useRef<SVGPathElement>(null);
  const gesture = useRef<{ pointerId: number; points: number[] } | null>(null);

  // Bút dạ quang phải nằm dưới bút mực, nếu không tô vàng sau là nuốt mất
  // vòng khoanh vẽ trước.
  const [highlights, pens] = useMemo(() => {
    const hl: Stroke[] = [];
    const pen: Stroke[] = [];
    for (const s of strokes ?? []) (s.tool === "highlighter" ? hl : pen).push(s);
    return [hl, pen];
  }, [strokes]);

  /** clientX/clientY -> toạ độ tỉ lệ trong khung ảnh. `rect` đã tính cả zoom. */
  const toLocal = useCallback((e: React.PointerEvent): [number, number] | null => {
    const rect = svgRef.current?.getBoundingClientRect();
    if (!rect || rect.width === 0 || rect.height === 0) return null;
    return [
      Math.min(Math.max((e.clientX - rect.left) / rect.width, 0), 1),
      Math.min(Math.max((e.clientY - rect.top) / rect.height, 0), 1),
    ];
  }, []);

  const eraseAt = useCallback(
    (x: number, y: number) => {
      // Đọc thẳng từ store: nét vừa bị xoá ở pointermove trước đó phải biến
      // mất ngay, không chờ vòng render kế tiếp cấp lại `strokes`.
      const current = useAnnotationStore.getState().strokes[`${bookId}:${page}`];
      if (!current?.length) return;
      const hit = current.filter((s) => strokeHit(s, x, y, aspectRatio, ERASER_RADIUS));
      if (hit.length > 0) {
        eraseStrokes(
          bookId,
          page,
          hit.map((s) => s.id)
        );
      }
    },
    [bookId, page, aspectRatio, eraseStrokes]
  );

  const endGesture = useCallback(() => {
    gesture.current = null;
    liveRef.current?.setAttribute("d", "");
  }, []);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (!active) return;
      // Ngón thứ hai chạm xuống = người dùng định pinch-zoom chứ không vẽ —
      // bỏ nét đang dở, nhường trọn cử chỉ cho thư viện zoom.
      if (gesture.current) {
        endGesture();
        return;
      }
      const local = toLocal(e);
      if (!local) return;
      e.currentTarget.setPointerCapture(e.pointerId);
      // Ở chế độ 2 trang, "hoàn tác"/"xoá hết" trên thanh công cụ nói về
      // trang vừa chạm bút — xem `annotation-store`.
      touchPage(page);

      if (tool === "eraser") {
        gesture.current = { pointerId: e.pointerId, points: [] };
        eraseAt(local[0], local[1]);
        return;
      }
      gesture.current = { pointerId: e.pointerId, points: local };
      liveRef.current?.setAttribute("d", strokePath(local, VIEWBOX_WIDTH, vbHeight));
    },
    [active, tool, toLocal, eraseAt, endGesture, vbHeight, touchPage, page]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      const g = gesture.current;
      if (!g || g.pointerId !== e.pointerId) return;
      const local = toLocal(e);
      if (!local) return;

      if (tool === "eraser") {
        eraseAt(local[0], local[1]);
        return;
      }
      if (!farEnough(g.points, local[0], local[1], MIN_POINT_DISTANCE)) return;
      g.points.push(local[0], local[1]);
      liveRef.current?.setAttribute("d", strokePath(g.points, VIEWBOX_WIDTH, vbHeight));
    },
    [tool, toLocal, eraseAt, vbHeight]
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      const g = gesture.current;
      if (!g || g.pointerId !== e.pointerId) return;
      const points = g.points;
      endGesture();
      if (tool === "eraser" || points.length === 0) return;

      addStroke(bookId, page, {
        id: crypto.randomUUID(),
        tool,
        color,
        width,
        points: quantize(points),
      });
    },
    [tool, color, width, addStroke, bookId, page, endGesture]
  );

  if (!active && highlights.length === 0 && pens.length === 0) return null;

  return (
    <svg
      ref={svgRef}
      viewBox={`0 0 ${VIEWBOX_WIDTH} ${vbHeight}`}
      preserveAspectRatio="none"
      className="absolute inset-0 h-full w-full"
      style={{
        // Tắt chế độ vẽ thì lớp này trong suốt với chuột/ngón tay: chạm để ẩn
        // thanh công cụ, vuốt lật trang, bấm vùng dịch đều phải đi xuyên qua.
        pointerEvents: active ? "auto" : "none",
        touchAction: "none",
        cursor: active ? "crosshair" : undefined,
      }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      {highlights.map((s) => (
        <StrokePath key={s.id} stroke={s} vbHeight={vbHeight} />
      ))}
      {pens.map((s) => (
        <StrokePath key={s.id} stroke={s} vbHeight={vbHeight} />
      ))}

      <path
        ref={liveRef}
        d=""
        fill="none"
        stroke={color}
        strokeWidth={width * VIEWBOX_WIDTH}
        strokeOpacity={tool === "highlighter" ? HIGHLIGHTER_OPACITY : 1}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function StrokePath({ stroke, vbHeight }: { stroke: Stroke; vbHeight: number }) {
  return (
    <path
      d={strokePath(stroke.points, VIEWBOX_WIDTH, vbHeight)}
      fill="none"
      stroke={stroke.color}
      strokeWidth={stroke.width * VIEWBOX_WIDTH}
      strokeOpacity={stroke.tool === "highlighter" ? HIGHLIGHTER_OPACITY : 1}
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}
