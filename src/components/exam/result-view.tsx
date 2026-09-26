"use client";

import { useState } from "react";
import Link from "next/link";
import { Pause, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  examAudioUrl,
  examTitle,
  groupOf,
  isFullTest,
  milestoneLabel,
  milestones,
  nextMilestone,
  qKey,
  questionAudio,
  scoreExam,
  sectionVi,
  type Exam,
  type ExamSection,
} from "@/lib/exams";
import { useExamStore, type MockAttempt } from "@/lib/exam-store";
import { Explanation, GroupBlock, PromptView } from "@/components/exam/exam-content";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";
import { MilestoneBadge, ProgressBar } from "@/components/exam/exam-chrome";
import { useIsClient } from "@/lib/use-is-client";
import { BackLink } from "@/components/layout/back-link";

function duration(a: MockAttempt): string {
  if (!a.finishedAt) return "";
  const min = Math.round((Date.parse(a.finishedAt) - Date.parse(a.startedAt)) / 60_000);
  return `${min} phút`;
}

/** Kết quả một lượt thi thử (điểm quy đổi ước tính, mốc mục tiêu) + xem lại từng câu có giải thích. */
export function ResultView({ exam, attemptId }: { exam: Exam; attemptId?: string }) {
  const isClient = useIsClient();
  const attempts = useExamStore((s) => s.attempts);
  const [filter, setFilter] = useState<ReviewFilter>("wrong");
  if (!isClient) return null;

  const attempt = attempts.find((a) => a.id === attemptId && a.examId === exam.id && a.finishedAt);
  if (!attempt) {
    return (
      <div className="mx-auto flex max-w-2xl flex-col items-start gap-3">
        <p>Không tìm thấy lượt thi này trên trình duyệt.</p>
        <Button asChild variant="outline">
          <Link href={`/exam/${exam.id}`}>Về đề {examTitle(exam)}</Link>
        </Button>
      </div>
    );
  }

  const result = scoreExam(exam, attempt.answers);
  const next = nextMilestone(exam, result.score);
  const wrong = exam.sections.flatMap((s) =>
    s.questions.filter((q) => attempt.answers[qKey(s.id, q.no)] !== q.answer).map((q) => ({ section: s.id, no: q.no }))
  );
  const firstWrong = wrong[0];
  const wrongCount = wrong.length;

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8 pb-16">
      <BackLink href={`/exam/${exam.id}`}>{examTitle(exam)}</BackLink>

      <section className="flex flex-col gap-6">
        <div className="flex flex-col gap-2">
          <p className="text-sm text-muted-foreground">
            Kết quả thi thử · {new Date(attempt.finishedAt!).toLocaleDateString("vi-VN")} · làm trong {duration(attempt)}
          </p>
          <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
            <span className="text-5xl font-semibold tracking-tight tabular-nums">
              {result.score}
              <span className="text-xl font-normal text-muted-foreground">/{result.max}</span>
            </span>
            <span className="flex flex-col gap-1">
              <MilestoneBadge exam={exam} score={result.score} className="px-3 py-1 text-sm" />
              {next && (
                <span className="text-xs text-muted-foreground">
                  Còn <b className="text-foreground tabular-nums">{next.need}</b> điểm nữa tới mốc{" "}
                  {milestoneLabel(exam, next.milestone)}
                </span>
              )}
            </span>
          </div>
          <p className="text-xs text-muted-foreground">Điểm quy đổi ước tính — không phải điểm chính thức của ETS.</p>
        </div>

        <MilestoneLadder exam={exam} score={result.score} max={result.max} />

        <ul className="flex flex-col divide-y divide-border border-y border-border">
          {result.sections.map((s) => (
            <li key={s.id} className="grid grid-cols-[3.5rem_1fr_auto] items-center gap-3 py-3 text-sm">
              <span className="font-medium">{sectionVi(s.id)}</span>
              <ProgressBar value={s.score} max={s.max} />
              <span className="text-right tabular-nums">
                <b>{s.score}</b>
                <span className="text-muted-foreground">
                  /{s.max} · đúng {s.correct}/{s.total} câu
                </span>
              </span>
            </li>
          ))}
        </ul>

        <div className="flex flex-wrap gap-2">
          {firstWrong && (
            <Button asChild size="lg">
              <Link href={`/exam/${exam.id}/practice?section=${firstWrong.section}&q=${firstWrong.no}`}>
                Luyện lại {wrongCount} câu chưa đúng
              </Link>
            </Button>
          )}
          <Button asChild size="lg" variant={firstWrong ? "outline" : "default"}>
            <Link href={`/exam/${exam.id}/mock`}>Thi lại</Link>
          </Button>
        </div>
      </section>

      <div className="flex items-center justify-between gap-3">
        <h2 className="text-base font-semibold">Xem lại bài làm</h2>
        <div role="radiogroup" aria-label="Lọc câu" className="flex gap-0.5 rounded-lg bg-muted p-0.5 text-sm">
          {FILTERS.map((f) => (
            <button
              key={f.id}
              type="button"
              role="radio"
              aria-checked={filter === f.id}
              onClick={() => setFilter(f.id)}
              className={cn(
                "rounded-md px-3 py-1 transition-colors",
                filter === f.id ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground"
              )}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>
      {exam.sections.map((s) => (
        <ReviewSection key={s.id} exam={exam} section={s} attempt={attempt} filter={filter} />
      ))}
    </div>
  );
}

/**
 * Thang mốc mục tiêu: thanh 0 → điểm tối đa của đề, vạch ở từng mốc, phần đã
 * đạt tô đậm — nhìn là biết mình đứng đâu và cách mốc sau bao xa. Đề thiếu
 * phần (vd chỉ Reading) thì mốc là một nửa, dưới mỗi mốc ghi tổng 990 tương
 * đương.
 */
function MilestoneLadder({ exam, score, max }: { exam: Exam; score: number; max: number }) {
  const marks = milestones(exam);
  const full = isFullTest(exam);
  const pct = (n: number) => `${(n / max) * 100}%`;
  const caption = full
    ? `Mục tiêu ${marks.map((m) => m.total).join(" / ")}`
    : `Mục tiêu ${marks.map((m) => m.score).join(" / ")} — tương đương tổng ~${marks.map((m) => m.total).join(" / ")}`;
  return (
    <div className="flex flex-col gap-1.5" aria-label={`Thang mốc mục tiêu: ${caption}`} role="img">
      <p className="text-xs text-muted-foreground" aria-hidden>
        {caption}
      </p>
      <div className="relative h-2 rounded-full bg-muted">
        <div className="absolute inset-y-0 left-0 rounded-full bg-foreground/80" style={{ width: pct(Math.min(score, max)) }} />
        {marks.map((m) => (
          <span
            key={m.total}
            className="absolute -top-1 h-4 w-0.5 -translate-x-1/2 rounded-full bg-background ring-1 ring-foreground/30"
            style={{ left: pct(m.score) }}
            aria-hidden
          />
        ))}
      </div>
      <div className="relative h-8 text-[0.7rem] text-muted-foreground tabular-nums">
        {marks.map((m) => (
          <span
            key={m.total}
            className={cn(
              "absolute flex -translate-x-1/2 flex-col items-center leading-tight",
              score >= m.score && "font-medium text-foreground"
            )}
            style={{ left: pct(m.score) }}
          >
            <span>{m.score}</span>
            {!full && <span>~{m.total}</span>}
          </span>
        ))}
      </div>
    </div>
  );
}

type ReviewFilter = "all" | "wrong" | "blank";
const FILTERS: { id: ReviewFilter; label: string }[] = [
  { id: "wrong", label: "Câu sai" },
  { id: "blank", label: "Bỏ trống" },
  { id: "all", label: "Tất cả" },
];

function ReviewSection({
  exam,
  section,
  attempt,
  filter,
}: {
  exam: Exam;
  section: ExamSection;
  attempt: MockAttempt;
  filter: ReviewFilter;
}) {
  const audio = useExamAudio(section.audio ? examAudioUrl(exam, section.audio) : undefined);
  const shown = section.questions.filter((q) => {
    const a = attempt.answers[qKey(section.id, q.no)];
    // "Câu sai" gồm cả câu bỏ trống — đều là câu mất điểm.
    return filter === "all" || (filter === "blank" ? a === undefined : a !== q.answer);
  });
  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-medium text-muted-foreground">
        {sectionVi(section.id)} · {shown.length} câu
      </h3>
      {shown.length === 0 && (
        <p className="text-sm text-muted-foreground">
          {filter === "blank" ? "Không bỏ trống câu nào." : "Không có câu nào sai. Tuyệt vời!"}
        </p>
      )}
      {shown.map((q, i) => {
        // Khối (lời chỉ dẫn + văn bản) chỉ hiện ở câu ĐẦU của khối trong danh
        // sách đang xem — lặp lại ở mọi câu thì một bài đọc hiện bốn lần liền.
        const group = groupOf(section, q.no);
        const prev = i > 0 ? groupOf(section, shown[i - 1].no) : undefined;
        const showGroup = group && group !== prev;
        const segments = questionAudio(section, q);
        const playingThis =
          audio.playing && audio.segment !== null && segments.length > 0 && audio.segment[1] === segments[segments.length - 1][1];
        const chosen = attempt.answers[qKey(section.id, q.no)];
        return (
          <div key={q.no} className="flex flex-col gap-3 border-t border-border pt-5">
            {showGroup && <GroupBlock exam={exam} group={group} />}
            <p className="flex items-baseline gap-2">
              <span className="font-semibold tabular-nums">Câu {q.no}</span>
              <span
                className={cn(
                  "text-xs",
                  chosen === q.answer
                    ? "text-emerald-700 dark:text-emerald-400"
                    : chosen
                      ? "text-red-700 dark:text-red-400"
                      : "text-muted-foreground"
                )}
              >
                {chosen === q.answer ? "đúng" : chosen ? "sai" : "bỏ trống"}
              </span>
            </p>
            <PromptView exam={exam} question={q} />
            {segments.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                className="self-start"
                onClick={() => (playingThis ? audio.pause() : audio.playSegments(segments))}
              >
                {playingThis ? <Pause className="size-4" aria-hidden /> : <Play className="size-4" aria-hidden />}
                {playingThis ? "Tạm dừng" : "Nghe lại câu này"}
              </Button>
            )}
            <OptionList exam={exam} question={q} value={chosen} onChange={() => {}} reveal disabled />
            <Explanation question={q} />
          </div>
        );
      })}
    </section>
  );
}
