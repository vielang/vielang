"use client";

import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { BookOpen, X } from "lucide-react";
import type { GrammarPoint } from "@/lib/page-grammar";
import { NOTE_PROSE_CLASS } from "@/lib/note-store";

/**
 * Chấm xem giải thích ngữ pháp, đặt cạnh tiêu đề điểm ngữ pháp trên ảnh trang.
 *
 * Cùng ý tưởng với `translation-overlay`: khung chỉ để định vị, thứ bấm được
 * là chính cái chấm với lề chạm 44px. Trang sách giữ nguyên dáng bản in.
 *
 * Khác chỗ hiển thị: bản dịch là một đoạn chữ trơn nên nhét vừa bong bóng
 * nhỏ, còn giải thích ngữ pháp có bảng chia theo patchim và nhiều mục — phải
 * là tấm phủ đọc được, cuộn được. Bong bóng 24rem thì bảng vỡ hết.
 */
export function GrammarOverlay({ points }: { points: GrammarPoint[] }) {
  const [active, setActive] = useState<GrammarPoint | null>(null);

  if (points.length === 0) return null;

  return (
    <>
      {points.map((point) => {
        const [x, y, w, h] = point.rect;
        return (
          // Khung KHÔNG bấm được — xem chú thích cùng kiểu ở
          // `translation-overlay`: vùng bấm vô hình to hơn thứ vẽ ra là nói
          // dối người dùng, và nó nuốt mất cử chỉ chạm của trang.
          <div
            key={point.id}
            className="pointer-events-none absolute"
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          >
            {/* Đặt bên TRÁI tiêu đề, canh giữa theo chiều dọc. Tiêu đề ngữ
                pháp luôn nằm sát mép phải trang, nên để chấm bên phải là
                rơi ra ngoài giấy. */}
            <button
              type="button"
              data-grammar-point
              onClick={() => setActive(point)}
              aria-label={`Xem giải thích ngữ pháp: ${point.title}`}
              className="group pointer-events-auto absolute top-1/2 right-full flex size-11 -translate-y-1/2 cursor-help items-center justify-center"
            >
              <span
                className="flex size-5 items-center justify-center text-neutral-900 transition-transform group-hover:scale-110"
                style={{
                  filter:
                    "drop-shadow(0 0 1.5px rgb(255 255 255)) drop-shadow(0 0 1.5px rgb(255 255 255))",
                }}
                aria-hidden
              >
                <BookOpen className="size-4" />
              </span>
            </button>
          </div>
        );
      })}

      {active && <GrammarSheet point={active} onClose={() => setActive(null)} />}
    </>
  );
}

/**
 * Tấm phủ đọc giải thích ngữ pháp.
 *
 * Dựng qua portal ra `document.body`, cùng lý do với bong bóng bản dịch: lớp
 * phủ nằm trong cây đã bị `react-zoom-pan-pinch` gắn `transform`, mà phần tử
 * tổ tiên có `transform` thì trở thành gốc toạ độ cho con `position: fixed`
 * — render tại chỗ là vừa phóng to theo ảnh vừa lệch vị trí khi zoom.
 *
 * Neo đáy màn hình trên điện thoại (dễ với ngón cái) và canh giữa trên màn
 * rộng. Chặn cử chỉ chạm lọt xuống trang bên dưới, nếu không thì cuộn nội
 * dung lại thành vuốt lật trang.
 */
function GrammarSheet({
  point,
  onClose,
}: {
  point: GrammarPoint;
  onClose: () => void;
}) {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-end justify-center sm:items-center"
      onPointerDown={(e) => e.stopPropagation()}
      onPointerUp={(e) => e.stopPropagation()}
    >
      {/* Nền mờ bấm được để đóng — đường thoát mà ai cũng đoán ra. */}
      <button
        type="button"
        aria-label="Đóng giải thích ngữ pháp"
        className="absolute inset-0 bg-black/50"
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        aria-label={`Ngữ pháp: ${point.title}`}
        className="relative flex max-h-[85vh] w-full max-w-xl flex-col rounded-t-xl bg-background shadow-xl sm:rounded-xl"
      >
        <div className="flex items-start justify-between gap-3 border-b border-border px-4 py-3">
          <div className="min-w-0">
            <h2 className="font-heading text-lg leading-tight font-semibold">
              {point.title}
            </h2>
            <p className="mt-0.5 text-sm text-muted-foreground">{point.vi}</p>
            {point.ko && (
              <p className="mt-1 text-xs text-muted-foreground/80 italic">
                {point.ko}
              </p>
            )}
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Đóng"
            className="shrink-0 rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <X className="size-5" aria-hidden />
          </button>
        </div>

        {/* `touch-auto` để cuộn được bên trong, dù trang đọc bên ngoài đang
            chặn cử chỉ chạm. */}
        <div className="touch-auto overflow-y-auto px-4 py-3">
          <div
            className={NOTE_PROSE_CLASS}
            // Nội dung do chính dự án biên soạn rồi dựng sẵn thành HTML lúc
            // build (xem `scripts/build-content.ts`) — không có gì từ người
            // dùng hay từ mạng lọt vào đây.
            dangerouslySetInnerHTML={{ __html: point.html }}
          />
        </div>
      </div>
    </div>,
    document.body
  );
}
