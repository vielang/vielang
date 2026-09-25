import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
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
  return { title: lesson ? `${videoLessonTitle(lesson)} — Học tiếng Hàn qua video` : "Không tìm thấy trang" };
}

export default async function VideoPlayerPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const lesson = getVideoLesson(id);
  if (!lesson) notFound();
  const { prev, next } = adjacentVideoLessons(id);

  return (
    <div className="flex flex-col gap-4">
      <link rel="preconnect" href={mediaOriginBase()} />
      <BackLink href="/video">Học tiếng Hàn qua video</BackLink>

      <div>
        <h1 className="text-xl font-semibold tracking-tight">{videoLessonTitle(lesson)}</h1>
        <p className="font-korean text-sm text-muted-foreground">{lesson.showTitle}</p>
      </div>

      <VideoPlayer lesson={lesson} />

      <div className="flex items-center justify-between gap-3 border-t border-border pt-3 text-sm">
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
}
