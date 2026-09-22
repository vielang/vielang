"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowLeft, ArrowRight, Headphones, ListChecks, PenLine, Timer } from "lucide-react";
import { Button } from "@/components/ui/button";
import { examTitle, qKey, sectionCount, sectionVi, totalMinutes, type Exam, type Grades } from "@/lib/exams";
import { activeAttempt, finishedAttempts, useExamStore } from "@/lib/exam-store";

const NO_SUBSCRIBE = () => () => {};

/** Trang một đề: chọn thi thử hay luyện từng phần, xem các lượt đã thi. */
export function ExamOverview({ exam, source }: { exam: Exam; source?: string }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const attempts = useExamStore((s) => s.attempts);
  const practice = useExamStore((s) => s.practice[exam.id]);
  const active = isClient ? activeAttempt(attempts, exam.id) : undefined;
  const history = isClient ? finishedAttempts(attempts, exam.id) : [];
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const grades: Grades = isClient ? (practice?.grades ?? {}) : {};

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8">
      <Link href="/exam" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="size-4" aria-hidden />
        Luyện thi TOPIK
      </Link>

      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-6">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{examTitle(exam)}</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Năm {exam.year} · {exam.sections.map((s) => `${sectionVi(s.id)} ${sectionCount(s)} câu`).join(" · ")} ·{" "}
            {totalMinutes(exam)} phút
          </p>
        </div>
        <Button asChild className="self-start">
          <Link href={`/exam/${exam.id}/mock`}>
            <Timer className="size-4" aria-hidden />
            {active ? "Làm tiếp lượt thi thử đang dở" : "Thi thử có tính giờ"}
            <ArrowRight className="size-4" aria-hidden />
          </Link>
        </Button>
      </section>

      <section className="flex flex-col gap-3">
        <h2 className="text-base font-semibold">Luyện từng câu</h2>
        <div className="grid gap-3 sm:grid-cols-2">
          {exam.sections.map((s) => {
            const done = s.writing
              ? s.writing.tasks.filter((t) => grades[t.no] !== undefined).length
              : s.questions.filter((q) => checked.has(qKey(s.id, q.no))).length;
            const count = sectionCount(s);
            return (
              <Link
                key={s.id}
                href={`/exam/${exam.id}/practice?section=${s.id}`}
                className="flex flex-col gap-2 rounded-xl border border-border p-4 transition-colors hover:bg-muted/60"
              >
                <span className="flex items-center gap-2 font-medium">
                  {s.audio ? (
                    <Headphones className="size-4" aria-hidden />
                  ) : s.writing ? (
                    <PenLine className="size-4" aria-hidden />
                  ) : (
                    <ListChecks className="size-4" aria-hidden />
                  )}
                  {sectionVi(s.id)} <span className="font-korean text-sm text-muted-foreground">{s.title}</span>
                </span>
                <span className="text-sm text-muted-foreground tabular-nums">
                  {done > 0 ? `Đã ${s.writing ? "chấm" : "làm"} ${done}/${count} câu` : `${count} câu`} —{" "}
                  {s.writing
                    ? "viết rồi tự chấm theo đáp án mẫu"
                    : `chấm ngay từng câu${s.audio ? ", nghe lại từng câu" : ""}`}
                </span>
              </Link>
            );
          })}
        </div>
      </section>

      {history.length > 0 && (
        <section className="flex flex-col gap-3">
          <h2 className="text-base font-semibold">Các lần thi thử</h2>
          <ul className="flex flex-col divide-y divide-border rounded-xl border border-border">
            {history.map((a) => (
              <li key={a.id}>
                <Link
                  href={`/exam/${exam.id}/result?attempt=${a.id}`}
                  className="flex items-center justify-between gap-3 px-4 py-3 text-sm hover:bg-muted/60"
                >
                  <span>{new Date(a.finishedAt!).toLocaleString("vi-VN", { dateStyle: "short", timeStyle: "short" })}</span>
                  <span className="tabular-nums">
                    <span className="font-medium">{a.score}</span> điểm · {a.level ?? "chưa đạt"}
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        </section>
      )}

      {source && <p className="text-xs text-muted-foreground">Nguồn: {source}</p>}
    </div>
  );
}
