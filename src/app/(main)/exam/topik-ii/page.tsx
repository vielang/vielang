import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { ExamLevelPage } from "@/components/exam/exam-level-page";

export const metadata: Metadata = pageMetadata({
  title: "Luyện thi TOPIK II – đề thi thật có chấm điểm",
  description: "Làm đề TOPIK II các kỳ thi thật ngay trên trình duyệt: nghe, đọc, viết, tự động chấm điểm và quy ra cấp. Giao diện tiếng Việt.",
  path: "/exam/topik-ii",
});

export default function Page() {
  return <ExamLevelPage level="TOPIK II" />;
}
