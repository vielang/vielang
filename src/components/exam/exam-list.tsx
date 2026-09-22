"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { examTitle, sectionVi, totalMinutes, type Exam, type ExamLevel } from "@/lib/exams";
import { finishedAttempts, useExamStore } from "@/lib/exam-store";

const NO_SUBSCRIBE = () => () => {};

const LEVELS: { id: ExamLevel; hint: string }[] = [
  { id: "TOPIK I", hint: "Cấp 1–2 · Nghe, Đọc" },
  { id: "TOPIK II", hint: "Cấp 3–6 · Nghe, Viết, Đọc" },
];

/** Danh sách đề theo cấp, kèm điểm cao nhất của người học (nếu đã thi thử). */
export function ExamList({ exams }: { exams: Exam[] }) {
  const isClient = useSyncExternalStore(
    NO_SUBSCRIBE,
    () => true,
    () => false
  );
  const attempts = useExamStore((s) => s.attempts);
  const [level, setLevel] = useState<ExamLevel>("TOPIK I");

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
              "flex flex-col items-center rounded-md px-3 py-1.5 text-sm",
              level === l.id ? "bg-background font-medium shadow-sm" : "text-muted-foreground"
            )}
          >
            {l.id}
            <span className="text-xs font-normal text-muted-foreground">{l.hint}</span>
          </button>
        ))}
      </div>
      <ul className="flex flex-col gap-3">
        {exams
          .filter((e) => e.level === level)
          .map((exam) => {
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
                      Năm {exam.year} · {exam.sections.map((s) => sectionVi(s.id)).join(", ")} · {totalMinutes(exam)}{" "}
                      phút
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
    </div>
  );
}
