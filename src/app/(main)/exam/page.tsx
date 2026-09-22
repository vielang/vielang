import type { Metadata } from "next";
import { listExams } from "@/lib/exams";
import { ExamList } from "@/components/exam/exam-list";

export const metadata: Metadata = { title: "Luyện thi TOPIK" };

/** Tab "Luyện thi": danh sách đề TOPIK để luyện từng câu hoặc thi thử. */
export default function ExamListPage() {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Luyện thi TOPIK</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Đề thi thật đã công bố. Luyện từng câu và chấm ngay, hoặc thi thử có tính giờ như
          phòng thi rồi xem mình đạt cấp mấy.
        </p>
      </div>
      <ExamList exams={listExams()} />
    </div>
  );
}
