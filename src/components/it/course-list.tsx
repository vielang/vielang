"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";
import { BookOpen, ChevronRight, Clock } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { courseLessons, courseMinutes, courseProgressId, type Course } from "@/lib/courses";
import { getBookProgress, useProgressStore } from "@/lib/progress-store";

const NO_SUBSCRIBE = () => () => {};

/**
 * Lộ trình IT: các khoá xếp theo thứ tự nên học, kèm số bài đã học.
 *
 * Tiến độ dùng chung kho với sách (xem `courseProgressId`) nên được đồng bộ
 * giữa các tab và sao lưu cùng dữ liệu học, khỏi dựng kho thứ hai.
 */
export function CourseList({ courses }: { courses: Course[] }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const progressByBook = useProgressStore((s) => s.books);

  return (
    <ul className="flex flex-col divide-y divide-border border-y border-border">
      {courses.map((course) => {
        const lessons = courseLessons(course);
        const done = isClient
          ? getBookProgress(progressByBook, courseProgressId(course.id)).readPages.length
          : 0;
        return (
          <li key={course.id}>
            <Link
              href={`/it/${course.id}`}
              className="group -mx-2 flex items-center gap-4 rounded-lg px-2 py-4 transition-colors hover:bg-muted/60"
            >
              <span className="flex min-w-0 flex-1 flex-col gap-1.5">
                <span className="flex flex-wrap items-baseline gap-x-2">
                  <span className="font-medium">{course.title}</span>
                  <span className="text-xs text-muted-foreground">{course.level}</span>
                </span>
                <span className="text-sm text-muted-foreground">{course.summary}</span>
                <span className="flex items-center gap-3 text-xs text-muted-foreground tabular-nums">
                  <span className="inline-flex items-center gap-1">
                    <BookOpen className="size-3.5" aria-hidden />
                    {lessons.length} bài
                  </span>
                  <span className="inline-flex items-center gap-1">
                    <Clock className="size-3.5" aria-hidden />
                    {courseMinutes(course)} phút
                  </span>
                  {done > 0 && <span>đã học {done}/{lessons.length}</span>}
                </span>
                {done > 0 && <Progress value={(done / lessons.length) * 100} className="h-1" />}
              </span>
              <ChevronRight
                className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                aria-hidden
              />
            </Link>
          </li>
        );
      })}
    </ul>
  );
}
