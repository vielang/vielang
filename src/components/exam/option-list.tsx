"use client";

import { Check, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { isImage, type Exam, type ExamQuestion } from "@/lib/exams";
import { ContentView, OptionsGrid } from "@/components/exam/exam-content";

const MARKS = ["①", "②", "③", "④"];

/**
 * Bốn lựa chọn của một câu — bấm thẳng vào lựa chọn để chọn, như tô vào
 * phiếu. Xếp theo số cột của đề gốc (xem `OptionsGrid`).
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
                "relative flex items-start gap-2 rounded-lg border-2 px-3 py-2.5 text-left transition-colors disabled:cursor-default",
                image && "flex-col items-center",
                isAnswer
                  ? "border-emerald-600 bg-emerald-50 dark:bg-emerald-950/40"
                  : isWrong
                    ? "border-red-600 bg-red-50 dark:bg-red-950/40"
                    : selected
                      ? "border-primary bg-primary/5"
                      : "border-border hover:border-foreground/40 hover:bg-muted/50"
              )}
            >
              <span
                className={cn(
                  "flex size-6 shrink-0 items-center justify-center rounded-full text-sm",
                  selected || isAnswer ? "bg-foreground text-background" : "text-foreground/70"
                )}
                aria-hidden
              >
                {MARKS[i]}
              </span>
              <ContentView exam={exam} content={opt} alt={`Lựa chọn ${choice}`} className="min-w-0 text-[0.95rem]" />
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
