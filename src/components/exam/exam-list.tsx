"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { practiceProgress, totalMinutes, type Exam, type ExamLevel } from "@/lib/exams";
import { finishedAttempts, useExamStore, type MockAttempt } from "@/lib/exam-store";
import { LevelBadge, ProgressRing } from "@/components/exam/exam-chrome";

const NO_SUBSCRIBE = () => () => {};

const LEVELS: { id: ExamLevel; hint: string }[] = [
  { id: "TOPIK I", hint: "Cấp 1–2 · Nghe, Đọc" },
  { id: "TOPIK II", hint: "Cấp 3–6 · Nghe, Viết, Đọc" },
];

function best(attempts: MockAttempt[]): MockAttempt | null {
  return attempts.reduce<MockAttempt | null>((b, a) => (b === null || (a.score ?? 0) > (b.score ?? 0) ? a : b), null);
}

/**
 * Danh sách đề theo cấp: mỗi kỳ một dòng gọn — tiến độ luyện từng câu (vòng
 * tròn) và điểm thi thử cao nhất kèm cấp.
 */
export function ExamList({ exams }: { exams: Exam[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const attempts = useExamStore((s) => s.attempts);
  const practice = useExamStore((s) => s.practice);
  const [level, setLevel] = useState<ExamLevel>("TOPIK I");

  const shown = exams.filter((e) => e.level === level);
  const sample = shown[0];
  const done = isClient ? attempts.filter((a) => a.finishedAt && shown.some((e) => e.id === a.examId)) : [];
  const top = best(done);
  const triedExams = new Set(done.map((a) => a.examId)).size;

  return (
    <div className="flex flex-col gap-4">
      <div role="tablist" aria-label="Cấp đề thi" className="grid grid-cols-2 gap-1 rounded-lg bg-muted p-1">
        {LEVELS.map((l) => (
          <button
            key={l.id}
            type="button"
            role="tab"
            aria-selected={level === l.id}
            onClick={() => setLevel(l.id)}
            className={cn(
              "flex flex-col items-center rounded-md px-3 py-1.5 text-sm transition-colors",
              level === l.id ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            {l.id}
            <span className="text-xs font-normal text-muted-foreground">{l.hint}</span>
          </button>
        ))}
      </div>

      <p className="text-sm text-muted-foreground">
        {shown.length} đề{sample ? ` · mỗi đề ${totalMinutes(sample)} phút` : ""}
        {triedExams > 0 && top
          ? ` · đã thi thử ${triedExams} đề, cao nhất ${top.score} điểm${top.level ? ` (${top.level})` : ""}`
          : ""}
      </p>

      <ul className="flex flex-col divide-y divide-border border-y border-border">
        {shown.map((exam) => {
          const mine = isClient ? finishedAttempts(attempts, exam.id) : [];
          const b = best(mine);
          const progress = practiceProgress(exam, isClient ? practice[exam.id] : undefined);
          return (
            <li key={exam.id}>
              <Link
                href={`/exam/${exam.id}`}
                className="group -mx-2 flex items-center gap-4 rounded-lg px-2 py-3.5 transition-colors hover:bg-muted/60"
              >
                <span className="min-w-0 flex-1">
                  <span className="block font-medium">Kỳ {exam.round}</span>
                  <span className="block text-sm text-muted-foreground">Năm {exam.year}</span>
                </span>
                <span
                  className="flex items-center gap-2 text-xs text-muted-foreground tabular-nums"
                  title="Số câu đã luyện"
                >
                  <ProgressRing value={progress.done} max={progress.total} size={28} />
                  <span className="w-12">
                    {progress.done}/{progress.total}
                  </span>
                </span>
                <span className="flex w-24 flex-col items-end gap-0.5 text-sm tabular-nums">
                  {b ? (
                    <>
                      <span className="font-medium">{b.score} điểm</span>
                      <LevelBadge level={b.level} />
                    </>
                  ) : (
                    <span className="text-xs text-muted-foreground">Chưa thi thử</span>
                  )}
                </span>
                <ChevronRight
                  className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                  aria-hidden
                />
              </Link>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
