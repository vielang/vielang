"use client";

import Link from "next/link";
import { ChevronRight, Headphones, ListChecks, PenLine, Play, Timer } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  examTitle,
  nextPracticeTarget,
  practiceProgress,
  qKey,
  scoreExam,
  sectionCount,
  sectionVi,
  totalMinutes,
  type Exam,
  type ExamSection,
  type Grades,
} from "@/lib/exams";
import { activeAttempt, finishedAttempts, useExamStore } from "@/lib/exam-store";
import { LevelBadge, ProgressBar } from "@/components/exam/exam-chrome";
import { useIsClient } from "@/lib/use-is-client";
import { BackLink } from "@/components/layout/back-link";

function SectionIcon({ section }: { section: ExamSection }) {
  const Icon = section.audio ? Headphones : section.writing ? PenLine : ListChecks;
  return <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden />;
}

/** Trang một đề: thi thử hoặc luyện tiếp, tiến độ từng phần, các lượt đã thi. */
export function ExamOverview({ exam, source }: { exam: Exam; source?: string }) {
  const isClient = useIsClient();
  const attempts = useExamStore((s) => s.attempts);
  const stored = useExamStore((s) => s.practice[exam.id]);
  const practice = isClient ? stored : undefined;
  const active = isClient ? activeAttempt(attempts, exam.id) : undefined;
  const history = isClient ? finishedAttempts(attempts, exam.id) : [];
  const checked = new Set(practice?.checked ?? []);
  const grades: Grades = practice?.grades ?? {};
  const progress = practiceProgress(exam, practice);
  const target = nextPracticeTarget(exam, practice);

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-10">
      <BackLink href="/exam">Luyện thi TOPIK</BackLink>

      <section className="-mt-4 flex flex-col gap-5">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{examTitle(exam)}</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Năm {exam.year} · {exam.sections.map((s) => `${sectionVi(s.id)} ${sectionCount(s)} câu`).join(" · ")} ·{" "}
            {totalMinutes(exam)} phút
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="lg">
            <Link href={`/exam/${exam.id}/mock`}>
              <Timer className="size-4" aria-hidden />
              {active ? "Làm tiếp bài thi thử" : "Thi thử có tính giờ"}
            </Link>
          </Button>
          <Button asChild size="lg" variant="outline">
            <Link href={`/exam/${exam.id}/practice?section=${target.section}&q=${target.no}`}>
              <Play className="size-4" aria-hidden />
              {progress.done > 0
                ? `Luyện tiếp: ${sectionVi(target.section)} câu ${target.no}`
                : "Bắt đầu luyện từng câu"}
            </Link>
          </Button>
        </div>
        {active && (
          <p className="text-sm text-muted-foreground">
            Bạn đang có một lượt thi thử làm dở — đồng hồ vẫn chạy theo giờ thật.
          </p>
        )}
      </section>

      <section className="flex flex-col gap-2">
        <div className="flex items-baseline justify-between gap-3">
          <h2 className="text-base font-semibold">Luyện từng câu</h2>
          <span className="text-xs text-muted-foreground tabular-nums">
            {progress.done}/{progress.total} câu
          </span>
        </div>
        <ul className="flex flex-col divide-y divide-border border-y border-border">
          {exam.sections.map((s) => {
            const count = sectionCount(s);
            const done = s.writing
              ? s.writing.tasks.filter((t) => grades[t.no] !== undefined).length
              : s.questions.filter((q) => checked.has(qKey(s.id, q.no))).length;
            const right = s.questions.filter(
              (q) => checked.has(qKey(s.id, q.no)) && practice?.answers[qKey(s.id, q.no)] === q.answer
            ).length;
            return (
              <li key={s.id}>
                <Link
                  href={`/exam/${exam.id}/practice?section=${s.id}`}
                  className="group -mx-2 flex items-center gap-3 rounded-lg px-2 py-3.5 transition-colors hover:bg-muted/60"
                >
                  <SectionIcon section={s} />
                  <span className="flex min-w-0 flex-1 flex-col gap-1.5">
                    <span className="flex flex-wrap items-baseline gap-x-2">
                      <span className="font-medium">{sectionVi(s.id)}</span>
                      <span className="font-korean text-xs text-muted-foreground">{s.title}</span>
                    </span>
                    <ProgressBar value={done} max={count} />
                  </span>
                  <span className="w-28 text-right text-xs text-muted-foreground tabular-nums">
                    {s.writing ? `tự chấm ${done}/${count}` : done > 0 ? `${done}/${count} · đúng ${right}` : `${count} câu`}
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
      </section>

      {history.length > 0 && (
        <section className="flex flex-col gap-2">
          <h2 className="text-base font-semibold">Các lần thi thử</h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm tabular-nums">
              <thead>
                <tr className="border-b border-border text-left text-xs text-muted-foreground">
                  <th className="py-2 pr-3 font-normal">Ngày</th>
                  {exam.sections.map((s) => (
                    <th key={s.id} className="px-2 py-2 text-right font-normal">
                      {sectionVi(s.id)}
                    </th>
                  ))}
                  <th className="px-2 py-2 text-right font-normal">Tổng</th>
                  <th className="py-2 pl-2 text-right font-normal">Cấp</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {history.map((a) => {
                  const r = scoreExam(exam, a.answers, a.grades);
                  return (
                    <tr key={a.id} className="group">
                      <td className="py-2.5 pr-3">
                        <Link
                          href={`/exam/${exam.id}/result?attempt=${a.id}`}
                          className="underline-offset-2 group-hover:underline"
                        >
                          {new Date(a.finishedAt!).toLocaleString("vi-VN", { dateStyle: "short", timeStyle: "short" })}
                        </Link>
                      </td>
                      {r.sections.map((s) => (
                        <td key={s.id} className="px-2 py-2.5 text-right text-muted-foreground">
                          {s.id === "writing" && s.correct < s.total ? `${s.score}*` : s.score}
                        </td>
                      ))}
                      <td className="px-2 py-2.5 text-right font-medium">{r.score}</td>
                      <td className="py-2.5 pl-2 text-right">
                        <LevelBadge level={r.level} />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          {history.some((a) => scoreExam(exam, a.answers, a.grades).ungraded > 0) && (
            <p className="text-xs text-muted-foreground">* Phần viết chưa tự chấm hết — mở lượt thi để chấm.</p>
          )}
        </section>
      )}

      {source && <p className="text-xs text-muted-foreground">Nguồn: {source}</p>}
    </div>
  );
}
