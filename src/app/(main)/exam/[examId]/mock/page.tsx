import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { examTitle, getExam } from "@/lib/exams";
import { MockView } from "@/components/exam/mock-view";

export async function generateMetadata({ params }: { params: Promise<{ examId: string }> }): Promise<Metadata> {
  const exam = getExam((await params).examId);
  return { title: exam ? `Thi thử · ${examTitle(exam)}` : "Không tìm thấy đề" };
}

export default async function MockPage({ params }: { params: Promise<{ examId: string }> }) {
  const exam = getExam((await params).examId);
  if (!exam) notFound();
  return <MockView exam={exam} />;
}
