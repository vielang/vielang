"use client";

import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ArrowLeft, Clock, Headphones, ListChecks, Play } from "lucide-react";
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
  isPointsOnly,
  scoreExam,
  sectionVi,
  totalMinutes,
  type Exam,
  type ExamSection,
} from "@/lib/exams";
import { activeAttempt, useExamStore, type MockAttempt } from "@/lib/exam-store";
import { useActiveTime } from "@/lib/use-study-tracker";
import { ContentView, GroupBlock } from "@/components/exam/exam-content";
import { OptionList } from "@/components/exam/option-list";
import { useExamAudio } from "@/components/exam/use-exam-audio";

const NO_SUBSCRIBE = () => () => {};

function mmss(ms: number): string {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/**
 * Thi thử: giống thi thật nhất có thể.
 *
 * - Tính giờ theo từng phần (TOPIK I: nghe 40', đọc 60'). Hết giờ tự sang
 *   phần sau / tự nộp. Sang phần sau là không quay lại phần trước.
 * - Phần nghe phát LIỀN MẠCH một lần, không tạm dừng, không tua — như phòng
 *   thi. Câu đang phát được tô và cuộn tới.
 * - Tải lại trang giữa chừng thì làm tiếp: đồng hồ tính theo mốc hết giờ đã
 *   lưu, file nghe phát tiếp từ đúng chỗ lẽ ra đang phát.
 */
export function MockView({ exam }: { exam: Exam }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const attempts = useExamStore((s) => s.attempts);
  const attempt = isClient ? activeAttempt(attempts, exam.id) : undefined;
  const startMock = useExamStore((s) => s.startMock);

  if (!isClient) return null;
  if (!attempt) {
    return (
      <MockIntro
        exam={exam}
        onStart={() => startMock(exam.id, exam.sections[0].minutes)}
      />
    );
  }
  return <MockRunning key={attempt.id} exam={exam} attempt={attempt} />;
}

function MockIntro({ exam, onStart }: { exam: Exam; onStart: () => void }) {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <Link
        href={`/exam/${exam.id}`}
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        {examTitle(exam)}
      </Link>
      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-6">
        <h1 className="text-xl font-semibold tracking-tight">Thi thử {examTitle(exam)}</h1>
        <ul className="flex flex-col gap-2 text-sm">
          {exam.sections.map((s) => (
            <li key={s.id} className="flex items-center gap-2">
              {s.audio ? <Headphones className="size-4" aria-hidden /> : <ListChecks className="size-4" aria-hidden />}
              <span className="font-medium">{sectionVi(s.id)}</span>
              <span className="text-muted-foreground">
                {s.questions.length} câu · {s.minutes} phút
              </span>
            </li>
          ))}
        </ul>
        <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
          <li>Tổng thời gian {totalMinutes(exam)} phút. Hết giờ phần nào là tự sang phần sau.</li>
          <li>Phần nghe phát liền một lần như phòng thi — không tạm dừng, không tua lại được.</li>
          <li>Đã sang phần sau thì không quay lại phần trước.</li>
          <li>Lỡ tải lại trang thì vẫn làm tiếp được, đồng hồ không dừng.</li>
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
  const [sheetOpen, setSheetOpen] = useState(false);
  const remaining = attempt.deadline - now;

  const submitSection = useCallback(() => {
    setConfirm(false);
    if (isLast) {
      const result = scoreExam(exam, attempt.answers);
      finish(attempt.id, result.score, result.level);
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

  const answered = section.questions.filter((q) => attempt.answers[q.no] !== undefined).length;

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-5 pb-28">
      <div className="sticky top-14 z-30 -mx-4 flex items-center justify-between gap-3 border-b border-border bg-background/95 px-4 py-2 backdrop-blur">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">
            {examTitle(exam)} · {sectionVi(section.id)}
          </p>
          <p className="text-xs text-muted-foreground tabular-nums">
            Đã chọn {answered}/{section.questions.length} câu
          </p>
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

      <SectionQuestions
        key={section.id}
        exam={exam}
        section={section}
        attempt={attempt}
        onAnswer={(no, c) => setAnswer(attempt.id, no, c)}
      />

      <Button
        variant="outline"
        className="fixed right-4 bottom-4 z-30 shadow-lg"
        onClick={() => setSheetOpen(true)}
      >
        <ListChecks className="size-4" aria-hidden />
        Phiếu trả lời {answered}/{section.questions.length}
      </Button>

      <Dialog open={sheetOpen} onOpenChange={setSheetOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Phiếu trả lời — {sectionVi(section.id)}</DialogTitle>
            <DialogDescription>Bấm vào số câu để tới câu đó.</DialogDescription>
          </DialogHeader>
          <div className="grid grid-cols-5 gap-2 sm:grid-cols-6">
            {section.questions.map((q) => {
              const a = attempt.answers[q.no];
              return (
                <a
                  key={q.no}
                  href={`#q-${q.no}`}
                  onClick={() => setSheetOpen(false)}
                  className={cn(
                    "flex flex-col items-center rounded-md border py-1 text-xs tabular-nums",
                    a ? "border-primary bg-primary text-primary-foreground" : "border-border"
                  )}
                >
                  <span>{q.no}</span>
                  <span className="text-sm">{a ? "①②③④"[a - 1] : "·"}</span>
                </a>
              );
            })}
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={confirm} onOpenChange={setConfirm}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{isLast ? "Nộp bài?" : `Kết thúc phần ${sectionVi(section.id)}?`}</DialogTitle>
            <DialogDescription>
              {answered < section.questions.length
                ? `Còn ${section.questions.length - answered} câu chưa chọn. `
                : ""}
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
    <div className="flex flex-col gap-6">
      {section.audio && (
        <div className="flex items-center gap-3 rounded-xl border border-border p-4">
          <Headphones className="size-5 shrink-0" aria-hidden />
          {started && audio.playing ? (
            <p className="text-sm">
              Đang phát{current ? ` câu ${current}` : ""} — nghe và chọn đáp án như thi thật.
            </p>
          ) : started ? (
            <p className="text-sm text-muted-foreground">Bài nghe đã phát xong. Soát lại đáp án rồi bấm “Xong phần này”.</p>
          ) : (
            <Button onClick={startListening}>
              <Play className="size-4" aria-hidden />
              Bắt đầu phần nghe
            </Button>
          )}
        </div>
      )}

      {section.groups.map((g) => (
        <section key={g.from} className="flex flex-col gap-4">
          <GroupBlock exam={exam} group={g} />
          {section.questions
            .filter((q) => q.no >= g.from && q.no <= g.to)
            .map((q) => (
              <div
                key={q.no}
                id={`q-${q.no}`}
                className={cn(
                  "flex scroll-mt-32 flex-col gap-3 rounded-xl border p-3 transition-colors",
                  current === q.no ? "border-primary ring-2 ring-primary/30" : "border-border"
                )}
              >
                <p className="text-sm font-semibold">
                  {q.no}. <span className="font-normal text-muted-foreground">({q.points} điểm)</span>
                </p>
                {!isPointsOnly(q.prompt) && <ContentView exam={exam} content={q.prompt} alt={`Câu ${q.no}`} />}
                <OptionList
                  exam={exam}
                  question={q}
                  value={attempt.answers[q.no]}
                  onChange={(c) => onAnswer(q.no, c)}
                />
              </div>
            ))}
        </section>
      ))}
    </div>
  );
}
