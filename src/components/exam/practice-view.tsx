"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowLeft, ChevronLeft, ChevronRight, Pause, Play, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  examAudioUrl,
  examTitle,
  groupOf,
  isPointsOnly,
  questionAudio,
  sectionVi,
  type Exam,
  type SectionId,
} from "@/lib/exams";
import { useExamStore } from "@/lib/exam-store";
import { useActivityStore } from "@/lib/activity-store";
import { useActiveTime } from "@/lib/use-study-tracker";
import { ContentView, GroupBlock } from "@/components/exam/exam-content";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";

const NO_SUBSCRIBE = () => () => {};

/**
 * Luyện tập: từng câu một, không tính giờ, chấm ngay khi bấm "Kiểm tra".
 *
 * Nghe: đề nào đã đo mốc thời gian từng câu thì có nút "Nghe câu N" phát
 * đúng đoạn của câu; đề chưa đo thì hiện trình phát cả bài (tua được).
 *
 * Bài làm lưu trong `exam-store` (theo đề), nên đóng trang mở lại vẫn thấy
 * câu nào đã làm, đúng hay sai.
 */
export function PracticeView({
  exam,
  sectionId,
  initialNo,
}: {
  exam: Exam;
  sectionId: SectionId;
  initialNo?: number;
}) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const section = exam.sections.find((s) => s.id === sectionId) ?? exam.sections[0];
  const questions = section.questions;
  const [index, setIndex] = useState(() => {
    const i = questions.findIndex((q) => q.no === initialNo);
    return i >= 0 ? i : 0;
  });
  const q = questions[index];
  const group = groupOf(section, q.no);

  const practice = useExamStore((s) => s.practice[exam.id]);
  const setAnswer = useExamStore((s) => s.setPracticeAnswer);
  const check = useExamStore((s) => s.checkPractice);
  const reset = useExamStore((s) => s.resetPractice);
  const recordQuizCheck = useActivityStore((s) => s.recordQuizCheck);

  const answers = isClient ? (practice?.answers ?? {}) : {};
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const value = answers[q.no];
  const isChecked = checked.has(q.no);
  const isRight = isChecked && value === q.answer;

  const audioSrc = section.audio ? examAudioUrl(exam, section.audio) : undefined;
  const hasMarks = section.questions.some((x) => x.audio);
  const audio = useExamAudio(hasMarks ? audioSrc : undefined);
  const [fullPlaying, setFullPlaying] = useState(false);
  // Đang nghe thì vẫn là đang học, dù không chạm gì — xem `useActiveTime`.
  useActiveTime(() => audio.playing || fullPlaying);

  const doneCount = questions.filter((x) => checked.has(x.no)).length;
  const rightCount = questions.filter((x) => checked.has(x.no) && answers[x.no] === x.answer).length;

  const go = useCallback(
    (to: number) => {
      audio.pause();
      setIndex(Math.max(0, Math.min(questions.length - 1, to)));
      window.scrollTo({ top: 0 });
    },
    [audio, questions.length]
  );

  const doCheck = useCallback(() => {
    if (value === undefined || isChecked) return;
    check(exam.id, q.no);
    recordQuizCheck(value === q.answer);
  }, [value, isChecked, check, exam.id, q.no, q.answer, recordQuizCheck]);

  // Phím tắt: 1–4 chọn, Enter kiểm tra, ← → sang câu.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "AUDIO") return;
      if (["1", "2", "3", "4"].includes(e.key)) setAnswer(exam.id, q.no, Number(e.key));
      else if (e.key === "Enter") doCheck();
      else if (e.key === "ArrowRight") go(index + 1);
      else if (e.key === "ArrowLeft") go(index - 1);
      else return;
      e.preventDefault();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [exam.id, q.no, index, doCheck, go, setAnswer]);

  const segments = useMemo(() => questionAudio(section, q), [section, q]);
  const playingThis =
    audio.playing &&
    audio.segment !== null &&
    segments.length > 0 &&
    audio.segment[1] === segments[segments.length - 1][1];

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-5 pb-24">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Link
          href={`/exam/${exam.id}`}
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="size-4" aria-hidden />
          {examTitle(exam)}
        </Link>
        <div className="flex gap-1 rounded-lg bg-muted p-1 text-sm">
          {exam.sections.map((s) => (
            <Link
              key={s.id}
              href={`/exam/${exam.id}/practice?section=${s.id}`}
              className={cn(
                "rounded-md px-3 py-1",
                s.id === section.id ? "bg-background font-medium shadow-sm" : "text-muted-foreground"
              )}
            >
              {sectionVi(s.id)}
            </Link>
          ))}
        </div>
      </div>

      {section.audio && !hasMarks && audioSrc && (
        // Đề chưa chia theo từng câu: nghe cả bài, tua tới đoạn cần nghe.
        <div className="flex flex-col gap-1.5 rounded-xl border border-border p-3">
          <audio
            controls
            preload="none"
            src={audioSrc}
            className="w-full"
            onPlay={() => setFullPlaying(true)}
            onPause={() => setFullPlaying(false)}
            onEnded={() => setFullPlaying(false)}
          />
          <p className="text-xs text-muted-foreground">
            Bài nghe cả phần — kéo thanh thời gian tới đoạn của câu cần nghe.
          </p>
        </div>
      )}

      <div className="flex items-baseline justify-between gap-3">
        <h1 className="text-lg font-semibold">
          Câu {q.no} <span className="text-sm font-normal text-muted-foreground">· {q.points} điểm</span>
        </h1>
        <span className="text-sm text-muted-foreground tabular-nums">
          Đã làm {doneCount}/{questions.length} · đúng {rightCount}
        </span>
      </div>

      {group && <GroupBlock exam={exam} group={group} />}

      {!isPointsOnly(q.prompt) && <ContentView exam={exam} content={q.prompt} alt={`Câu ${q.no}`} />}

      {segments.length > 0 && (
        <div className="flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            onClick={() => (playingThis ? audio.pause() : audio.playSegments(segments))}
          >
            {playingThis ? <Pause className="size-4" aria-hidden /> : <Play className="size-4" aria-hidden />}
            {playingThis ? "Tạm dừng" : `Nghe câu ${q.no}`}
          </Button>
          {group?.audio && group.from === q.no && (
            <Button variant="ghost" size="sm" onClick={() => audio.playSegments([group.audio!])}>
              Nghe lời chỉ dẫn
            </Button>
          )}
        </div>
      )}

      <OptionList
        exam={exam}
        question={q}
        value={value}
        onChange={(c) => setAnswer(exam.id, q.no, c)}
        reveal={isChecked}
      />

      {isChecked ? (
        <p
          role="status"
          className={cn(
            "rounded-lg px-4 py-3 text-sm font-medium",
            isRight
              ? "bg-emerald-50 text-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300"
              : "bg-red-50 text-red-800 dark:bg-red-950/40 dark:text-red-300"
          )}
        >
          {isRight
            ? `Đúng rồi! +${q.points} điểm`
            : `Chưa đúng — đáp án là ${"①②③④"[q.answer - 1]}. ${section.audio ? "Nghe lại rồi thử hiểu vì sao nhé." : "Đọc lại rồi thử hiểu vì sao nhé."}`}
        </p>
      ) : (
        <Button onClick={doCheck} disabled={value === undefined} className="self-start">
          Kiểm tra
        </Button>
      )}

      <div className="flex items-center justify-between gap-3 border-t border-border pt-4">
        <Button variant="ghost" onClick={() => go(index - 1)} disabled={index === 0}>
          <ChevronLeft className="size-4" aria-hidden />
          Câu trước
        </Button>
        {isChecked && (
          <Button variant="ghost" size="sm" onClick={() => reset(exam.id, [q.no])}>
            <RotateCcw className="size-3.5" aria-hidden />
            Làm lại câu này
          </Button>
        )}
        <Button variant="ghost" onClick={() => go(index + 1)} disabled={index === questions.length - 1}>
          Câu sau
          <ChevronRight className="size-4" aria-hidden />
        </Button>
      </div>

      {/* Bảng số câu: xanh = đúng, đỏ = sai, đậm = đã chọn chưa kiểm tra. */}
      <nav aria-label="Chọn câu" className="flex flex-wrap gap-1.5">
        {questions.map((x, i) => {
          const done = checked.has(x.no);
          const right = done && answers[x.no] === x.answer;
          return (
            <button
              key={x.no}
              type="button"
              onClick={() => go(i)}
              aria-current={i === index ? "step" : undefined}
              aria-label={`Câu ${x.no}${done ? (right ? ", đúng" : ", sai") : ""}`}
              className={cn(
                "flex size-9 items-center justify-center rounded-md border text-xs tabular-nums",
                done
                  ? right
                    ? "border-emerald-600 bg-emerald-600 text-white"
                    : "border-red-600 bg-red-600 text-white"
                  : answers[x.no] !== undefined
                    ? "border-foreground/40 bg-muted"
                    : "border-border",
                i === index && "ring-2 ring-primary ring-offset-2 ring-offset-background"
              )}
            >
              {x.no}
            </button>
          );
        })}
      </nav>
      <p className="text-xs text-muted-foreground">
        Phím tắt: <kbd>1</kbd>–<kbd>4</kbd> chọn đáp án, <kbd>Enter</kbd> kiểm tra, <kbd>←</kbd>{" "}
        <kbd>→</kbd> sang câu.
      </p>
    </div>
  );
}
