"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowLeft, ArrowRight, Check, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import {
  courseLessons,
  courseMinutes,
  courseProgressId,
  lessonHref,
  lessonNumber,
  type Course,
} from "@/lib/courses";
import { getBookProgress, useProgressStore } from "@/lib/progress-store";

const NO_SUBSCRIBE = () => () => {};

/** Mục lục một khoá: các chương, bài đã học có dấu ✓, nút học tiếp. */
export function CourseOverview({ course }: { course: Course }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const progressByBook = useProgressStore((s) => s.books);
  const read = new Set(
    isClient ? getBookProgress(progressByBook, courseProgressId(course.id)).readPages : []
  );
  const lessons = courseLessons(course);
  const next = lessons.find((l) => !read.has(lessonNumber(course, l.slug))) ?? lessons[0];

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8">
      <Link href="/it" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="size-4" aria-hidden />
        Khoá học IT
      </Link>

      <section className="-mt-4 flex flex-col gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{course.title}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{course.summary}</p>
          <p className="mt-2 text-xs text-muted-foreground tabular-nums">
            {course.level} · {lessons.length} bài · {courseMinutes(course)} phút · đã học {read.size}/
            {lessons.length}
          </p>
        </div>
        {read.size > 0 && <Progress value={(read.size / lessons.length) * 100} className="h-1" />}
        {next && (
          <Button asChild size="lg" className="self-start">
            <Link href={lessonHref(course.id, next.slug)}>
              {read.size > 0 ? "Học tiếp" : "Bắt đầu học"}: {next.title}
              <ArrowRight className="size-4" aria-hidden />
            </Link>
          </Button>
        )}
      </section>

      {course.modules.map((mod) => (
        <section key={mod.slug} className="flex flex-col gap-2">
          <div>
            <h2 className="text-base font-semibold">{mod.title}</h2>
            {mod.summary && <p className="text-sm text-muted-foreground">{mod.summary}</p>}
          </div>
          <ul className="flex flex-col divide-y divide-border border-y border-border">
            {mod.lessons.map((lesson) => {
              const done = read.has(lessonNumber(course, lesson.slug));
              return (
                <li key={lesson.slug}>
                  <Link
                    href={lessonHref(course.id, lesson.slug)}
                    className="group -mx-2 flex items-center gap-3 rounded-lg px-2 py-3 transition-colors hover:bg-muted/60"
                  >
                    <span
                      className={cn(
                        "flex size-5 shrink-0 items-center justify-center rounded-full border text-[0.7rem] tabular-nums",
                        done ? "border-foreground bg-foreground text-background" : "border-border text-muted-foreground"
                      )}
                      aria-hidden
                    >
                      {done ? <Check className="size-3" /> : lessonNumber(course, lesson.slug)}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{lesson.title}</span>
                    <span className="inline-flex shrink-0 items-center gap-1 text-xs text-muted-foreground tabular-nums">
                      <Clock className="size-3.5" aria-hidden />
                      {lesson.minutes} phút
                    </span>
                  </Link>
                </li>
              );
            })}
          </ul>
        </section>
      ))}
    </div>
  );
}
