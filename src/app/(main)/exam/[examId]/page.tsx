import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { examTitle, getExam, listExams } from "@/lib/exams";
import { ExamOverview } from "@/components/exam/exam-overview";

export function generateStaticParams() {
  return listExams().map((e) => ({ examId: e.id }));
}
export const dynamicParams = false;

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  return { title: exam ? examTitle(exam) : "Không tìm thấy đề" };
}

export default async function ExamPage({ params }: { params: Promise<{ examId: string }> }) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <ExamOverview exam={exam} source={exam.source} />;
}
