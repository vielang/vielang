"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import { qKey, writingBlocks, type Exam, type ExamSection } from "@/lib/exams";
import { useExamStore } from "@/lib/exam-store";
import { WritingTaskView, hasWritten } from "@/components/exam/writing-parts";
import {
  ActionBar,
  NextButton,
  PracticeHeader,
  QuestionGrid,
  QuestionGridSheet,
  rangeLabel,
  type GridItem,
} from "@/components/exam/exam-chrome";
import { useIsClient } from "@/lib/use-is-client";

/**
 * Luyện phần viết (TOPIK II) THEO KHỐI như Nghe/Đọc: [51–52] (điền ㉠ ㉡, cùng
 * lời chỉ dẫn), [53], [54]. Mỗi câu có ảnh đề riêng (cắt từ trang in) và ô
 * viết; "Xem đáp án mẫu" mở đáp án chính thức của từng câu ngay dưới ô viết
 * kèm thanh tự chấm. Bài viết và điểm tự chấm lưu theo đề.
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
  const isClient = useIsClient();
  const tasks = section.writing!.tasks;
  const blocks = writingBlocks(tasks);
  const blockOf = (no: number) => Math.max(0, blocks.findIndex((b) => b.some((t) => t.no === no)));
  const [bi, setBi] = useState(() => (initialNo ? blockOf(initialNo) : 0));
  const block = blocks[bi];
  const isLast = bi === blocks.length - 1;
  const label = (b: typeof block | undefined) => b && rangeLabel(b[0].no, b[b.length - 1].no);

  const practice = useExamStore((s) => s.practice[exam.id]);
  const setText = useExamStore((s) => s.setPracticeText);
  const setGrade = useExamStore((s) => s.setPracticeGrade);
  const check = useExamStore((s) => s.checkPractice);

  const texts = isClient ? (practice?.texts ?? {}) : {};
  const grades = isClient ? (practice?.grades ?? {}) : {};
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const revealed = block.every((t) => checked.has(qKey("writing", t.no)));
  const graded = tasks.filter((t) => grades[t.no] !== undefined);
  const grid: GridItem[] = tasks.map((t) => ({
    no: t.no,
    state: grades[t.no] !== undefined ? "right" : hasWritten(t, texts) ? "answered" : "idle",
    suffix: grades[t.no] !== undefined ? ` · ${grades[t.no]}` : undefined,
    label: `Câu ${t.no}${grades[t.no] !== undefined ? `, tự chấm ${grades[t.no]} điểm` : ""}`,
  }));
  const current = tasks.findIndex((t) => t.no === block[0].no);

  const go = (to: number) => {
    setBi(Math.max(0, Math.min(blocks.length - 1, to)));
    window.scrollTo({ top: 0 });
  };
  const reveal = () => {
    for (const t of block) check(exam.id, qKey("writing", t.no));
  };

  const range = label(block)!;
  const points = block.reduce((n, t) => n + t.points, 0);

  return (
    // Máy tính: bài viết chia đôi (đề | ô viết) nên dùng cả bề ngang.
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6 pb-24 sm:pb-12 lg:max-w-none">
      <PracticeHeader
        exam={exam}
        section={section}
        done={graded.length}
        total={tasks.length}
        summary={`tự chấm ${graded.length}/${tasks.length} · ${graded.reduce((n, t) => n + grades[t.no], 0)} điểm`}
      />

      <section className="flex flex-col gap-6" aria-label={range}>
        <h1 className="flex items-baseline gap-2">
          <span className="text-xl font-semibold tabular-nums">{range}</span>
          <span className="text-sm text-muted-foreground">{points} điểm</span>
        </h1>
        {block.map((t, i) => (
          <div key={t.no} className={i > 0 ? "border-t border-border pt-6" : undefined}>
            <WritingTaskView
              exam={exam}
              task={t}
              texts={texts}
              onText={(key, text) => setText(exam.id, key, text)}
              revealed={revealed}
              grade={grades[t.no]}
              onGrade={(p) => setGrade(exam.id, t.no, p)}
              showHead={block.length > 1}
            />
          </div>
        ))}
      </section>

      <ActionBar>
        <Button variant="ghost" size="icon-lg" onClick={() => go(bi - 1)} disabled={bi === 0} aria-label={label(blocks[bi - 1]) ?? "Câu trước"}>
          <ChevronLeft className="size-5" aria-hidden />
        </Button>
        <QuestionGridSheet
          items={grid}
          current={current}
          onPick={(i) => go(blockOf(tasks[i].no))}
          count={`${graded.length}/${tasks.length}`}
        />
        <div className="flex flex-1 justify-end gap-2 sm:flex-none">
          {revealed ? (
            <NextButton exam={exam} section={section} next={label(blocks[bi + 1])} onNext={() => go(bi + 1)} />
          ) : (
            <Button size="lg" variant={block.some((t) => hasWritten(t, texts)) ? "default" : "outline"} onClick={reveal}>
              <Eye className="size-4" aria-hidden />
              Xem đáp án mẫu
            </Button>
          )}
        </div>
        <Button variant="ghost" size="icon-lg" onClick={() => go(bi + 1)} disabled={isLast} aria-label={label(blocks[bi + 1]) ?? "Câu sau"}>
          <ChevronRight className="size-5" aria-hidden />
        </Button>
      </ActionBar>

      <section aria-label="Bảng câu" className="hidden border-t border-border pt-5 sm:block">
        <QuestionGrid items={grid} current={current} onPick={(i) => go(blockOf(tasks[i].no))} />
      </section>
    </div>
  );
}
