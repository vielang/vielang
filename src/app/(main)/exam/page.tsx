import type { Metadata } from "next";
import { listExams } from "@/lib/exams";
import { ExamList } from "@/components/exam/exam-list";

export const metadata: Metadata = { title: "Luyện thi" };

/**
 * Tab "Luyện thi". Mỗi KỲ THI là một mục riêng — nay mới có TOPIK, mai thêm
 * kỳ khác (IELTS…) thì thêm một mục nữa ở đây, thanh điều hướng giữ nguyên.
 */
export default function ExamListPage() {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-8">
      <h1 className="text-2xl font-semibold tracking-tight">Luyện thi</h1>
      <section className="flex flex-col gap-4">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">TOPIK — tiếng Hàn</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Đề thi thật đã công bố. Luyện từng câu và chấm ngay, hoặc thi thử có tính giờ như
            phòng thi rồi xem mình đạt cấp mấy.
          </p>
        </div>
        <ExamList exams={listExams()} />
      </section>
    </div>
  );
}
