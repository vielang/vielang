"use client";

import Image from "next/image";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { formatVideoDuration, videoLessonTitle, videoPosterUrl, type VideoLesson } from "@/lib/videos";
import { useVideoProgressStore } from "@/lib/video-progress-store";

const MAX_ITEMS = 3;

/**
 * Dải "Xem tiếp" trên đầu trang `/video` — cùng vai trò với `ContinueReading`
 * của thư viện sách: quay lại đúng tập, đúng chỗ đang xem dở trong một lần
 * chạm, không phải dò lại từ danh sách.
 */
export function ContinueWatching({ lessons }: { lessons: readonly VideoLesson[] }) {
  const progressByLesson = useVideoProgressStore((s) => s.lessons);
  const hasHydrated = useVideoProgressStore((s) => s.hasHydrated);

  // Tiến độ nằm ở localStorage nên server không biết gì — chờ nạp xong mới
  // vẽ, nếu không HTML hai bên lệch nhau.
  if (!hasHydrated) return null;

  const items = lessons
    .map((lesson) => ({ lesson, progress: progressByLesson[lesson.id] }))
    .filter(({ progress }) => progress && !progress.done && progress.time > 0)
    .sort((a, b) => b.progress!.updatedAt.localeCompare(a.progress!.updatedAt))
    .slice(0, MAX_ITEMS);

  if (items.length === 0) return null;

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-base font-semibold tracking-tight">Xem tiếp</h2>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {items.map(({ lesson, progress }) => {
          const duration = lesson.durationSec ?? 0;
          const percent = duration > 0 ? Math.round(((progress?.time ?? 0) / duration) * 100) : 0;
          return (
            <Link
              key={lesson.id}
              href={`/video/${lesson.id}`}
              className="group focus-visible:ring-ring flex min-w-0 items-center gap-3 rounded-xl border border-border bg-card p-2.5 transition-shadow hover:shadow-md focus-visible:ring-2 focus-visible:outline-none"
            >
              <div className="relative aspect-video w-20 shrink-0 overflow-hidden rounded-md bg-muted">
                <Image
                  src={videoPosterUrl(lesson.id)}
                  alt=""
                  fill
                  unoptimized
                  sizes="80px"
                  className="object-cover"
                />
              </div>
              <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                <p className="truncate text-sm leading-tight font-medium">{videoLessonTitle(lesson)}</p>
                <p className="text-xs tabular-nums text-muted-foreground">
                  {formatVideoDuration(progress?.time ?? 0)}/{formatVideoDuration(duration)} · {percent}%
                </p>
                <Progress value={percent} className="h-1" />
              </div>
              <ChevronRight
                className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                aria-hidden
              />
            </Link>
          );
        })}
      </div>
    </section>
  );
}
