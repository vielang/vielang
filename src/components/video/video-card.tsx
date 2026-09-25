import Image from "next/image";
import Link from "next/link";
import { CirclePlay, Languages } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import {
  formatVideoDuration,
  videoLessonTitle,
  videoPosterUrl,
  type VideoLesson,
} from "@/lib/videos";

export function VideoCard({ lesson }: { lesson: VideoLesson }) {
  const duration = formatVideoDuration(lesson.durationSec);

  return (
    <Link
      href={`/video/${lesson.id}`}
      className="group focus-visible:ring-ring flex min-w-0 flex-col overflow-hidden rounded-xl border border-border bg-card transition-shadow hover:shadow-md focus-visible:outline-none focus-visible:ring-2"
    >
      <div className="relative aspect-video w-full overflow-hidden bg-muted">
        <Image
          src={videoPosterUrl(lesson.id)}
          alt={`Ảnh bìa ${videoLessonTitle(lesson)}`}
          fill
          unoptimized
          sizes="(min-width: 768px) 220px, 45vw"
          className="object-cover transition-transform duration-300 group-hover:scale-[1.03]"
        />
        <div className="absolute inset-0 flex items-center justify-center bg-black/0 transition-colors group-hover:bg-black/10">
          <CirclePlay
            className="size-9 text-white drop-shadow group-hover:scale-105"
            aria-hidden
          />
        </div>
        {duration && (
          <span className="absolute bottom-1.5 right-1.5 rounded bg-black/70 px-1.5 py-0.5 text-[11px] font-medium tabular-nums text-white">
            {duration}
          </span>
        )}
        {lesson.hasVi && (
          <Badge className="absolute top-1.5 left-1.5 gap-1 shadow-sm" variant="secondary">
            <Languages className="size-3" aria-hidden />
            Có phụ đề Việt
          </Badge>
        )}
      </div>

      <div className="flex flex-col gap-0.5 p-3">
        <h3 className="text-sm leading-tight font-medium">{videoLessonTitle(lesson)}</h3>
        <p className="font-korean truncate text-xs text-muted-foreground">{lesson.showTitle}</p>
      </div>
    </Link>
  );
}
