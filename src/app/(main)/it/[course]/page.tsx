import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { getCourse, listCourses } from "@/lib/courses";
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
  return { title: found ? found.title : "Không tìm thấy khoá học" };
}

export default async function CoursePage({ params }: { params: Promise<{ course: string }> }) {
  const { course } = await params;
  const found = getCourse(course);
  if (!found) notFound();

  return <CourseOverview course={found} />;
}
