"use client";

import { Check, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { isImage, optionLabel, type Exam, type ExamQuestion } from "@/lib/exams";
import { ContentView, OptionsGrid } from "@/components/exam/exam-content";

/**
 * Bốn lựa chọn của một câu, nhãn (A)–(D) như đề in (dữ liệu không ghi nhãn)
 * — bấm thẳng vào lựa chọn để chọn, như tô vào phiếu. Xếp theo số cột của
 * đề (xem `OptionsGrid`).
 *
 * `reveal`: đã chấm — viền xanh + dấu ✓ ở đáp án đúng, viền đỏ + dấu ✗ ở
 * lựa chọn sai của người học. Có dấu chứ không chỉ đổi màu, để người khó
 * phân biệt màu vẫn đọc được.
 */
export function OptionList({
  exam,
  question,
  value,
  onChange,
  reveal,
  disabled,
}: {
  exam: Exam;
  question: ExamQuestion;
  value: number | undefined;
  onChange: (choice: number) => void;
  reveal?: boolean;
  disabled?: boolean;
}) {
  const image = question.options.some(isImage);
  return (
    <div role="radiogroup" aria-label={`Câu ${question.no}`}>
      <OptionsGrid layout={question.layout} count={4} image={image}>
        {question.options.map((opt, i) => {
          const choice = i + 1;
          const selected = value === choice;
          const isAnswer = reveal && question.answer === choice;
          const isWrong = reveal && selected && question.answer !== choice;
          return (
            <button
              key={choice}
              type="button"
              role="radio"
              aria-checked={selected}
              disabled={disabled}
              onClick={() => onChange(choice)}
              className={cn(
                "relative flex items-start gap-2.5 rounded-lg border px-3 py-2.5 text-left transition-colors disabled:cursor-default",
                "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                image && "flex-col items-center",
                isAnswer
                  ? "border-emerald-600/60 bg-emerald-50/80 dark:border-emerald-500/50 dark:bg-emerald-950/30"
                  : isWrong
                    ? "border-red-600/60 bg-red-50/80 dark:border-red-500/50 dark:bg-red-950/30"
                    : selected
                      ? "border-foreground/50 bg-muted"
                      : "border-border enabled:hover:border-foreground/30 enabled:hover:bg-muted/40"
              )}
            >
              <span
                className={cn(
                  "flex h-6 min-w-8 shrink-0 items-center justify-center rounded-md px-1 text-xs font-medium tabular-nums",
                  isAnswer
                    ? "bg-emerald-600 text-white"
                    : isWrong
                      ? "bg-red-600 text-white"
                      : selected
                        ? "bg-foreground text-background"
                        : "border border-foreground/25 text-foreground/70"
                )}
              >
                {optionLabel(choice)}
              </span>
              <ContentView exam={exam} content={opt} alt={`Lựa chọn ${optionLabel(choice)}`} className="min-w-0 text-[0.95rem]" />
              {isAnswer && (
                <Check className="absolute top-1.5 right-1.5 size-4 text-emerald-700 dark:text-emerald-400" aria-label="đáp án đúng" />
              )}
              {isWrong && <X className="absolute top-1.5 right-1.5 size-4 text-red-700 dark:text-red-400" aria-label="chọn sai" />}
            </button>
          );
        })}
      </OptionsGrid>
    </div>
  );
}
