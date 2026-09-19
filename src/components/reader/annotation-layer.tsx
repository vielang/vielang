"use client";

import { useCallback, useMemo, useRef, useState } from "react";
import {
  HIGHLIGHTER_OPACITY,
  activeWidth,
  isShapeTool,
  useAnnotationStore,
  type Stroke,
} from "@/lib/annotation-store";
import {
  TEXT_LINE_HEIGHT,
  farEnough,
  penOutlinePath,
  quantize,
  quantizePressure,
  shapePath,
  strokeHit,
  strokePath,
} from "@/lib/annotation-geometry";

/**
 * Bề rộng viewBox. Toạ độ lưu theo tỉ lệ [0,1] (xem `annotation-store`), nhân
 * lên số này khi vẽ để chuỗi `d` là số nguyên cho gọn.
 */
const VIEWBOX_WIDTH = 1000;

/** Khoảng cách tối thiểu giữa 2 điểm được ghi, theo phần chiều rộng trang. */
const MIN_POINT_DISTANCE = 0.0025;

/** Bán kính đầu tẩy, theo phần chiều rộng trang. */
const ERASER_RADIUS = 0.012;

/** Chữ viết trên trang dùng đúng phông của app, không phải phông mặc định của SVG. */
const TEXT_FONT = "var(--font-sans), system-ui, sans-serif";

/**
 * Lớp dấu vẽ tay nằm đè lên ảnh MỘT trang sách: nét bút, bút dạ quang, mũi
 * tên, đường thẳng, khung và chữ chú thích.
 *
 * Phải nằm trong ĐÚNG khung ảnh thật chứ không phải khung chứa — ảnh dùng
 * `object-contain` nên có viền trống hai bên, đặt sai khung là nét vẽ trượt
 * khỏi con chữ. `page-viewer` tự đo khung ảnh rồi mới render lớp này, y như
 * cách nó làm với `TranslationOverlay`.
 *
 * Vẽ bằng SVG chứ không phải <canvas>: lớp này nằm trong phần tử bị
 * `react-zoom-pan-pinch` phóng to, canvas raster sẽ vỡ hạt đúng lúc người ta
 * zoom để soi chữ Hàn nhỏ — mà đó mới là lúc cần khoanh chú thích. Bề rộng
 * nét và cỡ chữ cũng tính theo viewBox nên giữ đúng tỉ lệ trang ở mọi mức zoom.
 *
 * Tắt chế độ vẽ thì lớp vẫn hiện dấu cũ nhưng `pointer-events: none` — chúng
 * thành một phần của trang, không cản chạm/vuốt/bấm vùng dịch.
 */
