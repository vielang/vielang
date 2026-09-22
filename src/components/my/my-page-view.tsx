"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { Book } from "@/lib/books";
import {
  bestStreak,
  countStudyDays,
  currentStreak,
  minutesThisWeek,
} from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { useProgressStore } from "@/lib/progress-store";
import { useQuizStore } from "@/lib/quiz-store";
import { useRecordingStore } from "@/lib/recording-store";
import { StudyHeatmap } from "@/components/my/study-heatmap";
import { BookProgressCard, booksInProgress } from "@/components/my/book-progress";
import { summarizeQuiz, WrongList } from "@/components/my/quiz-review";
import { BackupPanel } from "@/components/my/backup-panel";
import { AbilityPanel } from "@/components/my/ability-panel";
import { computeAbility } from "@/lib/ability";

const NO_SUBSCRIBE = () => () => {};
const GOAL_CHOICES = [30, 60, 90, 150, 300];

/**
 * My page: tiến độ và lịch sử học, tính hoàn toàn từ dữ liệu trên trình
 * duyệt.
 *
 * Chỉ vẽ sau khi đã ở trình duyệt: server không biết gì về localStorage, vẽ
 * luôn thì server ra "0 ngày" rồi trình duyệt nhảy sang số thật (và React báo
 * lệch hydration). Trước đó hiện khung chờ.
 */
