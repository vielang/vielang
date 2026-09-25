import type { Metadata } from "next";
import { BOOKS } from "@/lib/books";
import { MyPageView } from "@/components/my/my-page-view";
import { PageHeader } from "@/components/layout/page-header";

export const metadata: Metadata = { title: "Góc học tập của bạn" };

/**
 * My page: tiến độ, lịch học và kết quả bài tập của người dùng.
 *
 * Danh sách sách render ở server (cố định, nằm trong code); mọi con số về
 * người học thì chỉ trình duyệt biết — xem `MyPageView`.
 */
export default function MyPage() {
  return (
    // Cùng khung đầu trang và cùng canh trái với các tab khác (trước đây canh giữa).
    <div className="flex max-w-3xl flex-col gap-6">
      <PageHeader title="Góc học tập" subtitle="Tiến độ, lịch học, trang đã ghim và những gì nên ôn lại." />
      <MyPageView books={BOOKS} />
    </div>
  );
}
