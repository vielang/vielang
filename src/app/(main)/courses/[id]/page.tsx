import type { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { getPublishedCourseDetail } from '@/lib/supabase';
import { CourseDetailClient } from './CourseDetailClient';

export const dynamic = 'force-dynamic';

interface Props {
  params: Promise<{ id: string }>;
}

// Dynamic metadata so shared /courses/<id> links preview with the actual
// course title instead of a generic string.
export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { id } = await params;
  const course = await getPublishedCourseDetail(id).catch(() => null);
  if (!course) return { title: 'Course · VieLang' };
  const title = course.title_en || course.title_vn;
  return {
    title: `${title} · VieLang`,
    description:
      course.description_en ||
      course.description_vn ||
      `Learn ${title} with a certified VieLang tutor.`,
  };
}

export default async function CoursePage({ params }: Props) {
  const { id } = await params;
  const course = await getPublishedCourseDetail(id).catch(() => null);
  if (!course) notFound();
  return <CourseDetailClient course={course} />;
}
