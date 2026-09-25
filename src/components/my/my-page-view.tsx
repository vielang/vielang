"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowRight, ChevronDown, Flame, Lightbulb } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { Book } from "@/lib/books";
import { countStudyDays, currentStreak, minutesThisWeek } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { computeAbility } from "@/lib/ability";
import { buildSuggestions, companionMessage, greeting, shortBookName } from "@/lib/companion";
import { useProgressStore, resumePage } from "@/lib/progress-store";
import { useQuizStore } from "@/lib/quiz-store";
import { useRecordingHydration, useRecordingStore } from "@/lib/recording-store";
import { StudyHeatmap } from "@/components/my/study-heatmap";
import { WeekStrip } from "@/components/my/week-strip";
import { SuggestionList } from "@/components/my/suggestion-list";
import { BookProgressCard, booksInProgress } from "@/components/my/book-progress";
import { BookmarkStrip } from "@/components/my/bookmark-strip";
import { summarizeQuiz, WrongList } from "@/components/my/quiz-review";
import { RedoList, SkillBars, WeakGrammarList } from "@/components/my/ability-panel";
import { BackupPanel } from "@/components/my/backup-panel";
import { ExamCard } from "@/components/my/exam-card";

const NO_SUBSCRIBE = () => () => {};
const GOAL_CHOICES = [30, 60, 90, 150, 300];

/**
 * My page — viết như một người bạn đồng hành, không phải bảng báo cáo.
 *
 * Thứ tự theo câu hỏi người học mang tới trang này:
 * 1. "Hôm nay mình thế nào, giờ làm gì?" → lời nhắn + nút học tiếp.
 * 2. "Tuần này có đều không?" → 7 ngày và số phút so với mục tiêu.
 * 3. "Nên ôn gì?" → tối đa 3 gợi ý cụ thể.
 * 4. Sách đang học, kỹ năng.
 * 5. Mọi thứ chi tiết (lịch 12 tuần, danh sách đầy đủ, sao lưu) gập lại ở
 *    cuối — có khi cần, nhưng không đứng chắn đường.
 *
 * Chỉ vẽ sau khi đã ở trình duyệt: server không biết gì về localStorage, vẽ
 * luôn thì server ra "0 ngày" rồi nhảy sang số thật (lệch hydration).
 */
