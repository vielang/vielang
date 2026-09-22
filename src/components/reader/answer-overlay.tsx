"use client";

import { useState } from "react";
import { ListChecks, X } from "lucide-react";
import type { AnswerKey } from "@/lib/page-answers";
import { HintBubble } from "@/components/reader/hint-bubble";
import { cn } from "@/lib/utils";
import { gradeKey, useActivityStore, type GradeValue } from "@/lib/activity-store";

/** Ba mức tự chấm — đủ để thấy xu hướng, ít đến mức bấm được trong một giây. */
const GRADES: { value: GradeValue; label: string }[] = [
  { value: 1, label: "Đúng hết" },
  { value: 0.5, label: "Sai vài câu" },
  { value: 0, label: "Sai nhiều" },
];

interface ActiveBubble {
  answerKey: AnswerKey;
  /** Toạ độ điểm bấm, theo hệ toạ độ màn hình (clientX/clientY). */
  x: number;
  y: number;
}

/**
 * Chấm xem đáp án sách, đặt cạnh mục bài tập trên ảnh trang.
 *
 * Đáp án vốn in sẵn ở cuối sách (모범 답안) — chấm này chỉ đỡ cho người học
 * phải lật xuống cuối sách rồi tìm đường quay lại. Bấm vào chấm là một hành
 * động chủ ý, nên đáp án hiện ra luôn, không che mờ thêm lớp nữa.
 *
 * Cùng dáng với chấm dịch và chấm ngữ pháp: khung chỉ để định vị, thứ bấm
 * được là cái chấm 20px với lề chạm 44px — xem lý do đầy đủ ở
 * `translation-overlay`.
 */
export function AnswerOverlay({
  answerKeys,
  bookId,
  page,
}: {
  answerKeys: AnswerKey[];
  /** Sách và trang — để lưu lượt tự chấm đúng chỗ (xem `gradeKey`). */
  bookId: string;
  page: number;
}) {
  const [active, setActive] = useState<ActiveBubble | null>(null);
  const recordAnswerOpened = useActivityStore((s) => s.recordAnswerOpened);
  const grades = useActivityStore((s) => s.grades);
  const setGrade = useActivityStore((s) => s.setGrade);

  if (answerKeys.length === 0) return null;

  return (
    <>
      {answerKeys.map((answerKey) => {
        const [x, y, w, h] = answerKey.rect;
        const isOpen = active?.answerKey.id === answerKey.id;
        return (
          <div
            key={answerKey.id}
            className="pointer-events-none absolute"
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          >
            {/* Canh vào GIỮA vùng, như chấm ngữ pháp. Màu cố định chứ không
                theo theme vì ảnh trang lúc nào cũng là giấy in sáng — xem
                chú thích ở `translation-overlay`. */}
            <button
              type="button"
              data-answer-key
              onClick={(e) => {
                if (!isOpen) recordAnswerOpened();
                setActive(isOpen ? null : { answerKey, x: e.clientX, y: e.clientY });
              }}
              aria-label={
                isOpen
                  ? `Đóng đáp án: ${answerKey.section}`
                  : `Xem đáp án: ${answerKey.section}`
              }
              aria-expanded={isOpen}
              className="group pointer-events-auto absolute top-1/2 left-1/2 flex size-11 -translate-x-1/2 -translate-y-1/2 cursor-help items-center justify-center"
            >
              <span
                className="flex size-5 items-center justify-center text-neutral-900 transition-transform group-hover:scale-110"
                style={{
                  filter:
                    "drop-shadow(0 0 1.5px rgb(255 255 255)) drop-shadow(0 0 1.5px rgb(255 255 255))",
                }}
                aria-hidden
              >
                {isOpen ? <X className="size-4" /> : <ListChecks className="size-4" />}
              </span>
            </button>
          </div>
        );
      })}

      {active && (
        <HintBubble
          key={`${active.answerKey.id}:${active.x}:${active.y}`}
          x={active.x}
          y={active.y}
          onClose={() => setActive(null)}
          ownTriggerSelector="[data-answer-key]"
          role="dialog"
          label={`Đáp án: ${active.answerKey.section}`}
        >
          <span className="font-heading mb-1 block font-semibold">
            Đáp án · {active.answerKey.section}
          </span>
          {/* Số câu và đáp án xếp thành hai cột thẳng hàng, giống bảng đáp án
              in trong sách — dò từ câu hỏi trên trang sang dễ hơn.

              Chặn chiều cao và cho cuộn bên trong: bảng chia động từ của sách
              bài tập có bài dài 15–18 dòng, điện thoại cầm ngang chỉ cao
              ~390px — không chặn thì đuôi bong bóng tràn khỏi màn hình và
              không có cách nào xem. `touch-pan-y` mở lại cử chỉ cuộn mà
              `HintBubble` đã tắt (`touch-none`) để kéo bong bóng. */}
          <span className="grid max-h-[calc(100dvh-8rem)] touch-pan-y grid-cols-[auto_1fr] gap-x-2 gap-y-0.5 overflow-y-auto overscroll-contain">
            {active.answerKey.answers.map((line, i) => (
              <span key={i} className="contents">
                <span className="text-white/70">{line.label}</span>
                <span>{line.text}</span>
              </span>
            ))}
          </span>
          {/* Tự chấm: nguồn dữ liệu năng lực duy nhất phủ được cả tám cuốn —
              app không biết người học viết gì vào sách, chỉ họ biết. Ba mức
              thôi, bấm được ngay trong một giây; bấm lại mức khác thì đè. */}
          <span className="mt-2 flex flex-col gap-1 border-t border-white/25 pt-2">
            {/* Chấm rồi thì nói lại một câu: bấm mà không thấy gì đổi ngoài màu
                nút thì người ta không chắc đã lưu chưa. */}
            <span className="text-xs text-white/70" aria-live="polite">
              {grades[gradeKey(bookId, page, active.answerKey.id)]
                ? "Đã ghi lại — bấm mức khác nếu muốn chấm lại."
                : "Bạn làm thế nào?"}
            </span>
            <span className="flex flex-wrap gap-1" role="group" aria-label="Tự chấm bài này">
              {GRADES.map(({ value, label }) => {
                const key = gradeKey(bookId, page, active.answerKey.id);
                const selected = grades[key]?.grade === value;
                return (
                  <button
                    key={value}
                    type="button"
                    aria-pressed={selected}
                    onClick={() => setGrade(key, value)}
                    className={cn(
                      "rounded-md border px-2 py-1 text-xs transition-colors",
                      selected
                        ? "border-white bg-white text-neutral-900"
                        : "border-white/40 text-white hover:bg-white/15"
                    )}
                  >
                    {label}
                  </button>
                );
              })}
            </span>
          </span>
          {/* Ghi rõ nguồn: đây là đáp án của sách, không phải app tự nghĩ ra,
              và người học muốn thì tự lật tới đúng trang mà đối chiếu. */}
          <span className="mt-2 block border-t border-white/25 pt-1.5 text-xs text-white/70">
            Đáp án sách · tr.{active.answerKey.source}
          </span>
        </HintBubble>
      )}
    </>
  );
}
