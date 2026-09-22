"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowLeft, Pause, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  examAudioUrl,
  examTitle,
  groupOf,
  levelCuts,
  nextLevel,
  qKey,
  questionAudio,
  scoreExam,
  sectionVi,
  type Exam,
  type ExamSection,
} from "@/lib/exams";
import { useExamStore, type MockAttempt } from "@/lib/exam-store";
import { GroupBlock, PromptView } from "@/components/exam/exam-content";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";
import { LevelBadge, ProgressBar } from "@/components/exam/exam-chrome";
import { SelfGrade, WritingImage, WritingInput, hasWritten } from "@/components/exam/writing-parts";

const NO_SUBSCRIBE = () => () => {};

function duration(a: MockAttempt): string {
  if (!a.finishedAt) return "";
  const min = Math.round((Date.parse(a.finishedAt) - Date.parse(a.startedAt)) / 60_000);
  return `${min} phút`;
}

/** Kết quả một lượt thi thử + xem lại từng câu. */
export function ResultView({ exam, attemptId }: { exam: Exam; attemptId?: string }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
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

  const result = scoreExam(exam, attempt.answers, attempt.grades);
  const next = nextLevel(exam.level, result.score);
  const wrong = exam.sections.flatMap((s) =>
    s.questions.filter((q) => attempt.answers[qKey(s.id, q.no)] !== q.answer).map((q) => ({ section: s.id, no: q.no }))
  );
  const firstWrong = wrong[0];
  const wrongCount = wrong.length;

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8 pb-16">
      <Link
        href={`/exam/${exam.id}`}
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        {examTitle(exam)}
      </Link>

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
              <LevelBadge
                level={result.level ? `Đạt ${exam.level} ${result.level}` : null}
                emptyLabel="Chưa đạt cấp nào"
                className="px-3 py-1 text-sm"
              />
              {next && (
                <span className="text-xs text-muted-foreground">
                  Còn <b className="text-foreground tabular-nums">{next.need}</b> điểm nữa lên {next.level}
                </span>
              )}
            </span>
          </div>
        </div>

        <LevelLadder exam={exam} score={result.score} max={result.max} />

        {result.ungraded > 0 && (
          <p className="rounded-lg bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:bg-amber-950/40 dark:text-amber-200">
            Còn {result.ungraded} câu viết chưa tự chấm — điểm và cấp trên đây mới là tạm tính.{" "}
            <a href="#writing" className="font-medium underline underline-offset-2">
              Chấm phần viết
            </a>
          </p>
        )}

        <ul className="flex flex-col divide-y divide-border border-y border-border">
          {result.sections.map((s) => (
            <li key={s.id} className="grid grid-cols-[3.5rem_1fr_auto] items-center gap-3 py-3 text-sm">
              <span className="font-medium">{sectionVi(s.id)}</span>
              <ProgressBar value={s.score} max={s.max} />
              <span className="text-right tabular-nums">
                <b>{s.score}</b>
                <span className="text-muted-foreground">
                  /{s.max} · {s.id === "writing" ? "đã chấm" : "đúng"} {s.correct}/{s.total}
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
      {exam.sections.map((s) =>
        s.writing ? (
          <WritingReview key={s.id} exam={exam} section={s} attempt={attempt} />
        ) : (
          <ReviewSection key={s.id} exam={exam} section={s} attempt={attempt} filter={filter} />
        )
      )}
    </div>
  );
}

/**
 * Thang cấp: thanh 0 → điểm tối đa, vạch ở từng ngưỡng cấp, phần đã đạt tô
 * đậm — nhìn là biết mình đứng đâu và cách cấp sau bao xa.
 */
function LevelLadder({ exam, score, max }: { exam: Exam; score: number; max: number }) {
  const cuts = levelCuts(exam.level);
  const pct = (n: number) => `${(n / max) * 100}%`;
  return (
    <div className="flex flex-col gap-1.5" aria-label={`Thang cấp ${exam.level}`} role="img">
      <div className="relative h-2 rounded-full bg-muted">
        <div className="absolute inset-y-0 left-0 rounded-full bg-foreground/80" style={{ width: pct(Math.min(score, max)) }} />
        {cuts.map(([cut]) => (
          <span
            key={cut}
            className="absolute -top-1 h-4 w-0.5 -translate-x-1/2 rounded-full bg-background ring-1 ring-foreground/30"
            style={{ left: pct(cut) }}
            aria-hidden
          />
        ))}
      </div>
      <div className="relative h-8 text-[0.7rem] text-muted-foreground tabular-nums">
        {cuts.map(([cut, name]) => (
          <span
            key={cut}
            className={cn(
              "absolute flex -translate-x-1/2 flex-col items-center leading-tight",
              score >= cut && "font-medium text-foreground"
            )}
            style={{ left: pct(cut) }}
          >
            <span>{name}</span>
            <span>{cut}</span>
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
        // Khối chỉ dẫn chỉ hiện ở câu ĐẦU của khối trong danh sách đang xem —
        // lặp lại ở mọi câu thì [1~4] xuất hiện bốn lần liền nhau.
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
                {chosen === q.answer ? `đúng · +${q.points} điểm` : chosen ? `sai · ${q.points} điểm` : `bỏ trống · ${q.points} điểm`}
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
          </div>
        );
      })}
    </section>
  );
}

/**
 * Phần viết sau khi nộp: bài đã viết (không sửa được nữa), đáp án mẫu chính
 * thức, và thanh tự chấm từng câu — chấm đến đâu điểm tổng và cấp cập nhật
 * đến đó (cả trong danh sách các lần thi).
 */
function WritingReview({ exam, section, attempt }: { exam: Exam; section: ExamSection; attempt: MockAttempt }) {
  const gradeMock = useExamStore((s) => s.gradeMock);
  const texts = attempt.texts ?? {};
  const grades = attempt.grades ?? {};
  const grade = (no: number, points: number) => {
    const next = { ...grades, [no]: points };
    const r = scoreExam(exam, attempt.answers, next);
    gradeMock(attempt.id, no, points, r.score, r.level);
  };
  return (
    <section id="writing" className="flex scroll-mt-20 flex-col gap-4">
      <h3 className="text-sm font-medium text-muted-foreground">
        {sectionVi(section.id)} · {section.writing!.tasks.length} câu — tự chấm theo đáp án mẫu
      </h3>
      <details className="rounded-xl bg-muted/60 p-3">
        <summary className="cursor-pointer text-sm font-medium">Đáp án mẫu và tiêu chí chấm chính thức</summary>
        <div className="mt-3 flex flex-col gap-3">
          {section.writing!.modelAnswers.map((src, i) => (
            <WritingImage key={src} exam={exam} src={src} alt={`Đáp án mẫu phần viết, trang ${i + 1}`} />
          ))}
        </div>
      </details>
      {section.writing!.tasks.map((t) => (
        <div key={t.no} className="flex flex-col gap-3 border-t border-border pt-5">
          <p className="text-sm font-medium">
            Câu {t.no}{" "}
            <span className="font-normal text-muted-foreground">
              · {hasWritten(t, texts) ? `tối đa ${t.points} điểm` : "bỏ trống"}
            </span>
          </p>
          <details>
            <summary className="cursor-pointer text-sm text-muted-foreground">Xem đề</summary>
            <div className="mt-2">
              <WritingImage exam={exam} src={t.page} alt={`Đề câu ${t.no}`} />
            </div>
          </details>
          {hasWritten(t, texts) && <WritingInput task={t} texts={texts} onChange={() => {}} readOnly />}
          <SelfGrade task={t} value={grades[t.no]} onChange={(p) => grade(t.no, p)} />
        </div>
      ))}
    </section>
  );
}