export function MyPageView({ books }: { books: readonly Book[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const days = useActivityStore((s) => s.days);
  const studied = useActivityStore((s) => s.studied);
  const goal = useActivityStore((s) => s.weeklyGoalMinutes);
  const setGoal = useActivityStore((s) => s.setWeeklyGoal);
  const grades = useActivityStore((s) => s.grades);
  const progressByBook = useProgressStore((s) => s.books);
  const quizPages = useQuizStore((s) => s.pages);
  const recordings = useRecordingStore((s) => s.recordings);
  // Bản ghi âm nằm ở IndexedDB và chỉ nạp khi có người gọi — trước đây chỉ
  // trang đọc gọi, nên số bản ghi ở đây quay mãi không ra.
  const recordingsReady = useRecordingHydration();
  const [editingGoal, setEditingGoal] = useState(false);

  if (!isClient) return <MyPageSkeleton />;

  const now = new Date();
  const streak = currentStreak(days, now);
  const weekMinutes = minutesThisWeek(days, now);
  const goalPercent = Math.min(100, Math.round((weekMinutes / goal) * 100));
  const rows = booksInProgress(books, progressByBook, studied);
  const quiz = summarizeQuiz(quizPages);
  const ability = computeAbility(grades, quizPages);
  const suggestions = buildSuggestions(ability, quiz.wrong, books);
  const studyDays = countStudyDays(days);
  const studiedTotal = Object.values(studied).reduce((n, pages) => n + pages.length, 0);
  const recordingTotal = Object.values(recordings).reduce((n, list) => n + list.length, 0);

  if (rows.length === 0 && studyDays === 0 && ability.total === 0) {
    return <Welcome />;
  }

  const message = companionMessage({ days, now, streak, weekMinutes, goal });
  const current = rows[0];

  return (
    <div className="flex flex-col gap-8">
      {/* 1. Lời nhắn + việc làm ngay */}
      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-5 sm:p-6">
        <div className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">{greeting(now)} 👋</p>
          <h2 className="text-xl font-semibold tracking-tight text-balance">{message.title}</h2>
          <p className="text-sm text-muted-foreground text-pretty">{message.body}</p>
        </div>
        {current && (
          <Button asChild className="self-start">
            <Link href={`/read/${current.book.id}/${resumePage(current.progress)}`}>
              Học tiếp {shortBookName(current.book)} · trang {resumePage(current.progress)}
              <ArrowRight className="size-4" aria-hidden />
            </Link>
          </Button>
        )}
      </section>

      {/* 2. Tuần này */}
      <section className="flex flex-col gap-4" aria-labelledby="week-title">
        <div className="flex items-baseline justify-between gap-3">
          <h2 id="week-title" className="text-base font-semibold">
            Tuần này
          </h2>
          {streak > 1 && (
            <span className="inline-flex items-center gap-1 text-sm text-muted-foreground">
              <Flame className="size-4 text-orange-500" aria-hidden />
              {streak} ngày liên tiếp
            </span>
          )}
        </div>
        <WeekStrip days={days} today={now} />
        <div className="flex flex-col gap-2">
          <div className="flex items-baseline justify-between gap-3 text-sm">
            <span>
              <span className="text-lg font-semibold tabular-nums">{weekMinutes}</span>
              <span className="text-muted-foreground"> / {goal} phút</span>
            </span>
            <button
              type="button"
              onClick={() => setEditingGoal((v) => !v)}
              aria-expanded={editingGoal}
              className="text-xs text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
            >
              {editingGoal ? "Xong" : "Đổi mục tiêu"}
            </button>
          </div>
          <div
            className="h-1.5 overflow-hidden rounded-full bg-muted"
            role="progressbar"
            aria-valuenow={goalPercent}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`Đạt ${goalPercent}% mục tiêu tuần`}
          >
            <div className="h-full rounded-full bg-primary" style={{ width: `${goalPercent}%` }} />
          </div>
          {editingGoal && (
            <ToggleGroup
              type="single"
              size="sm"
              variant="outline"
              value={String(goal)}
              onValueChange={(v) => v && setGoal(Number(v))}
              aria-label="Mục tiêu phút học mỗi tuần"
              className="flex-wrap self-start"
            >
              {GOAL_CHOICES.map((m) => (
                <ToggleGroupItem key={m} value={String(m)} className="tabular-nums">
                  {m} phút
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          )}
        </div>
        <p className="text-xs text-muted-foreground">
          Tổng cộng {studyDays} ngày học · {studiedTotal} trang
          {recordingsReady && recordingTotal > 0 ? ` · ${recordingTotal} bản ghi luyện nói` : ""}
        </p>
      </section>

      {/* 3. Gợi ý */}
      <section className="flex flex-col gap-3" aria-labelledby="next-title">
        <h2 id="next-title" className="text-base font-semibold">
          Gợi ý cho bạn
        </h2>
        {suggestions.length > 0 ? (
          <SuggestionList items={suggestions} />
        ) : (
          <p className="flex gap-3 rounded-xl border border-dashed border-border p-4 text-sm text-muted-foreground">
            <Lightbulb className="mt-0.5 size-4 shrink-0" aria-hidden />
            <span>
              Làm bài xong, mở chấm đáp án trên trang sách rồi chọn{" "}
              <span className="text-foreground">Đúng hết</span>,{" "}
              <span className="text-foreground">Sai vài câu</span> hoặc{" "}
              <span className="text-foreground">Sai nhiều</span> — mình sẽ gợi ý bài nào
              nên ôn lại.
            </span>
          </p>
        )}
      </section>

      {/* 4. Sách đang học */}
      {rows.length > 0 && (
        <section className="flex flex-col gap-3" aria-labelledby="books-title">
          <h2 id="books-title" className="text-base font-semibold">
            Sách đang học
          </h2>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {rows.map((row) => (
              <BookProgressCard key={row.book.id} row={row} />
            ))}
          </div>
        </section>
      )}

      {/* Trang đã đánh dấu — ngay sau sách đang học, vì cùng nói về việc đọc. */}
      <BookmarkStrip books={books} progressByBook={progressByBook} />

      {/* Luyện thi TOPIK */}
      <section className="flex flex-col gap-3" aria-labelledby="exam-title">
        <h2 id="exam-title" className="text-base font-semibold">
          Luyện thi TOPIK
        </h2>
        <ExamCard />
      </section>

      {/* 5. Kỹ năng */}
      {ability.total > 0 && (
        <section className="flex flex-col gap-3" aria-labelledby="skills-title">
          <h2 id="skills-title" className="text-base font-semibold">
            Kỹ năng của bạn
          </h2>
          <SkillBars skills={ability.skills} />
          <p className="text-xs text-muted-foreground">
            Tính từ các bài bạn tự chấm và câu quiz đã kiểm tra — để biết nên luyện thêm
            phần nào, không phải điểm thi.
          </p>
        </section>
      )}

      {/* 6. Chi tiết — gập lại */}
      <section className="flex flex-col divide-y divide-border border-y border-border">
        <Disclosure title="Lịch học 12 tuần">
          <StudyHeatmap days={days} today={now} />
        </Disclosure>
        {ability.weakGrammar.length > 0 && (
          <Disclosure title="Điểm ngữ pháp nên ôn" count={ability.weakGrammar.length}>
            <WeakGrammarList items={ability.weakGrammar} />
          </Disclosure>
        )}
        {ability.redo.length > 0 && (
          <Disclosure title="Bài tự chấm còn sai" count={ability.redo.length}>
            <RedoList items={ability.redo} />
          </Disclosure>
        )}
        {quiz.wrong.length > 0 && (
          <Disclosure title="Câu quiz còn sai" count={quiz.wrong.length}>
            <WrongList wrong={quiz.wrong} />
          </Disclosure>
        )}
        <Disclosure title="Sao lưu & chuyển sang máy khác">
          <BackupPanel />
        </Disclosure>
      </section>
    </div>
  );
}

/** Mục gập/mở, dùng `<details>` gốc — bàn phím và trình đọc màn hình tự lo. */
function Disclosure({
  title,
  count,
  children,
}: {
  title: string;
  count?: number;
  children: React.ReactNode;
}) {
  return (
    <details className="group py-1">
      <summary className="flex cursor-pointer list-none items-center justify-between gap-3 py-3 text-sm font-medium [&::-webkit-details-marker]:hidden">
        <span>
          {title}
          {count !== undefined && (
            <span className="ml-2 rounded-full bg-muted px-2 py-0.5 text-xs font-normal text-muted-foreground tabular-nums">
              {count}
            </span>
          )}
        </span>
        <ChevronDown
          className="size-4 text-muted-foreground transition-transform group-open:rotate-180"
          aria-hidden
        />
      </summary>
      <div className="pb-4">{children}</div>
    </details>
  );
}

/** Lần đầu vào, chưa có gì — mời bắt đầu, không vẽ một trang toàn số 0. */
function Welcome() {
  return (
    <section className="flex flex-col gap-5 rounded-2xl bg-muted/60 p-6">
      <div className="flex flex-col gap-2">
        <h2 className="text-xl font-semibold tracking-tight">Bắt đầu hành trình nào 🌱</h2>
        <p className="text-sm text-muted-foreground text-pretty">
          Mở một cuốn sách và học vài trang. Mình sẽ ghi lại thời gian học, trang đã học,
          và gợi ý bài nên ôn — tất cả ngay trên máy của bạn, không cần tài khoản.
        </p>
      </div>
      <Button asChild className="self-start">
        <Link href="/">
          Chọn sách để học
          <ArrowRight className="size-4" aria-hidden />
        </Link>
      </Button>
      <div className="border-t border-border pt-4">
        <BackupPanel canExport={false} />
      </div>
    </section>
  );
}

function MyPageSkeleton() {
  return (
    <div className="flex flex-col gap-8" aria-busy="true">
      <Skeleton className="h-40 rounded-2xl" />
      <Skeleton className="h-28" />
      <Skeleton className="h-32" />
    </div>
  );
}
