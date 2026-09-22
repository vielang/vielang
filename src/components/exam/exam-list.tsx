"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { examTitle, sectionVi, totalMinutes, type Exam } from "@/lib/exams";
import { finishedAttempts, useExamStore } from "@/lib/exam-store";

const NO_SUBSCRIBE = () => () => {};

/** Danh sách đề, kèm điểm cao nhất của người học (nếu đã thi thử). */
export function ExamList({ exams }: { exams: Exam[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const attempts = useExamStore((s) => s.attempts);

  return (
    <ul className="flex flex-col gap-3">
      {exams.map((exam) => {
        const done = isClient ? finishedAttempts(attempts, exam.id) : [];
        const best = done.reduce<(typeof done)[number] | null>(
          (b, a) => (b === null || (a.score ?? 0) > (b.score ?? 0) ? a : b),
          null
        );
        return (
          <li key={exam.id}>
            <Link
              href={`/exam/${exam.id}`}
              className="group flex items-center gap-4 rounded-xl border border-border p-4 transition-colors hover:bg-muted/60"
            >
              <span className="min-w-0 flex-1">
                <span className="block font-medium">{examTitle(exam)}</span>
                <span className="block text-sm text-muted-foreground">
                  Năm {exam.year} · {exam.sections.map((s) => sectionVi(s.id)).join(", ")} ·{" "}
                  {totalMinutes(exam)} phút
                </span>
              </span>
              {best && (
                <span className="shrink-0 text-right text-sm tabular-nums">
                  <span className="block font-medium">{best.score} điểm</span>
                  <span className="block text-xs text-muted-foreground">cao nhất · {best.level ?? "chưa đạt"}</span>
                </span>
              )}
              <ChevronRight className="size-4 shrink-0 text-muted-foreground" aria-hidden />
            </Link>
          </li>
        );
      })}
    </ul>
  );
}
