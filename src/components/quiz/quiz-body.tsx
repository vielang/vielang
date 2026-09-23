"use client";

import { useMemo, useState } from "react";
import { Check, Lightbulb, Puzzle, RotateCcw, X } from "lucide-react";
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
import { useQuizAnswers, useQuizStore } from "@/lib/quiz-store";
import { useActivityStore } from "@/lib/activity-store";

/** Đề dạng "______" thì không cần in ra — ô nhập bên dưới CHÍNH LÀ chỗ trống. */
const BLANK_ONLY = /^[_\s]+$/;

/**
 * Phần THÂN của một bài tập: dòng tổng kết, các mục, các câu và việc chấm.
 *
 * Tách khỏi `reader/quiz-panel` để bài học IT dùng lại y nguyên cách chấm và
 * cách lưu, chỉ khác cái khung bọc ngoài. Nơi gọi tự đặt `quizId` (trang sách
 * hay bài học) — ở đây nó chỉ là một chuỗi.
 */
export function QuizBody({
  quizId,
  sections,
  numbered = false,
}: {
  quizId: string;
  sections: QuizSection[];
  /**
   * Đánh số "Câu 1, 2, 3…" và gắn nhãn A/B/C/D cho từng phương án.
   *
   * Bật ở bài học IT, TẮT ở bài tập trong sách: đề in trên sách đã có số và
   * ký hiệu riêng (`item.label` giữ đúng "①", "1)" như bản in), thêm một hệ
   * đếm nữa thì người học không biết nhìn theo cái nào.
   */
  numbered?: boolean;
}) {
  const { answers, checked } = useQuizAnswers(quizId);
  const setAnswer = useQuizStore((s) => s.setAnswer);
  const markChecked = useQuizStore((s) => s.markChecked);
  const resetQuiz = useQuizStore((s) => s.resetQuiz);
  const recordQuizCheck = useActivityStore((s) => s.recordQuizCheck);

  const gradable = sections.flatMap((s) => s.items.filter(isGradable));
  const done = gradable.filter((i) => checked.includes(i.id));
  const correct = done.filter((i) => isCorrect(i, answers[i.id])).length;
  const touched = done.length > 0 || Object.keys(answers).length > 0;
  const xong = gradable.length > 0 && done.length === gradable.length;

  // Đánh số câu chạy XUYÊN các mục: người học đọc "Câu 3" là biết ngay câu nào,
  // không phải đếm lại từ đầu mục. Chỉ làm ở bài học — bài tập trong sách đã
  // có số in sẵn trên giấy (`item.label`), thêm số nữa là hai hệ đếm chồng nhau.
  let dem = 0;
  const so = new Map<string, number>();
  if (numbered) {
    for (const s of sections) for (const i of s.items) so.set(i.id, ++dem);
  }

  return (
    <>
      <div
        className={cn(
          "flex items-center justify-between gap-2 text-xs text-muted-foreground",
          numbered ? "mb-3" : "mb-4 h-6"
        )}
      >
        {/* Kiểu bài học: tiêu đề mục đứng cùng hàng với bộ đếm, khỏi tốn thêm
            một dòng cho mỗi thứ. */}
        {numbered ? (
          <h3 className="font-heading text-base font-semibold text-foreground">
            {sections[0]?.title}
          </h3>
        ) : null}
        <span className="tabular-nums">
          {gradable.length > 0
            ? `${done.length}/${gradable.length} câu` +
              (done.length > 0 ? ` · đúng ${correct}` : "")
            : `${countItems(sections)} bài tập`}
        </span>
        {touched && (
          <Button
            variant="ghost"
            size="icon-sm"
            className={cn(numbered && "-my-1 -mr-2")}
            onClick={() => resetQuiz(quizId)}
            aria-label="Làm lại từ đầu"
            title="Làm lại từ đầu"
          >
            <RotateCcw className="size-3.5" aria-hidden />
          </Button>
        )}
      </div>

      {/* Thanh tiến độ chỉ có nghĩa khi đủ vài câu; panel bên sách thường chỉ
          một hai câu nên để nguyên như cũ. */}
      {numbered && gradable.length > 1 && (
        <div className="mb-5 h-0.5 overflow-hidden rounded-full bg-muted">
          <div
            className={cn(
              "h-full rounded-full transition-all duration-300",
              xong ? "bg-emerald-500" : "bg-foreground/70"
            )}
            style={{ width: `${(done.length / gradable.length) * 100}%` }}
          />
        </div>
      )}

      <div className={cn("flex flex-col", numbered ? "gap-6" : "gap-7")}>
        {sections.map((section) => (
          <section key={section.title} className={cn("flex flex-col", numbered ? "gap-5" : "gap-4")}>
            {/* Kiểu bài học đã in tiêu đề mục ở hàng trên cùng rồi. */}
            {!numbered && (
              <div className="flex flex-col gap-1">
                <h3 className="text-xs font-semibold tracking-wide text-foreground/70 uppercase">
                  {section.title}
                </h3>
                {section.instruction && (
                  <p className="text-xs leading-relaxed text-muted-foreground">
                    {section.instruction}
                  </p>
                )}
              </div>
            )}
            {numbered && section.instruction && (
              <p className="text-xs leading-relaxed text-muted-foreground">
                {section.instruction}
              </p>
            )}

            {section.items.map((item) => (
              <QuizItemView
                key={item.id}
                item={item}
                stt={so.get(item.id)}
                value={answers[item.id]}
                checked={checked.includes(item.id)}
                onAnswer={(v) => setAnswer(quizId, item.id, v)}
                onCheck={() => {
                  markChecked(quizId, item.id);
                  // Lịch sử học cho My page — chỉ câu chấm được mới có đúng/sai.
                  if (isGradable(item)) recordQuizCheck(isCorrect(item, answers[item.id]));
                }}
              />
            ))}
          </section>
        ))}
      </div>

      {numbered && xong && (
        <p className="mt-5 rounded-lg bg-muted/60 px-3 py-2 text-sm">
          Xong rồi: <strong className="tabular-nums">đúng {correct}/{gradable.length}</strong>
          {correct < gradable.length && " — đọc lại phần giải thích của câu sai rồi thử lại nhé."}
        </p>
      )}
    </>
  );
}

