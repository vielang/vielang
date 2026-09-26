import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { examTitle, getExam, listExams, sectionCount, sectionVi, totalMinutes } from "@/lib/exams";
import { ExamOverview } from "@/components/exam/exam-overview";

export function generateStaticParams() {
  return listExams().map((e) => ({ examId: e.id }));
}
export const dynamicParams = false;

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  if (!exam) return { title: "Không tìm thấy đề" };
  const parts = exam.sections.map((sec) => `${sectionVi(sec.id).toLowerCase()} ${sectionCount(sec)} câu`).join(", ");
  return pageMetadata({
    title: `${examTitle(exam)} – làm online, chấm điểm, có giải thích`,
    description: `${examTitle(exam)} (${parts}, ${totalMinutes(exam)} phút): luyện từng câu có giải thích tiếng Việt, thi thử tính giờ, quy đổi điểm TOEIC ước tính.`,
    path: `/exam/${exam.id}`,
  });
}

export default async function ExamPage({ params }: { params: Promise<{ examId: string }> }) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <ExamOverview exam={exam} source={exam.source} />;
}
