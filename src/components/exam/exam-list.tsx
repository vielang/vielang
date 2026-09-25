"use client";

import { useState, useSyncExternalStore } from "react";
import { practiceProgress, totalMinutes, type Exam, type ExamLevel } from "@/lib/exams";
import { finishedAttempts, useExamStore, type MockAttempt } from "@/lib/exam-store";
import { LevelBadge, ProgressRing } from "@/components/exam/exam-chrome";
import { PageHeader } from "@/components/layout/page-header";
import { SegmentedNav } from "@/components/layout/segmented-nav";
import { RowLink, RowList } from "@/components/layout/row-list";

const NO_SUBSCRIBE = () => () => {};

const LEVELS: { id: ExamLevel; hint: string }[] = [
  { id: "TOPIK I", hint: "Cấp 1–2 · Nghe, Đọc" },
  { id: "TOPIK II", hint: "Cấp 3–6 · Nghe, Viết, Đọc" },
];

function best(attempts: MockAttempt[]): MockAttempt | null {
  return attempts.reduce<MockAttempt | null>((b, a) => (b === null || (a.score ?? 0) > (b.score ?? 0) ? a : b), null);
}

/**
 * Trang Luyện thi: đầu trang, tab cấp đề, rồi danh sách đề — mỗi kỳ một dòng
 * gọn: tiến độ luyện từng câu (vòng tròn) và điểm thi thử cao nhất kèm cấp.
 * Cùng khung với Thư viện và Cẩm nang (PageHeader + SegmentedNav + RowList).
 */
export function ExamList({ exams }: { exams: Exam[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const attempts = useExamStore((s) => s.attempts);
  const practice = useExamStore((s) => s.practice);
  const [level, setLevel] = useState<ExamLevel>("TOPIK I");

  const shown = exams.filter((e) => e.level === level);
  const sample = shown[0];
  const done = isClient ? attempts.filter((a) => a.finishedAt && shown.some((e) => e.id === a.examId)) : [];
  const top = best(done);
  const triedExams = new Set(done.map((a) => a.examId)).size;

  return (
    <>
      <div className="flex flex-col gap-3">
        <PageHeader
          title="Luyện thi"
          subtitle="Đề TOPIK thật đã công bố — luyện từng câu và chấm ngay, hoặc thi thử có tính giờ."
        />
        <SegmentedNav
          label="Cấp đề thi"
          items={LEVELS.map((l) => ({
            key: l.id,
            label: l.id,
            active: level === l.id,
            onSelect: () => setLevel(l.id),
          }))}
        />
      </div>

      {/* Gợi ý của cấp (trước đây là dòng thứ hai trong nút tab) chuyển xuống
          đây — tab chỉ còn một dòng như ở Thư viện và Cẩm nang. */}
      <p className="-mt-3 text-sm text-muted-foreground">
        {LEVELS.find((l) => l.id === level)?.hint} · {shown.length} đề
        {sample ? ` · mỗi đề ${totalMinutes(sample)} phút` : ""}
        {triedExams > 0 && top
          ? ` · đã thi thử ${triedExams} đề, cao nhất ${top.score} điểm${top.level ? ` (${top.level})` : ""}`
          : ""}
      </p>

      <RowList>
        {shown.map((exam) => {
          const mine = isClient ? finishedAttempts(attempts, exam.id) : [];
          const b = best(mine);
          const progress = practiceProgress(exam, isClient ? practice[exam.id] : undefined);
          return (
            <RowLink key={exam.id} href={`/exam/${exam.id}`}>
                <span className="min-w-0 flex-1">
                  <span className="block font-medium">Kỳ {exam.round}</span>
                  <span className="block text-sm text-muted-foreground">Năm {exam.year}</span>
                </span>
                <span
                  className="flex items-center gap-2 text-xs text-muted-foreground tabular-nums"
                  title="Số câu đã luyện"
                >
                  <ProgressRing value={progress.done} max={progress.total} size={28} />
                  <span className="w-12">
                    {progress.done}/{progress.total}
                  </span>
                </span>
                <span className="flex w-24 flex-col items-end gap-0.5 text-sm tabular-nums">
                  {b ? (
                    <>
                      <span className="font-medium">{b.score} điểm</span>
                      <LevelBadge level={b.level} />
                    </>
                  ) : (
                    <span className="text-xs text-muted-foreground">Chưa thi thử</span>
                  )}
                </span>
            </RowLink>
          );
        })}
      </RowList>
    </>
  );
}
