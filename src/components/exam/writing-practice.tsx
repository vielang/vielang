"use client";

import { useState, useSyncExternalStore } from "react";
import { ChevronLeft, ChevronRight, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { qKey, type Exam, type ExamSection } from "@/lib/exams";
import { useExamStore } from "@/lib/exam-store";
import { SelfGrade, WritingImage, WritingInput, hasWritten } from "@/components/exam/writing-parts";

const NO_SUBSCRIBE = () => () => {};

/**
 * Luyện phần viết (TOPIK II, câu 51–54): xem trang đề, viết bài, mở đáp án
 * mẫu chính thức rồi tự chấm. Bài viết và điểm tự chấm lưu theo đề như bài
 * luyện trắc nghiệm.
 */
export function WritingPractice({
  exam,
  section,
  initialNo,
}: {
  exam: Exam;
  section: ExamSection;
  initialNo?: number;
}) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const tasks = section.writing!.tasks;
  const [index, setIndex] = useState(() => Math.max(0, tasks.findIndex((t) => t.no === initialNo)));
  const task = tasks[index];

  const practice = useExamStore((s) => s.practice[exam.id]);
  const setText = useExamStore((s) => s.setPracticeText);
  const setGrade = useExamStore((s) => s.setPracticeGrade);
  const check = useExamStore((s) => s.checkPractice);

  const texts = isClient ? (practice?.texts ?? {}) : {};
  const grades = isClient ? (practice?.grades ?? {}) : {};
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const revealed = checked.has(qKey("writing", task.no));
  const graded = tasks.filter((t) => grades[t.no] !== undefined);

  const go = (to: number) => {
    setIndex(Math.max(0, Math.min(tasks.length - 1, to)));
    window.scrollTo({ top: 0 });
  };

  return (
    <>
      <div className="flex items-baseline justify-between gap-3">
        <h1 className="text-lg font-semibold">
          Câu {task.no} <span className="text-sm font-normal text-muted-foreground">· {task.points} điểm</span>
        </h1>
        <span className="text-sm text-muted-foreground tabular-nums">
          Tự chấm {graded.length}/{tasks.length} câu · {graded.reduce((n, t) => n + grades[t.no], 0)} điểm
        </span>
      </div>

      <WritingImage exam={exam} src={task.page} alt={`Đề câu ${task.no <= 52 ? "51–52" : "53–54"}`} />

      <WritingInput task={task} texts={texts} onChange={(key, text) => setText(exam.id, key, text)} />

      {revealed ? (
        <div className="flex flex-col gap-3">
          <h2 className="text-sm font-semibold">Đáp án mẫu và tiêu chí chấm chính thức</h2>
          <p className="text-xs text-muted-foreground">
            Tìm dòng câu {task.no} trong bảng. Đáp án mẫu chỉ là một cách viết — ý đúng, ngữ pháp đúng là được điểm.
          </p>
          {section.writing!.modelAnswers.map((src, i) => (
            <WritingImage key={src} exam={exam} src={src} alt={`Đáp án mẫu phần viết, trang ${i + 1}`} />
          ))}
          <SelfGrade task={task} value={grades[task.no]} onChange={(p) => setGrade(exam.id, task.no, p)} />
        </div>
      ) : (
        <Button
          onClick={() => check(exam.id, qKey("writing", task.no))}
          variant={hasWritten(task, texts) ? "default" : "outline"}
          className="self-start"
        >
          <Eye className="size-4" aria-hidden />
          Xem đáp án mẫu và tự chấm
        </Button>
      )}

      <div className="flex items-center justify-between gap-3 border-t border-border pt-4">
        <Button variant="ghost" onClick={() => go(index - 1)} disabled={index === 0}>
          <ChevronLeft className="size-4" aria-hidden />
          Câu trước
        </Button>
        <Button variant="ghost" onClick={() => go(index + 1)} disabled={index === tasks.length - 1}>
          Câu sau
          <ChevronRight className="size-4" aria-hidden />
        </Button>
      </div>

      <nav aria-label="Chọn câu" className="flex flex-wrap gap-1.5">
        {tasks.map((t, i) => (
          <button
            key={t.no}
            type="button"
            onClick={() => go(i)}
            aria-current={i === index ? "step" : undefined}
            aria-label={`Câu ${t.no}${grades[t.no] !== undefined ? `, tự chấm ${grades[t.no]} điểm` : ""}`}
            className={cn(
              "flex h-9 min-w-12 items-center justify-center rounded-md border px-2 text-xs tabular-nums",
              grades[t.no] !== undefined
                ? "border-primary bg-primary text-primary-foreground"
                : hasWritten(t, texts)
                  ? "border-foreground/40 bg-muted"
                  : "border-border",
              i === index && "ring-2 ring-primary ring-offset-2 ring-offset-background"
            )}
          >
            {t.no}
            {grades[t.no] !== undefined && ` · ${grades[t.no]}`}
          </button>
        ))}
      </nav>
    </>
  );
}
