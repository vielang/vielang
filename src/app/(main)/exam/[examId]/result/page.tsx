import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { examTitle, getExam } from "@/lib/exams";
import { ResultView } from "@/components/exam/result-view";

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  if (!exam) return { title: "Không tìm thấy đề" };
  // Màn làm bài / chấm điểm — không có gì để tìm, trang đề chính mới là
  // trang cho Google (xem app/robots.ts).
  return pageMetadata({ title: `Kết quả · ${examTitle(exam)}`, path: `/exam/${exam.id}/result`, noindex: true });
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
