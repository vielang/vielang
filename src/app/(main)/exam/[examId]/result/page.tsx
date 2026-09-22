import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { examTitle, getExam } from "@/lib/exams";
import { ResultView } from "@/components/exam/result-view";

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  return { title: exam ? `Kết quả · ${examTitle(exam)}` : "Không tìm thấy đề" };
}

export default async function ResultPage({
  params,
  searchParams,
}: {
  params: Promise<{ examId: string }>;
  searchParams: Promise<{ attempt?: string }>;
}) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <ResultView exam={exam} attemptId={(await searchParams).attempt} />;
}
