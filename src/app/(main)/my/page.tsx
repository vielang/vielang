import type { Metadata } from "next";
import { BOOKS } from "@/lib/books";
import { MyPageView } from "@/components/my/my-page-view";

export const metadata: Metadata = { title: "Góc học tập của bạn" };

/**
 * My page: tiến độ, lịch học và kết quả bài tập của người dùng.
 *
 * Danh sách sách render ở server (cố định, nằm trong code); mọi con số về
 * người học thì chỉ trình duyệt biết — xem `MyPageView`.
 */
export default function MyPage() {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Góc học tập của bạn</h1>
      </div>
      <MyPageView books={BOOKS} />
    </div>
  );
}
