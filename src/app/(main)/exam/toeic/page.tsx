import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { ExamLevelPage } from "@/components/exam/exam-level-page";

export const metadata: Metadata = pageMetadata({
  title: "Luyện thi TOEIC online – đề luyện có giải thích, chấm điểm",
  description:
    "Làm đề luyện TOEIC Listening & Reading ngay trên trình duyệt: luyện từng câu có giải thích tiếng Việt, thi thử tính giờ 75 phút, quy đổi điểm ước tính.",
  path: "/exam/toeic",
});

export default function Page() {
  return <ExamLevelPage level="TOEIC" />;
}
