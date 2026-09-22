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
  isPointsOnly,
  questionAudio,
  scoreExam,
  sectionVi,
  type Exam,
  type ExamSection,
} from "@/lib/exams";
import { useExamStore, type MockAttempt } from "@/lib/exam-store";
import { ContentView, GroupBlock } from "@/components/exam/exam-content";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";

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
  const [onlyWrong, setOnlyWrong] = useState(true);
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

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8 pb-16">
      <Link
        href={`/exam/${exam.id}`}
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        {examTitle(exam)}
      </Link>

      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-6">
        <p className="text-sm text-muted-foreground">
          Kết quả thi thử · {new Date(attempt.finishedAt!).toLocaleDateString("vi-VN")} · làm trong{" "}
          {duration(attempt)}
        </p>
        <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
          <span className="text-4xl font-semibold tabular-nums">
            {result.score}
            <span className="text-lg font-normal text-muted-foreground">/{result.max}</span>
          </span>
          <span
            className={cn(
              "rounded-full px-3 py-1 text-sm font-medium",
              result.level ? "bg-primary text-primary-foreground" : "bg-background text-muted-foreground"
            )}
          >
            {result.level ? `Đạt ${exam.level} ${result.level}` : "Chưa đạt cấp nào"}
          </span>
        </div>
        <ul className="flex flex-col gap-2">
          {result.sections.map((s) => (
            <li key={s.id} className="grid grid-cols-[3.5rem_1fr_auto] items-center gap-3 text-sm">
              <span>{sectionVi(s.id)}</span>
              <span className="h-1.5 overflow-hidden rounded-full bg-background">
                <span className="block h-full rounded-full bg-primary" style={{ width: `${(s.score / s.max) * 100}%` }} />
              </span>
              <span className="tabular-nums text-muted-foreground">
                {s.score}/{s.max} điểm · đúng {s.correct}/{s.total}
              </span>
            </li>
          ))}
        </ul>
        <p className="text-xs text-muted-foreground">
          {exam.level === "TOPIK I"
            ? "Ngưỡng TOPIK I: Cấp 1 từ 80 điểm, Cấp 2 từ 140 điểm."
            : "Ngưỡng TOPIK II: Cấp 3 từ 120, Cấp 4 từ 150, Cấp 5 từ 190, Cấp 6 từ 230 điểm."}
        </p>
        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link href={`/exam/${exam.id}/mock`}>Thi lại</Link>
          </Button>
          <Button asChild variant="outline">
            <Link href={`/exam/${exam.id}/practice`}>Luyện từng câu</Link>
          </Button>
        </div>
      </section>

      <div className="flex items-center justify-between gap-3">
        <h2 className="text-base font-semibold">Xem lại bài làm</h2>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={onlyWrong} onChange={(e) => setOnlyWrong(e.target.checked)} />
          Chỉ câu sai
        </label>
      </div>
      {exam.sections.map((s) => (
        <ReviewSection key={s.id} exam={exam} section={s} attempt={attempt} onlyWrong={onlyWrong} />
      ))}
    </div>
  );
}

function ReviewSection({
  exam,
  section,
  attempt,
  onlyWrong,
}: {
  exam: Exam;
  section: ExamSection;
  attempt: MockAttempt;
  onlyWrong: boolean;
}) {
  const audio = useExamAudio(section.audio ? examAudioUrl(exam, section.audio) : undefined);
  const shown = section.questions.filter((q) => !onlyWrong || attempt.answers[q.no] !== q.answer);
  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-medium text-muted-foreground">
        {sectionVi(section.id)} · {shown.length} câu
      </h3>
      {shown.length === 0 && <p className="text-sm text-muted-foreground">Không có câu nào sai. Tuyệt vời!</p>}
      {shown.map((q, i) => {
        // Khối chỉ dẫn chỉ hiện ở câu ĐẦU của khối trong danh sách đang xem —
        // lặp lại ở mọi câu thì [1~4] xuất hiện bốn lần liền nhau.
        const group = groupOf(section, q.no);
        const prev = i > 0 ? groupOf(section, shown[i - 1].no) : undefined;
        const showGroup = group && group !== prev;
        const segments = questionAudio(section, q);
        const playingThis =
          audio.playing && audio.segment !== null && segments.length > 0 && audio.segment[1] === segments[segments.length - 1][1];
        const chosen = attempt.answers[q.no];
        return (
          <div key={q.no} className="flex flex-col gap-3 rounded-xl border border-border p-3">
            <p className="text-sm font-medium">
              Câu {q.no}{" "}
              <span className="font-normal text-muted-foreground">
                · {chosen === q.answer ? `đúng, +${q.points} điểm` : chosen ? "sai" : "bỏ trống"}
              </span>
            </p>
            {showGroup && <GroupBlock exam={exam} group={group} />}
            {!isPointsOnly(q.prompt) && <ContentView exam={exam} content={q.prompt} alt={`Câu ${q.no}`} />}
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
