"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { Clock, Headphones, ListChecks, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import {
  examAudioUrl,
  examTitle,
  qKey,
  scoreExam,
  sectionCount,
  sectionVi,
  totalMinutes,
  type Exam,
  type ExamSection,
} from "@/lib/exams";
import { activeAttempt, useExamStore, type MockAttempt } from "@/lib/exam-store";
import { useActiveTime } from "@/lib/use-study-tracker";
import { GroupBlock, PromptView } from "@/components/exam/exam-content";
import { ProgressBar, SectionStepper } from "@/components/exam/exam-chrome";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";
import { useLeaveGuard } from "@/components/exam/use-leave-guard";
import { AnswerSheet, MobileAnswerSheet, jumpToQuestion } from "@/components/exam/answer-sheet";
import { useIsClient } from "@/lib/use-is-client";
import { BackLink } from "@/components/layout/back-link";

function mmss(ms: number): string {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/**
 * Thi thử: giống thi thật nhất có thể.
 *
 * - Tính giờ theo từng phần (TOEIC: nghe 45', đọc 75'). Hết giờ tự sang
 *   phần sau / tự nộp. Sang phần sau là không quay lại phần trước.
 * - Phần nghe phát LIỀN MẠCH một lần, không tạm dừng, không tua — như phòng
 *   thi. Câu đang phát được tô và cuộn tới.
 * - Tải lại trang giữa chừng thì làm tiếp: đồng hồ tính theo mốc hết giờ đã
 *   lưu, file nghe phát tiếp từ đúng chỗ lẽ ra đang phát.
 */
export function MockView({ exam }: { exam: Exam }) {
  const isClient = useIsClient();
  const attempts = useExamStore((s) => s.attempts);
  const attempt = isClient ? activeAttempt(attempts, exam.id) : undefined;
  const startMock = useExamStore((s) => s.startMock);

  if (!isClient) return null;
  if (!attempt) {
    return <MockIntro exam={exam} onStart={() => startMock(exam.id, exam.sections[0].minutes)} />;
  }
  return <MockRunning key={attempt.id} exam={exam} attempt={attempt} />;
}

function MockIntro({ exam, onStart }: { exam: Exam; onStart: () => void }) {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <BackLink href={`/exam/${exam.id}`}>{examTitle(exam)}</BackLink>
      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-6">
        <h1 className="text-xl font-semibold tracking-tight">Thi thử {examTitle(exam)}</h1>
        <ul className="flex flex-col gap-2 text-sm">
          {exam.sections.map((s) => (
            <li key={s.id} className="flex items-center gap-2">
              {s.audio ? <Headphones className="size-4" aria-hidden /> : <ListChecks className="size-4" aria-hidden />}
              <span className="font-medium">{sectionVi(s.id)}</span>
              <span className="text-muted-foreground">
                {sectionCount(s)} câu · {s.minutes} phút
              </span>
            </li>
          ))}
        </ul>
        <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
          <li>Tổng thời gian {totalMinutes(exam)} phút. Hết giờ phần nào là tự sang phần sau.</li>
          <li>Phần nghe phát liền một lần như phòng thi — không tạm dừng, không tua lại được.</li>
          <li>Đã sang phần sau thì không quay lại phần trước.</li>
          <li>Lỡ tải lại trang thì vẫn làm tiếp được, đồng hồ không dừng.</li>
          <li>Nộp bài xong có điểm quy đổi ước tính và lời giải thích từng câu.</li>
        </ul>
        <Button onClick={onStart} className="self-start">
          <Play className="size-4" aria-hidden />
          Bắt đầu thi
        </Button>
      </section>
    </div>
  );
}

