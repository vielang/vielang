"use client";

import { useState } from "react";
import Link from "next/link";
import { ArrowRight, ChevronDown, Flame } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { courseProgressId, lessonHref, type CourseCard } from "@/lib/courses";
import { countStudyDays, currentStreak, minutesThisWeek } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { companionMessage, greeting } from "@/lib/companion";
import { getBookProgress, useProgressStore } from "@/lib/progress-store";
import { StudyHeatmap } from "@/components/my/study-heatmap";
import { WeekStrip } from "@/components/my/week-strip";
import { BackupPanel } from "@/components/my/backup-panel";
import { useIsClient } from "@/lib/use-is-client";

const GOAL_CHOICES = [30, 60, 90, 150, 300];

/**
 * My page — viết như một người bạn đồng hành, không phải bảng báo cáo.
 *
 * Thứ tự theo câu hỏi người học mang tới trang này:
 * 1. "Hôm nay mình thế nào, giờ làm gì?" → lời nhắn + nút học tiếp.
 * 2. "Tuần này có đều không?" → 7 ngày và số phút so với mục tiêu.
 * 3. Khoá học đang học, mỗi khoá một nút học tiếp.
 * 4. Mọi thứ chi tiết (lịch 12 tuần, danh sách đầy đủ, sao lưu) gập lại ở
 *    cuối — có khi cần, nhưng không đứng chắn đường.
 *
 * Chỉ vẽ sau khi đã ở trình duyệt: server không biết gì về localStorage, vẽ
 * luôn thì server ra "0 ngày" rồi nhảy sang số thật (lệch hydration).
 */
export function MyPageView({ courses = [] }: { courses?: readonly CourseCard[] }) {
  const isClient = useIsClient();
  const days = useActivityStore((s) => s.days);
  const goal = useActivityStore((s) => s.weeklyGoalMinutes);
  const setGoal = useActivityStore((s) => s.setWeeklyGoal);
  const progressById = useProgressStore((s) => s.books);
  const [editingGoal, setEditingGoal] = useState(false);

  if (!isClient) return <MyPageSkeleton />;

  const now = new Date();
  const streak = currentStreak(days, now);
  const weekMinutes = minutesThisWeek(days, now);
  const goalPercent = Math.min(100, Math.round((weekMinutes / goal) * 100));
  const courseRows = coursesInProgress(courses, progressById);
  const studyDays = countStudyDays(days);
  const lessonsDone = courseRows.reduce((n, row) => n + row.done, 0);

  if (courseRows.length === 0 && studyDays === 0) {
    return <Welcome />;
  }

  const message = companionMessage({ days, now, streak, weekMinutes, goal });
  const current = courseRows.find((row) => row.next);

  return (
    <div className="flex flex-col gap-8">
      {/* 1. Lời nhắn + việc làm ngay */}
      <section className="flex flex-col gap-4 rounded-2xl bg-muted/60 p-5 sm:p-6">
        <div className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">{greeting(now)} 👋</p>
          <h2 className="text-xl font-semibold tracking-tight text-balance">{message.title}</h2>
          <p className="text-sm text-muted-foreground text-pretty">{message.body}</p>
        </div>
        {current?.next && (
          <Button asChild className="self-start">
            <Link href={lessonHref(current.course.id, current.next.slug)}>
              Học tiếp {current.course.title}
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
          Tổng cộng {studyDays} ngày học · {lessonsDone} bài đã học
        </p>
      </section>

      {/* 3. Khoá học đang học */}
      {courseRows.length > 0 && (
        <section className="flex flex-col gap-3" aria-labelledby="courses-title">
          <h2 id="courses-title" className="text-base font-semibold">
            Khoá học đang học
          </h2>
          <ul className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {courseRows.map(({ course, done, next }) => (
              <li key={course.id} className="flex flex-col gap-2 rounded-xl border border-border p-4">
                <Link href={`/it/${course.id}`} className="font-medium hover:underline">
                  {course.title}
                </Link>
                <div
                  className="h-1.5 overflow-hidden rounded-full bg-muted"
                  role="progressbar"
                  aria-valuenow={done}
                  aria-valuemin={0}
                  aria-valuemax={course.total}
                  aria-label={`Đã học ${done}/${course.total} bài`}
                >
                  <div className="h-full rounded-full bg-primary" style={{ width: `${(done / course.total) * 100}%` }} />
                </div>
                <p className="text-xs text-muted-foreground tabular-nums">
                  Đã học {done}/{course.total} bài
                </p>
                {next && (
                  <Link
                    href={lessonHref(course.id, next.slug)}
                    className="inline-flex items-center gap-1 text-sm text-primary hover:underline"
                  >
                    Học tiếp: {next.title}
                    <ArrowRight className="size-3.5" aria-hidden />
                  </Link>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}

      {/* 4. Chi tiết — gập lại */}
      <section className="flex flex-col divide-y divide-border border-y border-border">
        <Disclosure title="Lịch học 12 tuần">
          <StudyHeatmap days={days} today={now} />
        </Disclosure>
        <Disclosure title="Sao lưu & chuyển sang máy khác">
          <BackupPanel />
        </Disclosure>
      </section>
    </div>
  );
}

/**
 * Khoá học đã bắt đầu, khoá học gần đây nhất lên trước. Tiến độ nằm trong
 * `progress-store` (xem `courseProgressId`), mỗi "trang" là số thứ tự bài.
 * Bài học tiếp = bài có số nhỏ nhất chưa học.
 */
function coursesInProgress(
  courses: readonly CourseCard[],
  progressById: Parameters<typeof getBookProgress>[0]
) {
  return courses
    .map((course) => {
      const p = getBookProgress(progressById, courseProgressId(course.id));
      const read = new Set(p.readPages);
      const nextIndex = course.lessons.findIndex((_, i) => !read.has(i + 1));
      return { course, done: read.size, next: nextIndex >= 0 ? course.lessons[nextIndex] : undefined, updatedAt: p.updatedAt };
    })
    .filter((row) => row.done > 0)
    .sort((a, b) => (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""));
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
          Mở một khoá học và học vài bài. Mình sẽ ghi lại thời gian học và bài đã học — tất cả ngay trên máy của bạn, không cần tài khoản.
        </p>
      </div>
      <Button asChild className="self-start">
        <Link href="/">
          Chọn khoá học
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
