import { listExams, type ExamLevel } from "@/lib/exams";
import { ExamList } from "@/components/exam/exam-list";

/**
 * Tab "Luyện thi", một trang cho mỗi kỳ thi (`/exam/toeic` — `/exam` chuyển
 * thẳng tới đó, xem next.config). Thêm kỳ thi khác thì thêm vào
 * `EXAM_LEVELS` và một route như `exam/toeic`, thanh điều hướng giữ nguyên.
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
