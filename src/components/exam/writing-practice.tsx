"use client";

import { useState, useSyncExternalStore } from "react";
import { ChevronLeft, ChevronRight, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import { qKey, type Exam, type ExamSection } from "@/lib/exams";
import { useExamStore } from "@/lib/exam-store";
import { SelfGrade, WritingImage, WritingInput, hasWritten } from "@/components/exam/writing-parts";
import {
  ActionBar,
  PracticeHeader,
  QuestionGrid,
  QuestionGridSheet,
  type GridItem,
} from "@/components/exam/exam-chrome";

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
  const isClient = useSyncExternalStore(
    NO_SUBSCRIBE,
    () => true,
    () => false
  );
  const tasks = section.writing!.tasks;
  const [index, setIndex] = useState(() =>
    Math.max(
      0,
      tasks.findIndex((t) => t.no === initialNo)
    )
  );
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
  const grid: GridItem[] = tasks.map((t) => ({
    no: t.no,
    state: grades[t.no] !== undefined ? "right" : hasWritten(t, texts) ? "answered" : "idle",
    suffix: grades[t.no] !== undefined ? ` · ${grades[t.no]}` : undefined,
    label: `Câu ${t.no}${grades[t.no] !== undefined ? `, tự chấm ${grades[t.no]} điểm` : ""}`,
  }));

  const go = (to: number) => {
    setIndex(Math.max(0, Math.min(tasks.length - 1, to)));
    window.scrollTo({ top: 0 });
  };

  return (
    // Máy tính: trang đề dính bên trái, ô viết + đáp án mẫu bên phải.
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6 pb-24 sm:pb-12 lg:max-w-none">
      <PracticeHeader
        exam={exam}
        section={section}
        done={graded.length}
        total={tasks.length}
        summary={`tự chấm ${graded.length}/${tasks.length} · ${graded.reduce((n, t) => n + grades[t.no], 0)} điểm`}
      />

      <article className="flex flex-col gap-4">
        <h1 className="flex items-baseline gap-2">
          <span className="text-xl font-semibold tabular-nums">Câu {task.no}</span>
          <span className="text-sm text-muted-foreground">{task.points} điểm</span>
        </h1>

        <div className="flex flex-col gap-4 lg:grid lg:grid-cols-2 lg:items-start lg:gap-8">
          <div className="lg:sticky lg:top-20">
            <WritingImage exam={exam} src={task.page} alt={`Đề câu ${task.no <= 52 ? "51–52" : "53–54"}`} />
          </div>
          <div className="flex flex-col gap-4">
            <WritingInput task={task} texts={texts} onChange={(key, text) => setText(exam.id, key, text)} />

            {revealed ? (
              <div className="flex flex-col gap-3 border-t border-border pt-4">
                <h2 className="text-sm font-semibold">Đáp án mẫu và tiêu chí chấm chính thức</h2>
                <p className="text-xs text-muted-foreground">
                  Tìm dòng câu {task.no} trong bảng. Đáp án mẫu chỉ là một cách viết — ý đúng, ngữ pháp đúng là được
                  điểm.
                </p>
                {section.writing!.modelAnswers.map((src, i) => (
                  <WritingImage key={src} exam={exam} src={src} alt={`Đáp án mẫu phần viết, trang ${i + 1}`} />
                ))}
                <SelfGrade task={task} value={grades[task.no]} onChange={(p) => setGrade(exam.id, task.no, p)} />
              </div>
            ) : null}
          </div>
        </div>
      </article>

      <ActionBar>
        <Button
          variant="ghost"
          size="icon-lg"
          onClick={() => go(index - 1)}
          disabled={index === 0}
          aria-label="Câu trước"
        >
          <ChevronLeft className="size-5" aria-hidden />
        </Button>
        <QuestionGridSheet items={grid} current={index} onPick={go} count={`${graded.length}/${tasks.length}`} />
        <div className="flex flex-1 justify-end gap-2 sm:flex-none">
          {revealed ? (
            <Button size="lg" onClick={() => go(index + 1)} disabled={index === tasks.length - 1} className="min-w-28">
              Câu sau
              <ChevronRight className="size-4" aria-hidden />
            </Button>
          ) : (
            <Button
              size="lg"
              variant={hasWritten(task, texts) ? "default" : "outline"}
              onClick={() => check(exam.id, qKey("writing", task.no))}
            >
              <Eye className="size-4" aria-hidden />
              Xem đáp án mẫu
            </Button>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon-lg"
          onClick={() => go(index + 1)}
          disabled={index === tasks.length - 1}
          aria-label="Câu sau"
        >
          <ChevronRight className="size-5" aria-hidden />
        </Button>
      </ActionBar>

      <section aria-label="Bảng câu" className="hidden border-t border-border pt-5 sm:block">
        <QuestionGrid items={grid} current={index} onPick={go} />
      </section>
    </div>
  );
}
