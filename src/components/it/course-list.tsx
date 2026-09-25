"use client";

import Link from "next/link";
import { ArrowRight, BookOpen, Check, ChevronRight, Clock } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { courseProgressId, lessonHref, type CourseCard } from "@/lib/courses";
import { getBookProgress, useProgressStore, type BookProgress } from "@/lib/progress-store";
import { useIsClient } from "@/lib/use-is-client";

/**
 * Mô tả khoá trong course.md kết thúc bằng "Cần học trước: …". Ở lộ trình,
 * số thứ tự đã nói lên điều đó nên bỏ đi cho gọn; trang khoá học vẫn giữ.
 */
function shortSummary(summary: string): string {
  return summary.split(" Cần học trước:")[0];
}

/**
 * Lộ trình IT: các khoá đánh số theo thứ tự nên học, nối bằng một đường dọc,
 * kèm thẻ "Học tiếp" trỏ về bài mở gần nhất.
 *
 * Tiến độ dùng chung kho với sách (xem `courseProgressId`) nên được đồng bộ
 * giữa các tab và sao lưu cùng dữ liệu học, khỏi dựng kho thứ hai.
 */
export function CourseList({ courses }: { courses: CourseCard[] }) {
  const isClient = useIsClient();
  const progressByBook = useProgressStore((s) => s.books);

  const progressOf = (id: string): BookProgress | undefined =>
    isClient ? progressByBook[courseProgressId(id)] : undefined;

  // Khoá mở gần nhất: bản ghi tiến độ có updatedAt mới nhất.
  const recent = courses
    .map((course) => ({ course, progress: progressOf(course.id) }))
    .filter((x) => x.progress?.updatedAt)
    .sort((a, b) => (b.progress!.updatedAt > a.progress!.updatedAt ? 1 : -1))[0];
  const resumeLesson = recent?.course.lessons[(recent.progress!.lastPage || 1) - 1];

  return (
    <div className="flex flex-col gap-6">
      {recent && resumeLesson && (
        <Link
          href={lessonHref(recent.course.id, resumeLesson.slug)}
          className="group flex items-center gap-4 rounded-xl border border-border bg-muted/40 px-4 py-3 transition-colors hover:bg-muted"
        >
          <span className="flex min-w-0 flex-1 flex-col">
            <span className="text-xs text-muted-foreground">Học tiếp · {recent.course.title}</span>
            <span className="truncate font-medium">
              Bài {recent.progress!.lastPage}: {resumeLesson.title}
            </span>
          </span>
          <ArrowRight
            className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
            aria-hidden
          />
        </Link>
      )}

      <ol className="flex flex-col">
        {courses.map((course, i) => {
          const done = isClient
            ? getBookProgress(progressByBook, courseProgressId(course.id)).readPages.length
            : 0;
          const finished = done > 0 && done >= course.total;
          const last = i === courses.length - 1;
          return (
            <li key={course.id} className="relative flex gap-4">
              {/* Cột số bước và đường nối sang khoá kế tiếp. */}
              <span className="relative flex w-7 shrink-0 flex-col items-center">
                <span
                  className={cn(
                    "z-10 mt-4 flex size-7 items-center justify-center rounded-full border text-xs font-medium tabular-nums",
                    finished
                      ? "border-foreground bg-foreground text-background"
                      : done > 0
                        ? "border-foreground bg-background text-foreground"
                        : "border-border bg-background text-muted-foreground"
                  )}
                  aria-hidden
                >
                  {finished ? <Check className="size-3.5" /> : i + 1}
                </span>
                {!last && <span className="absolute top-11 bottom-0 w-px bg-border" aria-hidden />}
              </span>

              <Link
                href={`/it/${course.id}`}
                className="group -mx-2 flex min-w-0 flex-1 items-center gap-4 rounded-lg px-2 py-4 transition-colors hover:bg-muted/60"
              >
                <span className="flex min-w-0 flex-1 flex-col gap-1.5">
                  <span className="font-medium">{course.title}</span>
                  <span className="line-clamp-2 text-sm text-muted-foreground">
                    {shortSummary(course.summary)}
                  </span>
                  <span className="flex items-center gap-3 text-xs text-muted-foreground tabular-nums">
                    <span className="inline-flex items-center gap-1">
                      <BookOpen className="size-3.5" aria-hidden />
                      {course.total} bài
                    </span>
                    <span className="inline-flex items-center gap-1">
                      <Clock className="size-3.5" aria-hidden />
                      {course.minutes} phút
                    </span>
                    {done > 0 && (
                      <span>
                        đã học {done}/{course.total}
                      </span>
                    )}
                  </span>
                  {done > 0 && !finished && (
                    <Progress value={(done / course.total) * 100} className="h-1" />
                  )}
                </span>
                <ChevronRight
                  className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                  aria-hidden
                />
              </Link>
            </li>
          );
        })}
      </ol>
    </div>
  );
}
