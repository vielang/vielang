import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { excerpt, pageMetadata } from "@/lib/seo";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { BackLink } from "@/components/layout/back-link";
import { VideoPlayer } from "@/components/video/video-player";
import {
  adjacentVideoLessons,
  getVideoLesson,
  getVideoLessons,
  videoLessonTitle,
} from "@/lib/videos";
import { mediaOriginBase } from "@/lib/books";

export function generateStaticParams() {
  return getVideoLessons().map((v) => ({ id: v.id }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ id: string }>;
}): Promise<Metadata> {
  const lesson = getVideoLesson((await params).id);
  if (!lesson) return { title: "Không tìm thấy trang" };
  return pageMetadata({
    title: `${videoLessonTitle(lesson)} — Học tiếng Hàn qua video`,
    description: excerpt(
      `Xem ${videoLessonTitle(lesson)} với phụ đề tiếng Hàn${lesson.hasVi ? " và tiếng Việt" : ""}: ${lesson.koCues.slice(0, 4).map((c) => c.t).join(" ")}`
    ),
    path: `/video/${lesson.id}`,
  });
}

export default async function VideoPlayerPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const lesson = getVideoLesson(id);
  if (!lesson) notFound();
  const { prev, next } = adjacentVideoLessons(id);

  const info = (
    <div className="flex flex-col gap-2 px-4 py-3 lg:px-0 lg:py-4">
      <div className="hidden lg:block">
        <BackLink href="/video">Học tiếng Hàn qua video</BackLink>
      </div>
      <div className="flex items-baseline justify-between gap-3">
        <div className="min-w-0">
          <h1 className="text-base font-semibold tracking-tight lg:text-xl">{videoLessonTitle(lesson)}</h1>
          <p className="font-korean truncate text-xs text-muted-foreground lg:text-sm">{lesson.showTitle}</p>
        </div>
        <Link href="/video" className="shrink-0 text-xs text-muted-foreground hover:text-foreground lg:hidden">
          Tất cả tập
        </Link>
      </div>
      <div className="flex items-center justify-between gap-3 text-sm">
        {prev ? (
          <Link
            href={`/video/${prev.id}`}
            className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground"
          >
            <ChevronLeft className="size-4 shrink-0" aria-hidden />
            {videoLessonTitle(prev)}
          </Link>
        ) : (
          <span />
        )}
        {next ? (
          <Link
            href={`/video/${next.id}`}
            className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground"
          >
            {videoLessonTitle(next)}
            <ChevronRight className="size-4 shrink-0" aria-hidden />
          </Link>
        ) : (
          <span />
        )}
      </div>
    </div>
  );

  return (
    <>
      <link rel="preconnect" href={mediaOriginBase()} />
      {/* `key`: chuyển tập thì dựng lại player từ đầu (vị trí, câu đang lặp…). */}
      <VideoPlayer key={lesson.id} lesson={lesson} info={info} />
    </>
  );
}
