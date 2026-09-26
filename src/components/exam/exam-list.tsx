"use client";

import { examTitle, practiceProgress, totalMinutes, type Exam, type ExamLevel } from "@/lib/exams";
import { finishedAttempts, useExamStore, type MockAttempt } from "@/lib/exam-store";
import { MilestoneBadge, ProgressRing } from "@/components/exam/exam-chrome";
import { PageHeader } from "@/components/layout/page-header";
import { SegmentedNav } from "@/components/layout/segmented-nav";
import { RowLink, RowList } from "@/components/layout/row-list";
import { useIsClient } from "@/lib/use-is-client";
import { examLevelHref, examLevelInfo, levelsOfLang } from "@/lib/exam-levels";
import { LANGUAGES } from "@/lib/languages";

function best(attempts: MockAttempt[]): MockAttempt | null {
  return attempts.reduce<MockAttempt | null>((b, a) => (b === null || (a.score ?? 0) > (b.score ?? 0) ? a : b), null);
}

/**
 * Trang Luyện thi: đầu trang, tab kỳ thi, rồi danh sách đề — mỗi đề một dòng
 * gọn: tiến độ luyện từng câu (vòng tròn) và điểm thi thử cao nhất kèm mốc.
 * Cùng khung với Thư viện và Cẩm nang (PageHeader + SegmentedNav + RowList).
 *
 * Kỳ thi đến từ URL (`/exam/toeic`) chứ không phải state của trang: header
 * điện thoại hiện tab kỳ thi và đổi được, chia sẻ link hay bấm quay lại cũng
 * giữ đúng kỳ thi.
 */
export function ExamList({ exams, level }: { exams: Exam[]; level: ExamLevel }) {
  const isClient = useIsClient();
  const attempts = useExamStore((s) => s.attempts);
  const practice = useExamStore((s) => s.practice);

  const info = examLevelInfo(level);
  // Tầng ngôn ngữ đã chọn trên header; hàng chọn trong trang là KỲ THI của
  // ngôn ngữ đó — một kỳ thì khỏi hiện hàng.
  const siblings = levelsOfLang(info.lang);
  const langLabel = LANGUAGES.find((l) => l.code === info.lang)?.label ?? "";
  const shown = exams.filter((e) => e.level === level);
  const sample = shown[0];
  const done = isClient ? attempts.filter((a) => a.finishedAt && shown.some((e) => e.id === a.examId)) : [];
  const top = best(done);
  const triedExams = new Set(done.map((a) => a.examId)).size;

  return (
    <>
      <div className="flex flex-col gap-3">
        <PageHeader
          title={`Luyện thi ${langLabel.replace(/^Tiếng/, "tiếng")}`.trim()}
          subtitle="Đề luyện do VieLang tự biên soạn theo đúng format TOEIC — luyện từng câu có giải thích, hoặc thi thử có tính giờ."
        />
        {siblings.length > 1 && (
          <SegmentedNav
            label="Kỳ thi"
            items={siblings.map((l) => ({
              key: l.id,
              label: l.id,
              active: level === l.id,
              href: examLevelHref(l.id),
            }))}
          />
        )}
      </div>

      {/* Gợi ý của kỳ thi (trước đây là dòng thứ hai trong nút tab) chuyển
          xuống đây — tab chỉ còn một dòng như ở Thư viện và Cẩm nang. */}
      <p className="-mt-3 text-sm text-muted-foreground">
        {info.hint} · {shown.length} đề
        {sample ? ` · mỗi đề ${totalMinutes(sample)} phút` : ""}
        {triedExams > 0 && top
          ? ` · đã thi thử ${triedExams} đề, cao nhất ${top.score} điểm`
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
                  <span className="block font-medium">{examTitle(exam)}</span>
                  <span className="block text-sm text-muted-foreground">
                    {exam.sections.map((s) => s.title).join(" · ")}
                  </span>
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
                      <MilestoneBadge exam={exam} score={b.score ?? 0} />
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
