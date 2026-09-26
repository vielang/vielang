"use client";

import Link from "next/link";
import { ChevronRight, GraduationCap } from "lucide-react";
import { examTitle, getExam, milestoneLabel, reachedMilestone } from "@/lib/exams";
import { finishedAttempts, useExamStore } from "@/lib/exam-store";
import { EXAM_LEVELS, examLevelHref } from "@/lib/exam-levels";

/**
 * Thẻ "Luyện thi" trên Góc học tập: lượt thi thử gần nhất (điểm, mốc), hoặc
 * lượt đang làm dở để làm tiếp, hoặc lời mời thử một đề.
 */
export function ExamCard() {
  // Chỉ lượt của đề còn trên web — bản lưu cũ có thể còn lượt của đề đã gỡ,
  // dẫn tới đó là trang 404.
  const attempts = useExamStore((s) => s.attempts).filter((a) => getExam(a.examId));
  const active = attempts.find((a) => !a.finishedAt);
  const last = finishedAttempts(attempts)[0];
  const exam = getExam((active ?? last)?.examId ?? "");
  const reached = last && exam ? reachedMilestone(exam, last.score ?? 0) : null;

  const href = active
    ? `/exam/${active.examId}/mock`
    : last
      ? `/exam/${last.examId}/result?attempt=${last.id}`
      : examLevelHref(EXAM_LEVELS[0].id);

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
              {examTitle(exam)}: {last.score} điểm
              {reached ? ` · mốc ${milestoneLabel(exam, reached)}` : ""}
            </span>
            <span className="block text-sm text-muted-foreground">
              Lần thi thử gần nhất, {new Date(last.finishedAt!).toLocaleDateString("vi-VN")} — xem lại câu sai
            </span>
          </>
        ) : (
          <>
            <span className="block font-medium">Thử sức với đề luyện TOEIC</span>
            <span className="block text-sm text-muted-foreground">
              Luyện từng câu có giải thích, hoặc thi thử có tính giờ để biết mình đang ở khoảng điểm nào.
            </span>
          </>
        )}
      </span>
      <ChevronRight className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" aria-hidden />
    </Link>
  );
}