/** A, B, C… cho phương án thứ 0, 1, 2… */
const chuCai = (i: number) => String.fromCharCode(65 + i);

function QuizItemView({
  item,
  stt,
  value,
  checked,
  onAnswer,
  onCheck,
}: {
  item: QuizItem;
  /** Số thứ tự câu; có thì vẽ kiểu bài học (thẻ riêng, nhãn A/B/C/D). */
  stt?: number;
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
  const [picked, setPicked] = useState<number[]>([]);
  const chunks = useMemo(
    () =>
      item.kind === "fill"
        ? shuffleWithSeed(splitAnswerChunks(item.answers[0]), item.id)
        : [],
    [item]
  );
  const canHint = chunks.length >= 2 && !checked;

  /** Bấm mảnh chưa dùng thì thêm vào cuối, bấm mảnh đã dùng thì gỡ ra. Nhờ vậy
   *  không cần nút hoàn tác riêng — ghép sai thì bấm lại đúng mảnh đó. */
  function toggleChunk(index: number) {
    const next = picked.includes(index)
      ? picked.filter((i) => i !== index)
      : [...picked, index];
    setPicked(next);
    onAnswer(next.map((i) => chunks[i]).join(" "));
  }

  const showPrompt = !BLANK_ONLY.test(item.prompt);

  return (
    <div
      className={cn(
        "flex flex-col gap-2",
        // Kiểu bài học: nhãn "CÂU n" và khoảng trống là đủ để tách câu —
        // bọc thêm khung nữa thì thành ba lớp viền lồng nhau (khung ngoài,
        // thẻ câu, ô phương án) và tốn diện tích vô ích.
        stt !== undefined && "gap-2.5"
      )}
    >
      {stt !== undefined && (
        <div className="flex items-baseline justify-between gap-2">
          <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase tabular-nums">
            Câu {stt}
          </span>
          {isGradable(item) && checked && (
            <span
              className={cn(
                "inline-flex items-center gap-1 text-xs font-medium",
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
                </>
              )}
            </span>
          )}
        </div>
      )}

      {(item.label || showPrompt) && (
        <p className={cn("leading-relaxed", stt !== undefined ? "text-sm font-medium" : "text-sm")}>
          {item.label && (
            <span className="text-muted-foreground">{item.label} </span>
          )}
          {showPrompt && item.prompt}
        </p>
      )}

      {item.code && (
        <pre className="overflow-x-auto rounded-lg border border-border bg-muted/60 px-3 py-2 text-xs leading-relaxed">
          <code>{item.code}</code>
        </pre>
      )}

      {item.kind === "choice" && (
        <div className={cn("flex flex-col", stt !== undefined ? "gap-1.5" : "gap-1")}>
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
                  "rounded-md border text-left text-sm transition-colors",
                  stt !== undefined
                    ? "flex items-start gap-2.5 px-2.5 py-2"
                    : "px-3 py-1.5",
                  asCorrect
                    ? "border-emerald-500/70 bg-emerald-500/10"
                    : asWrong
                      ? "border-destructive/70 bg-destructive/10"
                      : selected
                        ? "border-primary bg-primary/10"
                        : "border-transparent bg-muted/50 hover:bg-muted"
                )}
              >
                {stt !== undefined && (
                  <span
                    className={cn(
                      "mt-px flex size-5 shrink-0 items-center justify-center rounded text-[0.7rem] font-semibold",
                      asCorrect
                        ? "bg-emerald-500 text-white"
                        : asWrong
                          ? "bg-destructive text-white"
                          : selected
                            ? "bg-primary text-primary-foreground"
                            : "bg-background text-muted-foreground inset-ring inset-ring-border"
                    )}
                    aria-hidden
                  >
                    {chuCai(i)}
                  </span>
                )}
                <span className="min-w-0">{option}</span>
              </button>
            );
          })}
        </div>
      )}

      {item.kind === "fill" && (
        <Input
          value={typeof value === "string" ? value : ""}
          onChange={(e) => {
            setPicked([]); // gõ tay thì bỏ liên kết với các mảnh gợi ý
            onAnswer(e.target.value);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && answered) onCheck();
          }}
          aria-label={item.label ?? item.prompt}
          className={cn(
            "h-9",
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
            aria-label={item.label ?? item.prompt}
            className="w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
          />
        ) : (
          <Input
            value={typeof value === "string" ? value : ""}
            onChange={(e) => onAnswer(e.target.value)}
            aria-label={item.label ?? item.prompt}
            className="h-9"
          />
        ))}

      {hintOpen && chunks.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {chunks.map((chunk, i) => {
            const used = picked.includes(i);
            return (
              <button
                key={`${chunk}-${i}`}
                type="button"
                onClick={() => toggleChunk(i)}
                className={cn(
                  "rounded-md border px-2 py-0.5 text-sm transition-colors",
                  used
                    ? "border-dashed border-border text-muted-foreground/40"
                    : "border-border/70 hover:bg-muted"
                )}
              >
                {chunk}
              </button>
            );
          })}
        </div>
      )}

      <div className="flex items-center gap-1.5">
        {isGradable(item) && !checked && (
          <Button size="sm" className="h-7" disabled={!answered} onClick={onCheck}>
            Kiểm tra
          </Button>
        )}
        {canHint && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-muted-foreground"
            onClick={() => setHintOpen((v) => !v)}
          >
            <Puzzle className="size-3.5" aria-hidden />
            {hintOpen ? "Ẩn gợi ý" : "Gợi ý"}
          </Button>
        )}
        {/* Kiểu bài học đã có nhãn đúng/sai ở đầu thẻ; chỉ câu điền mới cần
            nhắc lại ở đây vì còn phải hiện đáp án đúng. */}
        {isGradable(item) && checked && (stt === undefined || item.kind === "fill") && (
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
                {item.kind === "fill" && ` — ${item.answers[0]}`}
              </>
            )}
          </span>
        )}
        {item.kind === "free" && item.model && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-muted-foreground"
            onClick={() => setShowModel((v) => !v)}
          >
            <Lightbulb className="size-3.5" aria-hidden />
            {showModel ? "Ẩn câu mẫu" : "Câu mẫu"}
          </Button>
        )}
      </div>

      {item.kind === "free" && item.model && showModel && (
        <p className="border-l-2 border-border pl-2.5 text-xs leading-relaxed whitespace-pre-line text-muted-foreground">
          {item.model}
        </p>
      )}

      {checked && item.explain && (
        <p className="border-l-2 border-border pl-2.5 text-xs leading-relaxed text-muted-foreground">
          {item.explain}
        </p>
      )}
    </div>
  );
}
