import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { excerpt, pageMetadata } from "@/lib/seo";
import {
  courseLessons,
  courseOutline,
  getCourse,
  getLesson,
  lessonHref,
  lessonNeighbours,
  lessonNumber,
  listCourses,
} from "@/lib/courses";
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
  if (!hit) return { title: "Không tìm thấy bài học" };
  return pageMetadata({
    title: `${hit.lesson.title} — ${hit.course.title}`,
    // Đoạn mở đầu bài viết — đúng thứ người đọc thấy đầu tiên.
    description: excerpt(hit.lesson.html),
    path: lessonHref(hit.course.id, hit.lesson.slug),
    type: "article",
  });
}

export default async function LessonPage({
  params,
}: {
  params: Promise<{ course: string; lesson: string[] }>;
}) {
  const hit = await resolve(params);
  if (!hit) notFound();

  const { course, lesson } = hit;
  const { prev, next } = lessonNeighbours(course, lesson.slug);
  const moduleTitle =
    courseOutline(course).modules.find((m) => m.lessons.some((l) => l.slug === lesson.slug))
      ?.title ?? "";

  return (
    <LessonView
      courseId={course.id}
      courseTitle={course.title}
      moduleTitle={moduleTitle}
      total={courseLessons(course).length}
      no={lessonNumber(course, lesson.slug)}
      lesson={lesson}
      prev={prev}
      next={next}
    />
  );
}
