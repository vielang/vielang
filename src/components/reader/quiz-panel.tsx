"use client";

import { useMemo, useState } from "react";
import { Check, Lightbulb, Puzzle, RotateCcw, Undo2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import {
  countItems,
  isCorrect,
  isGradable,
  shuffleWithSeed,
  splitAnswerChunks,
  type QuizItem,
  type QuizSection,
} from "@/lib/quiz";
import { usePageAnswers, useQuizStore } from "@/lib/quiz-store";

/**
 * Chỗ nộp đáp án cho BÀI TẬP IN TRONG SÁCH của trang đang mở — người dùng
 * nhìn đề trên ảnh trang, làm ở đây. Cấu trúc bám theo sách: mỗi
 * `QuizSection` là 1 mục thật ("연습 1", "읽기", "쓰기"...).
 *
 * Chấm TỪNG CÂU ngay khi bấm, không gom lại thành "nộp bài": người tự học thì
 * phản hồi ngay dạy được nhiều hơn — sai câu 1 thì hiểu và sửa trước khi làm
 * câu 2. Dòng tổng kết ở đầu vẫn cho cảm giác hoàn thành.
 *
 * Bài mở (말하기/쓰기) chỉ LƯU chứ không chấm — sách không có đáp án đúng duy
 * nhất, chấm bừa thì tệ hơn là không chấm.
 */
export function QuizPanel({
  bookId,
  page,
  sections,
}: {
  bookId: string;
  page: number;
  sections: QuizSection[];
}) {
  const { answers, checked } = usePageAnswers(bookId, page);
  const setAnswer = useQuizStore((s) => s.setAnswer);
  const markChecked = useQuizStore((s) => s.markChecked);
  const resetPage = useQuizStore((s) => s.resetPage);

  const gradable = sections.flatMap((s) => s.items.filter(isGradable));
  const done = gradable.filter((i) => checked.includes(i.id));
  const correct = done.filter((i) => isCorrect(i, answers[i.id])).length;
  const touched = done.length > 0 || Object.keys(answers).length > 0;

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-6">
      <div className="mb-3 flex items-center justify-between gap-2 text-xs text-muted-foreground">
        <span className="tabular-nums">
          {gradable.length > 0
            ? `Đã chấm ${done.length}/${gradable.length}` +
              (done.length > 0 ? ` · đúng ${correct}` : "")
            : `${countItems(sections)} bài tập`}
        </span>
        {touched && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7"
            onClick={() => resetPage(bookId, page)}
          >
            <RotateCcw className="size-3.5" aria-hidden /> Làm lại
          </Button>
        )}
      </div>

      <div className="flex flex-col gap-6">
        {sections.map((section) => (
          <section key={section.title} className="flex flex-col gap-3">
            <div>
              <h3 className="text-sm font-semibold">{section.title}</h3>
              {section.instruction && (
                <p className="mt-0.5 text-xs text-muted-foreground">
                  {section.instruction}
                </p>
              )}
            </div>

            {section.items.map((item) => (
              <QuizItemView
                key={item.id}
                item={item}
                value={answers[item.id]}
                checked={checked.includes(item.id)}
                onAnswer={(v) => setAnswer(bookId, page, item.id, v)}
                onCheck={() => markChecked(bookId, page, item.id)}
              />
            ))}
          </section>
        ))}
      </div>
    </div>
  );
}

