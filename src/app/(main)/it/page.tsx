import type { Metadata } from "next";
import { Code2 } from "lucide-react";
import { courseCard, listCourses } from "@/lib/courses";
import { CourseList } from "@/components/it/course-list";
import { PageHeader } from "@/components/layout/page-header";
import { EmptyState } from "@/components/layout/empty-state";

export const metadata: Metadata = {
  title: "Thư viện IT",
};

/**
 * Trang mảng IT — lộ trình .NET developer: C# Core → OOP → SQL/Oracle →
 * ASP.NET Core → WinForms → DSA → kiến trúc. Route tĩnh `/it` nên nó thắng `[lang]`.
 */
export default function ItLibraryPage() {
  const courses = listCourses().map(courseCard);

  return (
    <div className="flex flex-col gap-6">
      {/* Cùng khung đầu trang với các mảng khác của Thư viện — xem LibraryView. */}
      <PageHeader
        title="Thư viện IT"
        subtitle="Lộ trình .NET developer — học lần lượt từ trên xuống, khoá trước là nền cho khoá sau. Giải thích bằng tiếng Việt, thuật ngữ giữ nguyên tiếng Anh để bạn đọc được tài liệu gốc và đi phỏng vấn."
      />

      {courses.length === 0 ? (
        <EmptyState
          dashed
          icon={Code2}
          title="Sắp có nội dung"
          description="Các khoá học sẽ hiện ở đây theo lộ trình, mỗi khoá gồm nhiều chương và bài học ngắn."
        />
      ) : (
        <CourseList courses={courses} />
      )}
    </div>
  );
}
