import type { Metadata } from "next";
import { ExamLevelPage } from "@/components/exam/exam-level-page";

export const metadata: Metadata = { title: "Luyện thi TOPIK I" };

export default function Page() {
  return <ExamLevelPage level="TOPIK I" />;
}
