"use client";

import Link from "next/link";
import { ArrowRight, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { courseProgressId, lessonHref, type CourseOutline } from "@/lib/courses";
import { getBookProgress, useProgressStore } from "@/lib/progress-store";
import { useIsClient } from "@/lib/use-is-client";
import { BackLink } from "@/components/layout/back-link";

/** Mục lục một khoá: các chương, bài đã học có dấu ✓, nút học tiếp. */
export function CourseOverview({ course }: { course: CourseOutline }) {
  const isClient = useIsClient();
  const progressByBook = useProgressStore((s) => s.books);
  const read = new Set(
    isClient ? getBookProgress(progressByBook, courseProgressId(course.id)).readPages : []
  );
  const lessons = course.modules.flatMap((m) => m.lessons);
  const next = lessons.find((l) => !read.has(l.no)) ?? lessons[0];

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8">
      <BackLink href="/it">Khoá học IT</BackLink>

      <section className="-mt-4 flex flex-col gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{course.title}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{course.summary}</p>
          <p className="mt-2 text-xs text-muted-foreground tabular-nums">
            {course.level} · {course.total} bài · {course.minutes} phút · đã học {read.size}/
            {course.total}
          </p>
        </div>
        {read.size > 0 && <Progress value={(read.size / course.total) * 100} className="h-1" />}
        {next && (
          <Button asChild size="lg" className="min-w-0 max-w-full self-start">
            <Link href={lessonHref(course.id, next.slug)}>
              <span className="min-w-0 truncate">
                {read.size > 0 ? "Học tiếp" : "Bắt đầu học"}: {next.title}
              </span>
              <ArrowRight className="size-4 shrink-0" aria-hidden />
            </Link>
          </Button>
        )}
      </section>

      {course.modules.map((mod, index) => (
        <section key={mod.slug} className="flex flex-col gap-2">
          <div>
            <div className="flex items-baseline gap-3">
              <h2 className="text-base font-semibold">
                <span className="mr-2 text-sm font-normal text-muted-foreground">Chương {index + 1}</span>
                {mod.title}
              </h2>
              <span className="ml-auto text-xs text-muted-foreground tabular-nums">
                {mod.lessons.filter((l) => read.has(l.no)).length}/{mod.lessons.length}
              </span>
            </div>
            {mod.summary && <p className="text-sm text-muted-foreground">{mod.summary}</p>}
          </div>
          <ul className="flex flex-col divide-y divide-border border-y border-border">
            {mod.lessons.map((lesson) => {
              const done = read.has(lesson.no);
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
                      {done ? <Check className="size-3" /> : lesson.no}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{lesson.title}</span>
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
