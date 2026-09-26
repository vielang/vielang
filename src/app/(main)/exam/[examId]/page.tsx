import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { getExam, listExams } from "@/lib/exams";
import { ExamOverview } from "@/components/exam/exam-overview";

export function generateStaticParams() {
  return listExams().map((e) => ({ examId: e.id }));
}
export const dynamicParams = false;

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  if (!exam) return { title: "Không tìm thấy đề" };
  return pageMetadata({
    title: `Đề thi ${exam.level} kỳ ${exam.round} (${exam.year}) – làm online, chấm điểm`,
    description: `Đề thi ${exam.level} lần thứ ${exam.round} năm ${exam.year}: làm bài online có file nghe, chấm điểm tự động và quy ra cấp TOPIK.`,
    path: `/exam/${exam.id}`,
  });
}

export default async function ExamPage({ params }: { params: Promise<{ examId: string }> }) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <ExamOverview exam={exam} source={exam.source} />;
}
