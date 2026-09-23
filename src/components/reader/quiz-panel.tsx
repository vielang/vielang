"use client";

import { QuizBody } from "@/components/quiz/quiz-body";
import { noteKey } from "@/lib/note-store";
import type { QuizSection } from "@/lib/quiz";

/**
 * Chỗ nộp đáp án cho BÀI TẬP IN TRONG SÁCH của trang đang mở — người dùng
 * nhìn đề trên ảnh trang, làm ở đây. Cấu trúc bám theo sách: mỗi
 * `QuizSection` là 1 mục thật ("연습 1", "읽기", "쓰기"...).
 *
 * Chấm TỪNG CÂU ngay khi bấm, không gom lại thành "nộp bài": người tự học thì
 * phản hồi ngay dạy được nhiều hơn — sai câu 1 thì hiểu và sửa trước khi làm
 * câu 2. Dòng tổng kết ở đầu vẫn cho cảm giác hoàn thành.
 *
 * Phần thân nằm ở `components/quiz/quiz-body` vì bài học IT dùng chung; ở đây
 * chỉ còn cái khung cuộn của panel bên cạnh trang sách, và việc dựng khoá lưu
 * `${bookId}:${page}` — đúng định dạng đã nằm sẵn trong máy người dùng.
 */
export function QuizPanel({
  bookId,
  page,
  sections,
}: {
  bookId: string;
  page: number;
  sections: QuizSection[];
}) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-6">
      <QuizBody quizId={noteKey(bookId, page)} sections={sections} />
    </div>
  );
}
