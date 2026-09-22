import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { examTitle, getExam, type SectionId } from "@/lib/exams";
import { PracticeView } from "@/components/exam/practice-view";

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  return { title: exam ? `Luyện tập · ${examTitle(exam)}` : "Không tìm thấy đề" };
}

export default async function PracticePage({
  params,
  searchParams,
}: {
  params: Promise<{ examId: string }>;
  searchParams: Promise<{ section?: string; q?: string }>;
}) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  const { section, q } = await searchParams;
  const sectionId = (exam.sections.find((s) => s.id === section)?.id ?? exam.sections[0].id) as SectionId;
  // `key` theo phần: đổi phần là dựng lại từ câu đầu, không giữ chỉ số câu của phần kia.
  return <PracticeView key={sectionId} exam={exam} sectionId={sectionId} initialNo={q ? Number(q) : undefined} />;
}
