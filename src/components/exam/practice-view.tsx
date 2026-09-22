"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { ChevronLeft, ChevronRight, Pause, Play, RotateCcw, Volume2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  examAudioUrl,
  groupOf,
  qKey,
  questionAudio,
  type Exam,
  type ExamSection,
  type SectionId,
} from "@/lib/exams";
import { useExamStore } from "@/lib/exam-store";
import { useActivityStore } from "@/lib/activity-store";
import { useActiveTime } from "@/lib/use-study-tracker";
import { GroupBlock, PromptView } from "@/components/exam/exam-content";
import {
  ActionBar,
  GridLegend,
  KeyHint,
  PracticeHeader,
  QuestionGrid,
  QuestionGridSheet,
  Verdict,
  type GridItem,
} from "@/components/exam/exam-chrome";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";
import { WritingPractice } from "@/components/exam/writing-practice";

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
  const section = exam.sections.find((s) => s.id === sectionId) ?? exam.sections[0];
  if (section.writing) return <WritingPractice exam={exam} section={section} initialNo={initialNo} />;
  return <ChoicePractice exam={exam} section={section} initialNo={initialNo} />;
}

/** Luyện câu trắc nghiệm (nghe, đọc). */
function ChoicePractice({
  exam,
  section,
  initialNo,
}: {
  exam: Exam;
  section: ExamSection;
  initialNo?: number;
}) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const questions = section.questions;
  const key = useCallback((no: number) => qKey(section.id, no), [section.id]);
  const [index, setIndex] = useState(() => {
    const i = questions.findIndex((q) => q.no === initialNo);
    return i >= 0 ? i : 0;
  });
  const q = questions[index];
  const group = groupOf(section, q.no);
  const isLast = index === questions.length - 1;

  const practice = useExamStore((s) => s.practice[exam.id]);
  const setAnswer = useExamStore((s) => s.setPracticeAnswer);
  const check = useExamStore((s) => s.checkPractice);
  const reset = useExamStore((s) => s.resetPractice);
  const recordQuizCheck = useActivityStore((s) => s.recordQuizCheck);

  const answers = isClient ? (practice?.answers ?? {}) : {};
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const value = answers[key(q.no)];
  const isChecked = checked.has(key(q.no));
  const isRight = isChecked && value === q.answer;

  const audioSrc = section.audio ? examAudioUrl(exam, section.audio) : undefined;
  const hasMarks = section.questions.some((x) => x.audio);
  const audio = useExamAudio(hasMarks ? audioSrc : undefined);
  const [fullPlaying, setFullPlaying] = useState(false);
  // Đang nghe thì vẫn là đang học, dù không chạm gì — xem `useActiveTime`.
  useActiveTime(() => audio.playing || fullPlaying);

  const grid: GridItem[] = questions.map((x) => {
    const done = checked.has(key(x.no));
    const right = done && answers[key(x.no)] === x.answer;
    return {
      no: x.no,
      state: done ? (right ? "right" : "wrong") : answers[key(x.no)] !== undefined ? "answered" : "idle",
      label: `Câu ${x.no}${done ? (right ? ", đúng" : ", sai") : ""}`,
    };
  });
  const doneCount = grid.filter((x) => x.state === "right" || x.state === "wrong").length;
  const rightCount = grid.filter((x) => x.state === "right").length;

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
    check(exam.id, key(q.no));
    recordQuizCheck(value === q.answer);
  }, [value, isChecked, check, exam.id, key, q.no, q.answer, recordQuizCheck]);

  // Phím tắt: 1–4 chọn, Enter kiểm tra (đã kiểm tra thì sang câu), ← → sang câu.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "AUDIO") return;
      if (["1", "2", "3", "4"].includes(e.key)) setAnswer(exam.id, key(q.no), Number(e.key));
      else if (e.key === "Enter") {
        if (isChecked) go(index + 1);
        else doCheck();
      } else if (e.key === "ArrowRight") go(index + 1);
      else if (e.key === "ArrowLeft") go(index - 1);
      else return;
      e.preventDefault();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [exam.id, key, q.no, index, isChecked, doCheck, go, setAnswer]);

  const segments = useMemo(() => questionAudio(section, q), [section, q]);
  const playingThis =
    audio.playing &&
    audio.segment !== null &&
    segments.length > 0 &&
    audio.segment[1] === segments[segments.length - 1][1];

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6 pb-24 sm:pb-12">
      <PracticeHeader
        exam={exam}
        section={section}
        done={doneCount}
        total={questions.length}
        summary={`${doneCount}/${questions.length} câu · đúng ${rightCount}`}
      />

      {section.audio && !hasMarks && audioSrc && (
        // Đề chưa chia theo từng câu: nghe cả bài, tua tới đoạn cần nghe.
        <div className="flex flex-col gap-1.5">
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

      <article className="flex flex-col gap-4">
        <h1 className="flex items-baseline gap-2">
          <span className="text-xl font-semibold tabular-nums">Câu {q.no}</span>
          <span className="text-sm text-muted-foreground">{q.points} điểm</span>
        </h1>

        {group && <GroupBlock exam={exam} group={group} />}

        <PromptView exam={exam} question={q} />

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
                <Volume2 className="size-4" aria-hidden />
                Lời chỉ dẫn
              </Button>
            )}
          </div>
        )}

        <OptionList
          exam={exam}
          question={q}
          value={value}
          onChange={(c) => setAnswer(exam.id, key(q.no), c)}
          reveal={isChecked}
        />

        {isChecked && (
          <Verdict right={isRight}>
            <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
              <span>
                {isRight
                  ? `Đúng rồi — +${q.points} điểm.`
                  : `Chưa đúng — đáp án là ${"①②③④"[q.answer - 1]}. ${section.audio ? "Nghe lại" : "Đọc lại"} rồi thử hiểu vì sao nhé.`}
              </span>
              <button
                type="button"
                onClick={() => reset(exam.id, [key(q.no)])}
                className="inline-flex items-center gap-1 text-xs font-medium underline-offset-2 hover:underline"
              >
                <RotateCcw className="size-3" aria-hidden />
                Làm lại
              </button>
            </div>
          </Verdict>
        )}
      </article>

      <ActionBar>
        <Button variant="ghost" size="icon-lg" onClick={() => go(index - 1)} disabled={index === 0} aria-label="Câu trước">
          <ChevronLeft className="size-5" aria-hidden />
        </Button>
        <QuestionGridSheet items={grid} current={index} onPick={go} count={`${doneCount}/${questions.length}`} />
        <div className="flex flex-1 justify-end gap-2 sm:flex-none">
          {isChecked ? (
            <Button onClick={() => go(index + 1)} disabled={isLast} size="lg" className="min-w-28">
              {isLast ? "Câu cuối" : "Câu sau"}
              {!isLast && <ChevronRight className="size-4" aria-hidden />}
            </Button>
          ) : (
            <Button onClick={doCheck} disabled={value === undefined} size="lg" className="min-w-28">
              Kiểm tra
            </Button>
          )}
        </div>
        <Button variant="ghost" size="icon-lg" onClick={() => go(index + 1)} disabled={isLast} aria-label="Câu sau">
          <ChevronRight className="size-5" aria-hidden />
        </Button>
      </ActionBar>

      <section aria-label="Bảng câu" className="hidden flex-col gap-3 border-t border-border pt-5 sm:flex">
        <QuestionGrid items={grid} current={index} onPick={go} />
        <div className="flex flex-wrap items-center justify-between gap-2">
          <GridLegend />
          <KeyHint>
            Phím tắt: <kbd>1</kbd>–<kbd>4</kbd> chọn, <kbd>Enter</kbd> kiểm tra / sang câu, <kbd>←</kbd>{" "}
            <kbd>→</kbd> chuyển câu.
          </KeyHint>
        </div>
      </section>
    </div>
  );
}
