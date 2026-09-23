import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { courseLessons, getCourse, getLesson, listCourses } from "@/lib/courses";
import { LessonView } from "@/components/it/lesson-view";

// Slug của bài là "<chương>/<bài>" nên route dùng catch-all: một mảng 2 đoạn.
export function generateStaticParams() {
  return listCourses().flatMap((course) =>
    courseLessons(course).map((l) => ({ course: course.id, lesson: l.slug.split("/") }))
  );
}

export const dynamicParams = false;

async function resolve(params: Promise<{ course: string; lesson: string[] }>) {
  const { course: courseId, lesson } = await params;
  const course = getCourse(courseId);
  if (!course) return undefined;
  const found = getLesson(course, lesson.join("/"));
  return found ? { course, lesson: found } : undefined;
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ course: string; lesson: string[] }>;
}): Promise<Metadata> {
  const hit = await resolve(params);
  return {
    title: hit ? `${hit.lesson.title} — ${hit.course.title}` : "Không tìm thấy bài học",
  };
}

export default async function LessonPage({
  params,
}: {
  params: Promise<{ course: string; lesson: string[] }>;
}) {
  const hit = await resolve(params);
  if (!hit) notFound();

  return <LessonView course={hit.course} lesson={hit.lesson} />;
}
