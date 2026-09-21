"use client";

import { useState } from "react";
import { BookOpen, X } from "lucide-react";
import type { GrammarPoint } from "@/lib/page-grammar";
import { HintBubble } from "@/components/reader/hint-bubble";

interface ActiveBubble {
  point: GrammarPoint;
  /** Toạ độ điểm bấm, theo hệ toạ độ màn hình (clientX/clientY). */
  x: number;
  y: number;
}

/**
 * Chấm xem nghĩa điểm ngữ pháp, đặt cạnh tiêu đề trên ảnh trang.
 *
 * Sách chú thích ngữ pháp CHỈ bằng tiếng Hàn — dòng kiểu
 * `사람, 사물 이름을 말할 때 사용해요` là một vòng luẩn quẩn: người mới học
 * cần lời giải thích thì lại không đọc nổi chính lời giải thích đó. Chấm này
 * trả lời đúng một câu: "cái đuôi này để làm gì?".
 *
 * CHỈ một định nghĩa ngắn, cố ý. Bản đầu làm cả tấm phủ với bảng chia theo
 * patchim, ví dụ và mục lưu ý — đọc giữa lúc đang học thì quá dài và rối,
 * mà còn che mất trang sách đang xem. Muốn học sâu thì đã có tab bài giảng.
 *
 * Dùng chung `HintBubble` với bản dịch: cùng là "chạm vào chấm trên trang,
 * đọc vài dòng, đóng", nên cùng dáng và cùng cách kéo thả.
 */
export function GrammarOverlay({ points }: { points: GrammarPoint[] }) {
  const [active, setActive] = useState<ActiveBubble | null>(null);

  if (points.length === 0) return null;

  return (
    <>
      {points.map((point) => {
        const [x, y, w, h] = point.rect;
        const isOpen = active?.point.id === point.id;
        return (
          // Khung CHỈ để định vị, không bấm được — xem chú thích cùng kiểu ở
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
            {/* Đặt bên TRÁI tiêu đề, canh giữa theo chiều dọc: tiêu đề ngữ
                pháp luôn nằm sát mép phải trang, để chấm bên phải là rơi ra
                ngoài giấy.

                Vùng chạm 44px nhưng nét vẽ vẫn 20px — phần dôi ra là lề vô
                hình. To cái chấm lên cho dễ bấm thì chọc vào dáng bản in. */}
            <button
              type="button"
              data-grammar-point
              onClick={(e) =>
                setActive(isOpen ? null : { point, x: e.clientX, y: e.clientY })
              }
              aria-label={
                isOpen
                  ? `Đóng nghĩa ngữ pháp: ${point.title}`
                  : `Xem nghĩa ngữ pháp: ${point.title}`
              }
              aria-expanded={isOpen}
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
                {isOpen ? <X className="size-4" /> : <BookOpen className="size-4" />}
              </span>
            </button>
          </div>
        );
      })}

      {active && (
        <HintBubble
          key={`${active.point.id}:${active.x}:${active.y}`}
          x={active.x}
          y={active.y}
          onClose={() => setActive(null)}
          ownTriggerSelector="[data-grammar-point]"
        >
          <span className="font-heading mb-0.5 block font-semibold">
            {active.point.title}
          </span>
          {active.point.vi}
        </HintBubble>
      )}
    </>
  );
}
