import type { Metadata } from "next";
import { listExams } from "@/lib/exams";
import { ExamList } from "@/components/exam/exam-list";

export const metadata: Metadata = { title: "Luyện thi" };

/**
 * Tab "Luyện thi". Mỗi KỲ THI là một mục riêng — nay mới có TOPIK, mai thêm
 * kỳ khác (IELTS…) thì thêm một mục nữa ở đây, thanh điều hướng giữ nguyên.
 *
 * Canh trái, cùng bề ngang danh sách với Cẩm nang — trước đây trang này hẹp
 * và canh giữa, lệch hẳn với các trang cấp một khác.
 */
export default function ExamListPage() {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <ExamList exams={listExams()} />
    </div>
  );
}
