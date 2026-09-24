import type { Metadata } from "next";
import { Code2 } from "lucide-react";
import { courseCard, listCourses } from "@/lib/courses";
import { CourseList } from "@/components/it/course-list";
import { TrackNav } from "@/components/layout/track-nav";

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
    <div className="flex flex-col gap-8">
      <TrackNav />

      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Lộ trình .NET developer</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Học lần lượt từ trên xuống, khoá trước là nền cho khoá sau. Giải thích bằng tiếng Việt,
          thuật ngữ giữ nguyên tiếng Anh để bạn đọc được tài liệu gốc và đi phỏng vấn.
        </p>
      </div>

      {courses.length === 0 ? (
        <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed border-border py-24 text-center">
          <Code2 className="size-10 text-muted-foreground" aria-hidden />
          <div>
            <h2 className="text-lg font-semibold">Sắp có nội dung</h2>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Các khoá học sẽ hiện ở đây theo lộ trình, mỗi khoá gồm nhiều chương và bài học ngắn.
            </p>
          </div>
        </div>
      ) : (
        <CourseList courses={courses} />
      )}
    </div>
  );
}