function QuizItemView({
  item,
  value,
  checked,
  onAnswer,
  onCheck,
}: {
  item: QuizItem;
  value: string | number | undefined;
  checked: boolean;
  onAnswer: (value: string | number) => void;
  onCheck: () => void;
}) {
  const [showModel, setShowModel] = useState(false);
  const answered = value !== undefined && value !== "";
  const correct = checked && isCorrect(item, value as string | number);

  // Gợi ý ghép câu: cắt đáp án chuẩn thành các mảnh rồi xáo. Chỉ có nghĩa với
  // câu điền có từ 2 mảnh trở lên — 1 mảnh thì bấm gợi ý là ra thẳng đáp án.
  const [hintOpen, setHintOpen] = useState(false);
  const [usedChunks, setUsedChunks] = useState<number[]>([]);
  const chunks = useMemo(
    () =>
      item.kind === "fill"
        ? shuffleWithSeed(splitAnswerChunks(item.answers[0]), item.id)
        : [],
    [item]
  );
  const canHint = chunks.length >= 2 && !checked;

  function appendChunk(index: number) {
    const current = typeof value === "string" ? value : "";
    onAnswer(current ? `${current} ${chunks[index]}` : chunks[index]);
    setUsedChunks((used) => [...used, index]);
  }

  function undoChunk() {
    const used = usedChunks.slice(0, -1);
    setUsedChunks(used);
    onAnswer(used.map((i) => chunks[i]).join(" "));
  }

  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm">
        {item.label && (
          <span className="mr-1.5 font-medium text-muted-foreground">{item.label}</span>
        )}
        {item.prompt}
      </p>

      {item.kind === "choice" && (
        <div className="flex flex-col gap-1.5">
          {item.options.map((option, i) => {
            const selected = value === i;
            // Chỉ tô màu sau khi chấm, và chỉ tô phương án ĐÃ CHỌN cùng
            // phương án đúng — tô hết cả 4 thì hoá ra cho sẵn đáp án.
            const asCorrect = checked && i === item.answer;
            const asWrong = checked && selected && !correct;
            return (
              <button
                key={i}
                type="button"
                onClick={() => onAnswer(i)}
                aria-pressed={selected}
                className={cn(
                  "flex items-center gap-2 rounded-lg border px-3 py-2 text-left text-sm transition-colors",
                  asCorrect
                    ? "border-emerald-500 bg-emerald-500/10"
                    : asWrong
                      ? "border-destructive bg-destructive/10"
                      : selected
                        ? "border-primary bg-primary/10"
                        : "border-border hover:bg-muted"
                )}
              >
                <span
                  className={cn(
                    "flex size-5 shrink-0 items-center justify-center rounded-full border text-[11px] tabular-nums",
                    selected ? "border-current" : "border-border text-muted-foreground"
                  )}
                >
                  {i + 1}
                </span>
                {option}
              </button>
            );
          })}
        </div>
      )}

      {item.kind === "fill" && (
        <Input
          value={typeof value === "string" ? value : ""}
          onChange={(e) => onAnswer(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && answered) onCheck();
          }}
          placeholder="Nhập đáp án…"
          aria-label={item.prompt}
          className={cn(
            checked && (correct ? "border-emerald-500" : "border-destructive")
          )}
        />
      )}

      {item.kind === "free" &&
        (item.multiline ? (
          <textarea
            value={typeof value === "string" ? value : ""}
            onChange={(e) => onAnswer(e.target.value)}
            rows={4}
            placeholder="Viết câu trả lời của bạn…"
            aria-label={item.prompt}
            className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
          />
        ) : (
          <Input
            value={typeof value === "string" ? value : ""}
            onChange={(e) => onAnswer(e.target.value)}
            placeholder="Viết câu trả lời của bạn…"
            aria-label={item.prompt}
          />
        ))}

      {hintOpen && chunks.length > 0 && (
        <div className="flex flex-col gap-2 rounded-lg border border-dashed border-border bg-muted/40 p-2">
          <p className="text-[11px] text-muted-foreground">
            Bấm các mảnh theo đúng thứ tự để ghép thành câu trả lời.
          </p>
          <div className="flex flex-wrap gap-1.5">
            {chunks.map((chunk, i) => {
              const used = usedChunks.includes(i);
              return (
                <button
                  key={`${chunk}-${i}`}
                  type="button"
                  disabled={used}
                  onClick={() => appendChunk(i)}
                  className={cn(
                    "rounded-md border px-2 py-1 text-sm transition-colors",
                    used
                      ? "border-dashed border-border text-muted-foreground/40"
                      : "border-border bg-background hover:bg-muted"
                  )}
                >
                  {chunk}
                </button>
              );
            })}
          </div>
          {usedChunks.length > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 self-start"
              onClick={undoChunk}
            >
              <Undo2 className="size-3.5" aria-hidden /> Bỏ mảnh vừa chọn
            </Button>
          )}
        </div>
      )}

      <div className="flex items-center gap-2">
        {isGradable(item) && !checked && (
          <Button size="sm" className="h-7" disabled={!answered} onClick={onCheck}>
            Kiểm tra
          </Button>
        )}
        {canHint && (
          <Button
            variant="outline"
            size="sm"
            className="h-7"
            onClick={() => setHintOpen((v) => !v)}
          >
            <Puzzle className="size-3.5" aria-hidden />
            {hintOpen ? "Ẩn gợi ý" : "Gợi ý"}
          </Button>
        )}
        {isGradable(item) && checked && (
          <span
            className={cn(
              "flex items-center gap-1 text-xs font-medium",
              correct ? "text-emerald-600 dark:text-emerald-400" : "text-destructive"
            )}
          >
            {correct ? (
              <>
                <Check className="size-3.5" aria-hidden /> Đúng
              </>
            ) : (
              <>
                <X className="size-3.5" aria-hidden /> Chưa đúng
                {item.kind === "fill" && ` — đáp án: ${item.answers[0]}`}
              </>
            )}
          </span>
        )}
        {item.kind === "free" && item.model && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7"
            onClick={() => setShowModel((v) => !v)}
          >
            <Lightbulb className="size-3.5" aria-hidden />
            {showModel ? "Ẩn câu mẫu" : "Xem câu mẫu"}
          </Button>
        )}
      </div>

      {item.kind === "free" && item.model && showModel && (
        <p className="rounded-lg bg-muted/60 px-3 py-2 text-xs">
          <span className="text-muted-foreground">Câu mẫu: </span>
          {item.model}
        </p>
      )}

      {checked && item.explain && (
        <p className="rounded-lg bg-muted/60 px-3 py-2 text-xs text-muted-foreground">
          {item.explain}
        </p>
      )}
    </div>
  );
}