export function MyPageView({ books }: { books: readonly Book[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const days = useActivityStore((s) => s.days);
  const studied = useActivityStore((s) => s.studied);
  const goal = useActivityStore((s) => s.weeklyGoalMinutes);
  const grades = useActivityStore((s) => s.grades);
  const setGoal = useActivityStore((s) => s.setWeeklyGoal);
  const progressByBook = useProgressStore((s) => s.books);
  const quizPages = useQuizStore((s) => s.pages);
  const recordings = useRecordingStore((s) => s.recordings);
  const recordingsReady = useRecordingStore((s) => s.hasHydrated);

  if (!isClient) return <MyPageSkeleton />;

  const today = new Date();
  const streak = currentStreak(days, today);
  const weekMinutes = minutesThisWeek(days, today);
  const goalPercent = Math.min(100, Math.round((weekMinutes / goal) * 100));
  const studiedTotal = Object.values(studied).reduce((n, pages) => n + pages.length, 0);
  const recordingTotal = Object.values(recordings).reduce((n, list) => n + list.length, 0);
  const rows = booksInProgress(books, progressByBook, studied);
  const quiz = summarizeQuiz(quizPages);
  const ability = computeAbility(grades, quizPages);
  const hasAnyData = rows.length > 0 || countStudyDays(days) > 0 || ability.total > 0;

  if (!hasAnyData) {
    return (
      <section className="flex flex-col items-start gap-3 rounded-lg border border-dashed border-border p-6">
        <p className="font-medium">Chưa có dữ liệu học nào trên trình duyệt này.</p>
        <p className="text-sm text-muted-foreground">
          Mở một cuốn sách và học vài trang — thời gian học, trang đã học và kết quả bài
          tập sẽ hiện ở đây. Có file sao lưu từ máy khác thì nhập ở cuối trang.
        </p>
        <Button asChild size="sm">
          <Link href="/">Chọn sách để học</Link>
        </Button>
        <div className="mt-4 w-full border-t border-border pt-4">
          <BackupPanel />
        </div>
      </section>
    );
  }

  return (
    <div className="flex flex-col gap-8">
      {/* Mấy con số đầu trang: mỗi số một ô, không vẽ biểu đồ cho một con số. */}
      <section aria-label="Tóm tắt" className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <StatTile
          label="Chuỗi ngày học"
          value={`${streak} ngày`}
          hint={`Dài nhất: ${bestStreak(days)} ngày`}
        />
        <div className="flex flex-col gap-2 rounded-lg border border-border p-4">
          <span className="text-xs text-muted-foreground">Tuần này</span>
          <span className="text-2xl font-semibold tabular-nums">
            {weekMinutes}
            <span className="text-sm font-normal text-muted-foreground"> / {goal} phút</span>
          </span>
          <Progress
            value={goalPercent}
            className="h-1.5"
            aria-label={`Đạt ${goalPercent}% mục tiêu tuần`}
          />
        </div>
        <StatTile
          label="Trang đã học"
          value={String(studiedTotal)}
          hint={`${countStudyDays(days)} ngày có học`}
        />
        <StatTile
          label="Luyện nói"
          value={recordingsReady ? `${recordingTotal} bản ghi` : "…"}
          hint="Bản ghi âm đã lưu"
        />
      </section>

      <section className="flex flex-col gap-3">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <h2 className="text-lg font-semibold tracking-tight">Lịch học</h2>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>Mục tiêu mỗi tuần</span>
            <ToggleGroup
              type="single"
              size="sm"
              variant="outline"
              value={String(goal)}
              onValueChange={(v) => v && setGoal(Number(v))}
              aria-label="Mục tiêu phút học mỗi tuần"
            >
              {GOAL_CHOICES.map((m) => (
                <ToggleGroupItem key={m} value={String(m)} className="tabular-nums">
                  {m}′
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          </div>
        </div>
        <StudyHeatmap days={days} today={today} />
        <p className="text-xs text-muted-foreground">
          Chỉ tính lúc trang sách đang mở trên màn hình và bạn có thao tác trong khoảng
          1,5 phút gần nhất. Học từ 1 phút trở lên mới tính là một ngày học.
        </p>
      </section>

      {rows.length > 0 && (
        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-semibold tracking-tight">Sách đang học</h2>
          <p className="text-xs text-muted-foreground">
            &quot;Đã học&quot; là trang bạn ở lại đọc ít nhất 12 giây; &quot;đã xem&quot; là mọi
            trang từng mở ra. Mỗi ô số là một bài — càng đậm là học càng nhiều trang
            của bài đó.
          </p>
          <div className="flex flex-col gap-3">
            {rows.map((row) => (
              <BookProgressCard key={row.book.id} row={row} />
            ))}
          </div>
        </section>
      )}

      <section className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold tracking-tight">Năng lực theo kỹ năng</h2>
        {ability.total > 0 ? (
          <>
            <p className="text-xs text-muted-foreground">
              Tính từ những bài bạn <strong>tự chấm</strong> sau khi xem đáp án (đúng hết =
              100%, sai vài câu = 50%, sai nhiều = 0%) và các câu quiz đã kiểm tra. Đây là
              số tự báo để biết kỹ năng nào cần luyện thêm, không phải điểm thi.
            </p>
            <AbilityPanel ability={ability} />
          </>
        ) : (
          <p className="text-sm text-muted-foreground">
            Làm bài xong, bấm chấm đáp án trên trang sách rồi chọn &quot;Đúng hết&quot;,
            &quot;Sai vài câu&quot; hoặc &quot;Sai nhiều&quot; — năng lực theo từ vựng, ngữ pháp,
            nghe, đọc, viết sẽ hiện ở đây.
          </p>
        )}
      </section>

      {quiz.checked > 0 && (
        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-semibold tracking-tight">Bài tập tự chấm</h2>
          <p className="text-sm">
            Đúng{" "}
            <span className="font-semibold tabular-nums">
              {quiz.correct}/{quiz.checked}
            </span>{" "}
            câu đã kiểm tra (
            <span className="tabular-nums">
              {Math.round((quiz.correct / quiz.checked) * 100)}%
            </span>
            ).{" "}
            {quiz.wrong.length > 0
              ? "Những câu còn sai — bấm để làm lại:"
              : "Không còn câu nào sai."}
          </p>
          <WrongList wrong={quiz.wrong} />
        </section>
      )}

      <section className="flex flex-col gap-3 border-t border-border pt-6">
        <h2 className="text-lg font-semibold tracking-tight">Sao lưu dữ liệu học</h2>
        <BackupPanel />
      </section>
    </div>
  );
}

function StatTile({ label, value, hint }: { label: string; value: string; hint?: string }) {
  return (
    <div className="flex flex-col gap-1 rounded-lg border border-border p-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-2xl font-semibold tabular-nums">{value}</span>
      {hint && <span className="text-xs text-muted-foreground">{hint}</span>}
    </div>
  );
}

function MyPageSkeleton() {
  return (
    <div className="flex flex-col gap-8" aria-busy="true">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        {Array.from({ length: 4 }, (_, i) => (
          <Skeleton key={i} className="h-24" />
        ))}
      </div>
      <Skeleton className="h-40" />
      <Skeleton className="h-32" />
    </div>
  );
}
