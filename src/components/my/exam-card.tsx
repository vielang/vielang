"use client";

import Link from "next/link";
import { ChevronRight, GraduationCap } from "lucide-react";
import { examTitle, getExam } from "@/lib/exams";
import { finishedAttempts, useExamStore } from "@/lib/exam-store";

/**
 * Thẻ "Luyện thi TOPIK" trên Góc học tập: lượt thi thử gần nhất (điểm, cấp),
 * hoặc lượt đang làm dở để làm tiếp, hoặc lời mời thử một đề.
 */
export function ExamCard() {
  const attempts = useExamStore((s) => s.attempts);
  const active = attempts.find((a) => !a.finishedAt);
  const last = finishedAttempts(attempts)[0];
  const exam = getExam((active ?? last)?.examId ?? "");

  const href = active
    ? `/exam/${active.examId}/mock`
    : last
      ? `/exam/${last.examId}/result?attempt=${last.id}`
      : "/exam/topik-i";

  return (
    <Link
      href={href}
      className="group flex items-center gap-3 rounded-xl border border-border p-4 transition-colors hover:bg-muted/60"
    >
      <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-muted">
        <GraduationCap className="size-5" aria-hidden />
      </span>
      <span className="min-w-0 flex-1">
        {active && exam ? (
          <>
            <span className="block font-medium">Làm tiếp thi thử {examTitle(exam)}</span>
            <span className="block text-sm text-muted-foreground">Bạn đang làm dở — đồng hồ vẫn chạy.</span>
          </>
        ) : last && exam ? (
          <>
            <span className="block font-medium">
              {examTitle(exam)}: {last.score} điểm · {last.level ? `đạt ${last.level}` : "chưa đạt cấp"}
            </span>
            <span className="block text-sm text-muted-foreground">
              Lần thi thử gần nhất, {new Date(last.finishedAt!).toLocaleDateString("vi-VN")} — xem lại câu sai
            </span>
          </>
        ) : (
          <>
            <span className="block font-medium">Thử sức với đề TOPIK thật</span>
            <span className="block text-sm text-muted-foreground">
              Luyện từng câu hoặc thi thử có tính giờ để biết mình đang ở cấp nào.
            </span>
          </>
        )}
      </span>
      <ChevronRight className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" aria-hidden />
    </Link>
  );
}
