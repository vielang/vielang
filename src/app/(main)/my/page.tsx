import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { courseCard, listCourses } from "@/lib/courses";
import { MyPageView } from "@/components/my/my-page-view";
import { PageHeader } from "@/components/layout/page-header";

export const metadata: Metadata = pageMetadata({ title: "Góc học tập của bạn", path: "/my", noindex: true });

/**
 * My page: tiến độ, lịch học và kết quả bài tập của người dùng.
 *
 * Danh sách khoá học render ở server (cố định, nằm trong code) —
 * chỉ gửi thẻ tóm tắt khoá học, không gửi nội dung bài; mọi con số về
 * người học thì chỉ trình duyệt biết — xem `MyPageView`.
 */
export default function MyPage() {
  return (
    // Cùng khung đầu trang và cùng canh trái với các tab khác (trước đây canh giữa).
    <div className="flex max-w-3xl flex-col gap-6">
      <PageHeader title="Góc học tập" subtitle="Tiến độ khoá học, lịch học và sao lưu dữ liệu." />
      <MyPageView courses={listCourses().map(courseCard)} />
    </div>
  );
}