function MockRunning({ exam, attempt }: { exam: Exam; attempt: MockAttempt }) {
  const router = useRouter();
  const setAnswer = useExamStore((s) => s.setMockAnswer);
  const nextSection = useExamStore((s) => s.nextMockSection);
  const finish = useExamStore((s) => s.finishMock);

  const section = exam.sections[attempt.sectionIndex];
  const isLast = attempt.sectionIndex === exam.sections.length - 1;
  const [now, setNow] = useState(() => Date.now());
  const [confirm, setConfirm] = useState(false);
  const remaining = attempt.deadline - now;
  // Rời trang giữa lượt thi thì hỏi lại — đồng hồ vẫn chạy khi đi chỗ khác.
  const leaveGuard = useLeaveGuard(!attempt.finishedAt, `/exam/${exam.id}`);

  const submitSection = useCallback(() => {
    setConfirm(false);
    if (isLast) {
      const result = scoreExam(exam, attempt.answers);
      finish(attempt.id, result.score);
      router.push(`/exam/${exam.id}/result?attempt=${attempt.id}`);
    } else {
      nextSection(attempt.id, exam.sections[attempt.sectionIndex + 1].minutes);
      window.scrollTo({ top: 0 });
    }
  }, [isLast, exam, attempt, finish, nextSection, router]);

  // Đồng hồ + hết giờ thì tự sang phần sau / tự nộp — như chuông hết giờ ở
  // phòng thi. Xét trong nhịp đồng hồ (không phải trong effect theo
  // `remaining`), và chỉ một lần cho mỗi phần: `fired` giữ theo mốc hết giờ.
  const submitRef = useRef(submitSection);
  const fired = useRef<number | null>(null);
  useEffect(() => {
    submitRef.current = submitSection;
  });
  useEffect(() => {
    const t = window.setInterval(() => {
      const current = Date.now();
      setNow(current);
      if (current >= attempt.deadline && fired.current !== attempt.deadline) {
        fired.current = attempt.deadline;
        submitRef.current();
      }
    }, 500);
    return () => window.clearInterval(t);
  }, [attempt.deadline]);

  const total = sectionCount(section);
  const answered = section.questions.filter((q) => attempt.answers[qKey(section.id, q.no)] !== undefined).length;
  const onAnswer = (no: number, choice: number) => setAnswer(attempt.id, qKey(section.id, no), choice);

  return (
    // Máy tính dùng cả bề ngang (đề + phiếu trả lời).
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-5 pb-6 lg:max-w-none lg:pb-12">
      <div className="sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-30 -mx-4 flex flex-col gap-2 border-b border-border bg-background/95 px-4 pt-2 pb-2.5 backdrop-blur">
        <div className="flex items-center justify-between gap-3">
          <div className="min-w-0">
            <p className="truncate text-sm font-medium">{examTitle(exam)}</p>
            <SectionStepper exam={exam} current={attempt.sectionIndex} />
          </div>
          <span
            className={cn(
              "inline-flex items-center gap-1 rounded-full px-3 py-1 text-sm font-semibold tabular-nums",
              remaining < 5 * 60_000 ? "bg-red-600 text-white" : "bg-muted"
            )}
            aria-label={`Còn ${mmss(remaining)}`}
          >
            <Clock className="size-4" aria-hidden />
            {mmss(remaining)}
          </span>
          <Button size="sm" onClick={() => setConfirm(true)}>
            {isLast ? "Nộp bài" : "Xong phần này"}
          </Button>
        </div>
        <div className="flex items-center gap-3">
          <ProgressBar value={answered} max={total} className="flex-1" />
          <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
            đã chọn {answered}/{total}
          </span>
        </div>
      </div>

      {/* Máy tính: đề bên trái, phiếu trả lời dính bên phải như phiếu tô
          đáp án ở phòng thi. Điện thoại: phiếu kéo lên từ đáy màn hình. */}
      <div className="lg:grid lg:grid-cols-[minmax(0,1fr)_13rem] lg:gap-10">
        <SectionQuestions key={section.id} exam={exam} section={section} attempt={attempt} onAnswer={onAnswer} />
        <aside aria-label="Phiếu trả lời" className="hidden lg:block">
          <div className="sticky top-40 flex max-h-[calc(100vh-11rem)] flex-col gap-2">
            <p className="text-xs font-medium text-muted-foreground">Phiếu trả lời · {sectionVi(section.id)}</p>
            <div className="overflow-y-auto pr-1">
              <AnswerSheet
                section={section}
                answers={attempt.answers}
                onAnswer={onAnswer}
                onJump={jumpToQuestion}
              />
            </div>
          </div>
        </aside>
      </div>

      <MobileAnswerSheet section={section} answers={attempt.answers} onAnswer={onAnswer} />

      <Dialog open={leaveGuard.pending} onOpenChange={(open) => !open && leaveGuard.stay()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rời khỏi bài thi thử?</DialogTitle>
            <DialogDescription>
              Bài làm đã được lưu, nhưng đồng hồ vẫn chạy — còn {mmss(remaining)} cho phần {sectionVi(section.id)}.
              Hết giờ khi bạn đang ở trang khác thì phần này sẽ tự nộp.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={leaveGuard.leave}>
              Vẫn thoát
            </Button>
            <Button onClick={leaveGuard.stay}>Ở lại làm bài</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={confirm} onOpenChange={setConfirm}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{isLast ? "Nộp bài?" : `Kết thúc phần ${sectionVi(section.id)}?`}</DialogTitle>
            <DialogDescription>
              {answered < total ? `Còn ${total - answered} câu chưa chọn. ` : ""}
              {isLast
                ? "Nộp rồi thì không sửa được nữa."
                : `Sang phần ${sectionVi(exam.sections[attempt.sectionIndex + 1].id)} rồi thì không quay lại được.`}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirm(false)}>
              Làm tiếp
            </Button>
            <Button onClick={submitSection}>{isLast ? "Nộp bài" : "Sang phần sau"}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function SectionQuestions({
  exam,
  section,
  attempt,
  onAnswer,
}: {
  exam: Exam;
  section: ExamSection;
  attempt: MockAttempt;
  onAnswer: (no: number, choice: number) => void;
}) {
  const audio = useExamAudio(section.audio ? examAudioUrl(exam, section.audio) : undefined);
  const [started, setStarted] = useState(false);
  useActiveTime(() => audio.playing);

  // Chỗ lẽ ra đang phát nếu tải lại trang giữa chừng: tính từ lúc bắt đầu phần.
  const sectionStart = attempt.deadline - section.minutes * 60_000;
  const startListening = () => {
    const elapsed = Math.max(0, (Date.now() - sectionStart) / 1000);
    setStarted(true);
    audio.playFrom(elapsed < 3 ? 0 : elapsed);
  };

  // Câu đang phát: đoạn âm thanh chứa thời điểm hiện tại.
  const current = useMemo(() => {
    if (!section.audio || !audio.playing) return null;
    return section.questions.find((q) => q.audio && audio.time >= q.audio[0] && audio.time < q.audio[1])?.no ?? null;
  }, [section, audio.playing, audio.time]);

  const lastScrolled = useRef<number | null>(null);
  useEffect(() => {
    if (current === null || current === lastScrolled.current) return;
    lastScrolled.current = current;
    document.getElementById(`q-${current}`)?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, [current]);

  return (
    <div className="flex flex-col gap-8">
      {section.audio && (
        <div className="flex items-center gap-3 rounded-xl bg-muted/60 p-4">
          <Headphones className="size-5 shrink-0" aria-hidden />
          {started && audio.playing ? (
            <p className="text-sm">Đang phát{current ? ` câu ${current}` : ""} — nghe và chọn đáp án như thi thật.</p>
          ) : started ? (
            <p className="text-sm text-muted-foreground">
              Bài nghe đã phát xong. Soát lại đáp án rồi bấm “Xong phần này”.
            </p>
          ) : (
            <Button onClick={startListening}>
              <Play className="size-4" aria-hidden />
              Bắt đầu phần nghe
            </Button>
          )}
        </div>
      )}

      {section.groups.map((g, gi) => (
        <section key={g.from} className={cn("flex flex-col gap-5", gi > 0 && "border-t border-border pt-8")}>
          <GroupBlock exam={exam} group={g} />
          {section.questions
            .filter((q) => q.no >= g.from && q.no <= g.to)
            .map((q) => (
              <div key={q.no} id={`q-${q.no}`} className="flex scroll-mt-36 flex-col gap-3 py-1">
                <p className="font-semibold tabular-nums">{q.no}.</p>
                <PromptView exam={exam} question={q} />
                <OptionList
                  exam={exam}
                  question={q}
                  value={attempt.answers[qKey(section.id, q.no)]}
                  onChange={(c) => onAnswer(q.no, c)}
                />
              </div>
            ))}
        </section>
      ))}
    </div>
  );
}
