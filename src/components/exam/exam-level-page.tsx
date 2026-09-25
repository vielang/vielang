import { listExams, type ExamLevel } from "@/lib/exams";
import { ExamList } from "@/components/exam/exam-list";

/**
 * Tab "Luyện thi", một trang cho mỗi cấp đề (`/exam/topik-i`, `/exam/topik-ii`
 * — `/exam` chuyển thẳng tới cấp đầu, xem next.config). Nay mới có TOPIK;
 * thêm kỳ thi khác thì thêm tầng kỳ thi phía trên, thanh điều hướng giữ nguyên.
 *
 * Canh trái, cùng bề ngang danh sách với Cẩm nang.
 */
export function ExamLevelPage({ level }: { level: ExamLevel }) {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <ExamList exams={listExams()} level={level} />
    </div>
  );
}
