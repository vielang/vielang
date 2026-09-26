import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { courseOutline, getCourse, listCourses } from "@/lib/courses";
import { CourseOverview } from "@/components/it/course-overview";

export function generateStaticParams() {
  return listCourses().map((c) => ({ course: c.id }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ course: string }>;
}): Promise<Metadata> {
  const { course } = await params;
  const found = getCourse(course);
  if (!found) return { title: "Không tìm thấy khoá học" };
  return pageMetadata({ title: found.title, description: found.summary, path: `/it/${found.id}` });
}

export default async function CoursePage({ params }: { params: Promise<{ course: string }> }) {
  const { course } = await params;
  const found = getCourse(course);
  if (!found) notFound();

  return <CourseOverview course={courseOutline(found)} />;
}
