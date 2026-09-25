import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { examTitle, getExam, listExams } from "@/lib/exams";
import { MockView } from "@/components/exam/mock-view";

// Dựng sẵn như trang tổng quan đề: trang này không đọc tham số URL nào nên
// không có lý do dựng lại ở mỗi lần mở. (Trang luyện tập / kết quả đọc
// `?section=` / `?attempt=` nên vẫn phải dựng lúc chạy.)
export function generateStaticParams() {
  return listExams().map((e) => ({ examId: e.id }));
}
export const dynamicParams = false;

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  return { title: exam ? `Thi thử · ${examTitle(exam)}` : "Không tìm thấy đề" };
}

export default async function MockPage({ params }: { params: Promise<{ examId: string }> }) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <MockView exam={exam} />;
}
