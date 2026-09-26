import type { Metadata } from "next";
import { SITE_DESCRIPTION, pageMetadata } from "@/lib/seo";
import { Code2 } from "lucide-react";
import { courseCard, listCourses } from "@/lib/courses";
import { CourseList } from "@/components/it/course-list";
import { PageHeader } from "@/components/layout/page-header";
import { EmptyState } from "@/components/layout/empty-state";

const TITLE = "VieLang – Học lập trình và tiếng Anh bằng tiếng Việt";

export const metadata: Metadata = {
  ...pageMetadata({ title: TITLE, description: SITE_DESCRIPTION, path: "/" }),
  // Trang chủ không ghép đuôi "| VieLang" — tên đã nằm đầu tiêu đề.
  title: { absolute: TITLE },
};

/**
 * Trang chủ = mảng IT, mảng mặc định của Thư viện — lộ trình .NET developer:
 * C# Core → OOP → SQL/Oracle → ASP.NET Core → WinForms → DSA → kiến trúc.
 * Tiếng Anh ở `/en`. Link cũ `/it` chuyển về đây (next.config.ts).
 */
export default function HomePage() {
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