export function AnnotationLayer({
  bookId,
  page,
  aspectRatio,
  boxWidth,
}: {
  bookId: string;
  page: number;
  aspectRatio: number;
  /** Bề rộng khung ảnh tính bằng px bố cục — chỉ dùng để chọn cỡ chữ cho ô nhập. */
  boxWidth: number;
}) {
  const strokes = useAnnotationStore((s) => s.strokes[`${bookId}:${page}`]);
  const active = useAnnotationStore((s) => s.active);
  const peeking = useAnnotationStore((s) => s.peeking);
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
  const gesture = useRef<{
    pointerId: number;
    points: number[];
    /** Chỉ ghi khi vẽ bằng bút cảm ứng thật — xem `Stroke.pressures`. */
    pressures: number[] | null;
  } | null>(null);

  /** Ô nhập chữ đang mở, toạ độ tỉ lệ của góc trên trái dòng đầu. */
  const [editing, setEditing] = useState<{ x: number; y: number } | null>(null);
  const [draft, setDraft] = useState("");

  // Bút dạ quang phải nằm dưới mọi thứ khác, nếu không tô vàng sau là nuốt
  // mất vòng khoanh hay chữ đã viết trước.
  const [highlights, marks] = useMemo(() => {
    const hl: Stroke[] = [];
    const rest: Stroke[] = [];
    for (const s of strokes ?? []) (s.tool === "highlighter" ? hl : rest).push(s);
    return [hl, rest];
  }, [strokes]);

  /**
   * Chuỗi `d` cho dấu đang vẽ. Bút mực dựng viền ngoài để bề dày đổi theo
   * lực/tốc độ; bút dạ quang kẻ một đường dày cố định — đầu bút dạ quang
   * thật không hề thon lại; mũi tên/đường/khung thì kẻ theo đúng 2 điểm.
   */
  const livePath = useCallback(
    (points: number[], pressures: number[] | null) => {
      if (isShapeTool(tool)) {
        return shapePath(tool, points, VIEWBOX_WIDTH, vbHeight, width * VIEWBOX_WIDTH) ?? "";
      }
      if (tool === "highlighter") return strokePath(points, VIEWBOX_WIDTH, vbHeight);
      return penOutlinePath(
        points,
        pressures ?? undefined,
        VIEWBOX_WIDTH,
        vbHeight,
        width * VIEWBOX_WIDTH
      );
    },
    [tool, vbHeight, width]
  );

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
      // Đọc thẳng từ store: dấu vừa bị xoá ở pointermove trước đó phải biến
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

  /** Chốt ô nhập chữ đang mở. Bỏ trống thì coi như không viết gì. */
  const commitText = useCallback(() => {
    const at = editing;
    setEditing(null);
    setDraft("");
    if (!at || draft.trim() === "") return;
    addStroke(bookId, page, {
      id: crypto.randomUUID(),
      tool: "text",
      color,
      width,
      points: quantize([at.x, at.y]),
      text: draft,
    });
  }, [editing, draft, addStroke, bookId, page, color, width]);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (!active) return;
      // Ngón thứ hai chạm xuống = người dùng định pinch-zoom chứ không vẽ —
      // bỏ dấu đang dở, nhường trọn cử chỉ cho thư viện zoom.
      if (gesture.current) {
        endGesture();
        return;
      }
      const local = toLocal(e);
      if (!local) return;
      // Đang gõ dở mà chạm ra chỗ khác: chốt chữ cũ rồi mới xử lý chạm mới,
      // đúng như mọi công cụ chú thích khác.
      if (editing) commitText();

      // Ở chế độ 2 trang, "hoàn tác"/"khôi phục" trên thanh công cụ nói về
      // trang vừa chạm bút — xem `annotation-store`.
      touchPage(page);

      if (tool === "text") {
        setEditing({ x: local[0], y: local[1] });
        setDraft("");
        return;
      }
      e.currentTarget.setPointerCapture(e.pointerId);

      if (tool === "eraser") {
        gesture.current = { pointerId: e.pointerId, points: [], pressures: null };
        eraseAt(local[0], local[1]);
        return;
      }
      if (isShapeTool(tool)) {
        // Điểm đầu và điểm cuối; kéo tới đâu thì thay điểm cuối tới đó.
        gesture.current = {
          pointerId: e.pointerId,
          points: [local[0], local[1], local[0], local[1]],
          pressures: null,
        };
        return;
      }
      // Chốt một lần ở đầu nét: giữa chừng không đổi từ bút sang ngón được,
      // và `pressures` phải khớp 1-1 với `points` nên không thể ghi nửa chừng.
      const pressures = e.pointerType === "pen" ? [e.pressure] : null;
      gesture.current = { pointerId: e.pointerId, points: local, pressures };
      liveRef.current?.setAttribute("d", livePath(local, pressures));
    },
    [
      active,
      tool,
      toLocal,
      eraseAt,
      endGesture,
      livePath,
      touchPage,
      page,
      editing,
      commitText,
    ]
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
      if (isShapeTool(tool)) {
        g.points[2] = local[0];
        g.points[3] = local[1];
      } else {
        if (!farEnough(g.points, local[0], local[1], MIN_POINT_DISTANCE)) return;
        g.points.push(local[0], local[1]);
        g.pressures?.push(e.pressure);
      }
      liveRef.current?.setAttribute("d", livePath(g.points, g.pressures));
    },
    [tool, toLocal, eraseAt, livePath]
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      const g = gesture.current;
      if (!g || g.pointerId !== e.pointerId) return;
      const { points, pressures } = g;
      endGesture();
      if (tool === "eraser" || tool === "text" || points.length === 0) return;
      // Chạm rồi nhả tại chỗ với công cụ hình: chưa thành hình nào cả, bỏ qua
      // thay vì để lại một chấm vô nghĩa.
      if (isShapeTool(tool) && points[0] === points[2] && points[1] === points[3]) return;

      addStroke(bookId, page, {
        id: crypto.randomUUID(),
        tool,
        color,
        width,
        points: quantize(points),
        // Chỉ bút mực mới có bề dày đổi theo lực — chỗ khác lưu là thừa.
        ...(tool === "pen" && pressures
          ? { pressures: quantizePressure(pressures) }
          : {}),
      });
    },
    [tool, color, width, addStroke, bookId, page, endGesture]
  );

  // Hé xem ảnh gốc: giấu sạch lớp này, kể cả đang bật chế độ vẽ. Không nhận
  // chạm luôn — đang xem trang sạch mà lỡ tay vạch thêm một dấu thì hỏng đúng
  // cái đang muốn xem.
  if (peeking) return null;
  if (!active && highlights.length === 0 && marks.length === 0) return null;

  return (
    <>
      <svg
        ref={svgRef}
        viewBox={`0 0 ${VIEWBOX_WIDTH} ${vbHeight}`}
        preserveAspectRatio="none"
        className="absolute inset-0 h-full w-full"
        style={{
          // Tắt chế độ vẽ thì lớp này trong suốt với chuột/ngón tay: chạm để
          // ẩn thanh công cụ, vuốt lật trang, bấm vùng dịch đều đi xuyên qua.
          pointerEvents: active ? "auto" : "none",
          touchAction: "none",
          cursor: active ? (tool === "text" ? "text" : "crosshair") : undefined,
        }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        {highlights.map((s) => (
          <MarkShape key={s.id} stroke={s} vbHeight={vbHeight} />
        ))}
        {marks.map((s) => (
          <MarkShape key={s.id} stroke={s} vbHeight={vbHeight} />
        ))}

        {tool === "pen" ? (
          // Bút mực: `d` là viền ngoài khép kín nên TÔ, không kẻ. Đổi công cụ
          // là React dựng lại đúng thẻ <path> cần dùng — chỉ xảy ra giữa hai
          // dấu, không bao giờ giữa chừng một dấu.
          <path ref={liveRef} d="" fill={color} stroke="none" />
        ) : (
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
        )}
      </svg>

      {editing && (
        // Ô nhập nằm TRONG khung ảnh nên cũng bị phóng to theo trang — nhờ
        // vậy chữ đang gõ có đúng cỡ với chữ sau khi chốt, không nhảy cỡ.
        <textarea
          autoFocus
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commitText}
          onKeyDown={(e) => {
            e.stopPropagation();
            if (e.key === "Escape") {
              setEditing(null);
              setDraft("");
            }
          }}
          rows={1}
          placeholder="Ghi chú…"
          aria-label="Nội dung chữ chú thích"
          className="absolute z-10 min-w-24 resize-none overflow-hidden rounded border-2 border-dashed border-current bg-white/85 p-0.5 leading-[1.25] outline-none"
          style={{
            left: `${editing.x * 100}%`,
            top: `${editing.y * 100}%`,
            color,
            fontFamily: TEXT_FONT,
            fontSize: width * boxWidth,
            width: `${Math.max(0.08, 1 - editing.x) * 100}%`,
          }}
        />
      )}
    </>
  );
}

