import type { Metadata } from "next";
import { BOOKS } from "@/lib/books";
import { MyPageView } from "@/components/my/my-page-view";

export const metadata: Metadata = { title: "Quá trình học của tôi" };

/**
 * My page: tiến độ, lịch học và kết quả bài tập của người dùng.
 *
 * Danh sách sách render ở server (cố định, nằm trong code); mọi con số về
 * người học thì chỉ trình duyệt biết — xem `MyPageView`.
 */
export default function MyPage() {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Quá trình học của tôi</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Thời gian học, sách đang học và kết quả bài tập — lưu ngay trên trình duyệt
          này, không cần tài khoản.
        </p>
      </div>
      <MyPageView books={BOOKS} />
    </div>
  );
}
