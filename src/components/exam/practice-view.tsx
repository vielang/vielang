"use client";

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import { ChevronLeft, ChevronRight, RotateCcw, Volume2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  examAudioUrl,
  groupAudio,
  qKey,
  type Exam,
  type ExamQuestion,
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
import { BlockPlayer } from "@/components/exam/block-player";
import { WritingPractice } from "@/components/exam/writing-practice";

const NO_SUBSCRIBE = () => () => {};
const MARKS = "①②③④";

/**
 * Luyện tập: không tính giờ, chấm ngay khi bấm "Kiểm tra".
 *
 * Trắc nghiệm luyện THEO KHỐI "※ [a~b]" như tờ đề: lời chỉ dẫn, câu mẫu,
 * đoạn văn dùng chung hiện một lần, các câu của khối xếp bên dưới, một nút
 * kiểm tra cả khối. Phần viết luyện từng câu (mỗi câu một bài).
 *
 * Nghe: mỗi khối MỘT audio (lời chỉ dẫn + lời đọc các câu, bỏ khoảng dừng
 * trả lời) có thanh điều khiển — xem `BlockPlayer`. Đề chưa đo mốc thời gian
 * thì hiện trình phát cả bài (tua được).
 *
 * Bài làm lưu trong `exam-store` theo từng câu, nên đóng trang mở lại vẫn
 * thấy câu nào đã làm, đúng hay sai.
 */
export function PracticeView({ exam, sectionId, initialNo }: { exam: Exam; sectionId: SectionId; initialNo?: number }) {
  const section = exam.sections.find((s) => s.id === sectionId) ?? exam.sections[0];
  if (section.writing) return <WritingPractice exam={exam} section={section} initialNo={initialNo} />;
  return <GroupPractice exam={exam} section={section} initialNo={initialNo} />;
}

/** Luyện trắc nghiệm (nghe, đọc) theo khối. */
function GroupPractice({ exam, section, initialNo }: { exam: Exam; section: ExamSection; initialNo?: number }) {
  const isClient = useSyncExternalStore(
    NO_SUBSCRIBE,
    () => true,
    () => false
  );
  const { groups, questions } = section;
  const key = useCallback((no: number) => qKey(section.id, no), [section.id]);
  const groupIndexOf = useCallback(
    (no: number) =>
      Math.max(
        0,
        groups.findIndex((g) => no >= g.from && no <= g.to)
      ),
    [groups]
  );
  const [gi, setGi] = useState(() => (initialNo ? groupIndexOf(initialNo) : 0));
  const group = groups[gi];
  const items = questions.filter((q) => q.no >= group.from && q.no <= group.to);
  const isLastGroup = gi === groups.length - 1;
  // Câu cần cuộn tới sau khi đổi khối (bấm số câu trong bảng, link ?q=).
  const scrollTo = useRef<number | null>(initialNo ?? null);

  const practice = useExamStore((s) => s.practice[exam.id]);
  const setAnswer = useExamStore((s) => s.setPracticeAnswer);
  const check = useExamStore((s) => s.checkPractice);
  const reset = useExamStore((s) => s.resetPractice);
  const recordQuizCheck = useActivityStore((s) => s.recordQuizCheck);

  const answers = isClient ? (practice?.answers ?? {}) : {};
  const checked = new Set(isClient ? (practice?.checked ?? []) : []);
  const isChecked = (q: ExamQuestion) => checked.has(key(q.no));
  const answerOf = (q: ExamQuestion) => answers[key(q.no)];

  const pending = items.filter((q) => !isChecked(q));
  const toCheck = pending.filter((q) => answerOf(q) !== undefined);
  const groupDone = pending.length === 0;
  const groupRight = items.filter((q) => isChecked(q) && answerOf(q) === q.answer);
  const chosenCount = items.filter((q) => isChecked(q) || answerOf(q) !== undefined).length;

  // Câu đang "cầm bút": phím 1–4 chọn cho câu này. Mặc định là câu chưa
  // chọn đầu tiên của khối; bấm vào một câu thì chuyển sang câu đó.
  const [activeNo, setActiveNo] = useState<number | null>(null);
  const firstOpen = items.find((q) => !isChecked(q) && answerOf(q) === undefined) ?? pending[0];
  const active = items.find((q) => q.no === activeNo && !isChecked(q)) ?? firstOpen;

  const audioSrc = section.audio ? examAudioUrl(exam, section.audio) : undefined;
  const hasMarks = questions.some((x) => x.audio);
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
  const currentIndex = questions.findIndex((q) => q.no === (active ?? items[0]).no);

  const goGroup = useCallback(
    (to: number, no?: number) => {
      audio.pause();
      setActiveNo(null);
      scrollTo.current = no ?? null;
      setGi(Math.max(0, Math.min(groups.length - 1, to)));
      if (no === undefined) window.scrollTo({ top: 0 });
    },
    [audio, groups.length]
  );
  const goQuestion = (index: number) => {
    const no = questions[index].no;
    goGroup(groupIndexOf(no), no);
    setActiveNo(no);
  };

  // Tới đúng câu được chọn trong bảng (câu đầu khối thì lên đầu trang).
  useEffect(() => {
    const no = scrollTo.current;
    if (no === null) return;
    scrollTo.current = null;
    if (no === group.from) window.scrollTo({ top: 0 });
    else document.getElementById(`q-${no}`)?.scrollIntoView({ block: "center" });
  }, [gi, group.from]);

  const checkGroup = () => {
    for (const q of toCheck) {
      check(exam.id, key(q.no));
      recordQuizCheck(answerOf(q) === q.answer);
    }
  };

  const choose = (q: ExamQuestion, choice: number) => {
    setAnswer(exam.id, key(q.no), choice);
    setActiveNo(q.no);
  };

  // Phím tắt: 1–4 chọn cho câu đang cầm bút (rồi sang câu chưa chọn kế
  // tiếp), Enter kiểm tra khối (đã kiểm tra thì sang khối sau), ← → đổi khối.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "AUDIO") return;
      if (["1", "2", "3", "4"].includes(e.key)) {
        if (!active) return;
        setAnswer(exam.id, key(active.no), Number(e.key));
        const next = items.find((q) => q.no > active.no && !checked.has(key(q.no)) && answers[key(q.no)] === undefined);
        setActiveNo(next?.no ?? active.no);
      } else if (e.key === "Enter") {
        if (tag === "BUTTON" || tag === "A") return; // Enter trên nút là bấm nút đó
        if (groupDone) goGroup(gi + 1);
        else checkGroup();
      } else if (e.key === "ArrowRight") goGroup(gi + 1);
      else if (e.key === "ArrowLeft") goGroup(gi - 1);
      else return;
      e.preventDefault();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Một audio cho cả khối: lời chỉ dẫn + lời đọc từng câu, bỏ khoảng dừng trả lời.
  const blockSegments = groupAudio(section, group);
  // Câu đang được đọc (tô trong khối khi đang nghe).
  const playingNo = audio.playing
    ? items.find((q) => {
        const seg = q.replay ?? q.audio;
        return seg && audio.time >= seg[0] && audio.time < seg[1];
      })?.no
    : undefined;

  const range = group.from === group.to ? `Câu ${group.from}` : `Câu ${group.from}–${group.to}`;
  const points = items.reduce((n, q) => n + q.points, 0);

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

      <section className="flex flex-col gap-4" aria-label={range}>
        <h1 className="flex items-baseline gap-2">
          <span className="text-xl font-semibold tabular-nums">{range}</span>
          <span className="text-sm text-muted-foreground">{points} điểm</span>
        </h1>

        {blockSegments.length > 0 && (
          <BlockPlayer
            key={`${section.id}-${group.from}`}
            audio={audio}
            segments={blockSegments}
            label={`Bài nghe ${range}`}
          />
        )}

        <GroupBlock exam={exam} group={group} showRange={false} />

        <div className="flex flex-col gap-2">
          {items.map((q) => (
            <QuestionItem
              key={q.no}
              exam={exam}
              question={q}
              value={answerOf(q)}
              checked={isChecked(q)}
              active={items.length > 1 && active?.no === q.no}
              playing={items.length > 1 && playingNo === q.no}
              onChoose={(c) => choose(q, c)}
              onFocus={() => setActiveNo(q.no)}
            />
          ))}
        </div>

        {groupDone && (
          <Verdict right={groupRight.length === items.length}>
            <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
              <span>
                {items.length === 1
                  ? groupRight.length
                    ? `Đúng rồi — +${points} điểm.`
                    : `Chưa đúng — đáp án là ${MARKS[items[0].answer - 1]}. ${section.audio ? "Nghe lại" : "Đọc lại"} rồi thử hiểu vì sao nhé.`
                  : `Đúng ${groupRight.length}/${items.length} câu · +${groupRight.reduce((n, q) => n + q.points, 0)} điểm.`}
              </span>
              <button
                type="button"
                onClick={() =>
                  reset(
                    exam.id,
                    items.map((q) => key(q.no))
                  )
                }
                className="inline-flex items-center gap-1 text-xs font-medium underline-offset-2 hover:underline"
              >
                <RotateCcw className="size-3" aria-hidden />
                {items.length === 1 ? "Làm lại" : "Làm lại khối"}
              </button>
            </div>
          </Verdict>
        )}
      </section>

      <ActionBar>
        <Button
          variant="ghost"
          size="icon-lg"
          onClick={() => goGroup(gi - 1)}
          disabled={gi === 0}
          aria-label="Khối trước"
        >
          <ChevronLeft className="size-5" aria-hidden />
        </Button>
        <QuestionGridSheet
          items={grid}
          current={currentIndex}
          onPick={goQuestion}
          count={`${doneCount}/${questions.length}`}
        />
        <div className="flex flex-1 justify-end gap-2 sm:flex-none">
          {groupDone ? (
            <Button size="lg" onClick={() => goGroup(gi + 1)} disabled={isLastGroup} className="min-w-32">
              {isLastGroup ? "Hết phần" : "Khối sau"}
              {!isLastGroup && <ChevronRight className="size-4" aria-hidden />}
            </Button>
          ) : (
            <Button size="lg" onClick={checkGroup} disabled={toCheck.length === 0} className="min-w-32">
              Kiểm tra
              {items.length > 1 && (
                <span
                  className="tabular-nums opacity-70"
                  aria-label={`đã chọn ${chosenCount} trên ${items.length} câu`}
                >
                  {chosenCount}/{items.length}
                </span>
              )}
            </Button>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon-lg"
          onClick={() => goGroup(gi + 1)}
          disabled={isLastGroup}
          aria-label="Khối sau"
        >
          <ChevronRight className="size-5" aria-hidden />
        </Button>
      </ActionBar>
      {!groupDone && toCheck.length > 0 && toCheck.length < pending.length && (
        <p className="-mt-3 text-xs text-muted-foreground sm:text-right">
          Còn {pending.length - toCheck.length} câu chưa chọn — kiểm tra bây giờ thì chỉ chấm các câu đã chọn.
        </p>
      )}

      <section aria-label="Bảng câu" className="hidden flex-col gap-3 border-t border-border pt-5 sm:flex">
        <QuestionGrid items={grid} current={currentIndex} onPick={goQuestion} />
        <div className="flex flex-wrap items-center justify-between gap-2">
          <GridLegend />
          <KeyHint>
            Phím tắt: <kbd>1</kbd>–<kbd>4</kbd> chọn cho câu đang tô, <kbd>Enter</kbd> kiểm tra / sang khối,{" "}
            <kbd>←</kbd> <kbd>→</kbd> đổi khối.
          </KeyHint>
        </div>
      </section>
    </div>
  );
}

/** Một câu trong khối: số câu, đề, nút nghe riêng, bốn lựa chọn, kết quả. */
function QuestionItem({
  exam,
  question: q,
  value,
  checked,
  active,
  playing,
  onChoose,
  onFocus,
}: {
  exam: Exam;
  question: ExamQuestion;
  value: number | undefined;
  checked: boolean;
  active: boolean;
  playing: boolean;
  onChoose: (choice: number) => void;
  onFocus: () => void;
}) {
  const right = checked && value === q.answer;

  return (
    <div
      id={`q-${q.no}`}
      onFocusCapture={onFocus}
      className={cn(
        "-mx-3 flex scroll-mt-32 flex-col gap-3 rounded-xl border-l-2 px-3 py-3 transition-colors",
        playing ? "border-foreground bg-muted/60" : active ? "border-foreground/60 bg-muted/40" : "border-transparent"
      )}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="flex items-baseline gap-2">
          <span className="font-semibold tabular-nums">{q.no}.</span>
          <span className="text-xs text-muted-foreground">{q.points} điểm</span>
          {playing && (
            <span className="inline-flex items-center gap-1 text-xs font-medium">
              <Volume2 className="size-3" aria-hidden />
              đang đọc
            </span>
          )}
          {checked && (
            <span
              className={cn(
                "text-xs font-medium",
                right ? "text-emerald-700 dark:text-emerald-400" : "text-red-700 dark:text-red-400"
              )}
            >
              {right ? "· đúng" : value === undefined ? "· bỏ trống" : `· sai, đáp án ${MARKS[q.answer - 1]}`}
            </span>
          )}
        </p>
      </div>
      <PromptView exam={exam} question={q} />
      <OptionList exam={exam} question={q} value={value} onChange={onChoose} reveal={checked} disabled={checked} />
    </div>
  );
}