function MarkShape({ stroke, vbHeight }: { stroke: Stroke; vbHeight: number }) {
  // Dựng viền bút mực tốn hơn hẳn một chuỗi `d` thường — nhớ lại theo từng
  // dấu, nếu không mỗi lần vẽ thêm là tính lại toàn bộ dấu cũ trên trang.
  const d = useMemo(() => {
    if (stroke.tool === "text") return "";
    if (isShapeTool(stroke.tool)) {
      return (
        shapePath(
          stroke.tool,
          stroke.points,
          VIEWBOX_WIDTH,
          vbHeight,
          stroke.width * VIEWBOX_WIDTH
        ) ?? ""
      );
    }
    if (stroke.tool === "highlighter") {
      return strokePath(stroke.points, VIEWBOX_WIDTH, vbHeight);
    }
    return penOutlinePath(
      stroke.points,
      stroke.pressures,
      VIEWBOX_WIDTH,
      vbHeight,
      stroke.width * VIEWBOX_WIDTH
    );
  }, [stroke, vbHeight]);

  if (stroke.tool === "text") return <MarkText stroke={stroke} vbHeight={vbHeight} />;
  if (stroke.tool === "pen") return <path d={d} fill={stroke.color} stroke="none" />;
  return (
    <path
      d={d}
      fill="none"
      stroke={stroke.color}
      strokeWidth={stroke.width * VIEWBOX_WIDTH}
      strokeOpacity={stroke.tool === "highlighter" ? HIGHLIGHTER_OPACITY : 1}
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}

function MarkText({ stroke, vbHeight }: { stroke: Stroke; vbHeight: number }) {
  const fontSize = stroke.width * VIEWBOX_WIDTH;
  const x = stroke.points[0] * VIEWBOX_WIDTH;
  const y = stroke.points[1] * vbHeight;
  const lines = (stroke.text ?? "").split("\n");
  return (
    <text
      x={x}
      y={y}
      fill={stroke.color}
      fontSize={fontSize}
      // `hanging`: điểm chạm là góc TRÊN trái, khớp với ô nhập lúc gõ — để
      // mặc định (đường chân chữ) thì chữ nhảy lên trên sau khi chốt.
      dominantBaseline="hanging"
      style={{ fontFamily: TEXT_FONT, whiteSpace: "pre" }}
    >
      {lines.map((line, i) => (
        <tspan key={i} x={x} dy={i === 0 ? 0 : fontSize * TEXT_LINE_HEIGHT}>
          {line}
        </tspan>
      ))}
    </text>
  );
}
