import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { BOOKS } from "@/lib/books";
import { BookmarkList } from "@/components/library/bookmark-list";
import { PageHeader } from "@/components/layout/page-header";

export const metadata: Metadata = { title: "Trang đã đánh dấu" };

/**
 * Trang đã đánh dấu, gom theo sách — nằm TRONG Góc học tập (`/my/danh-dau`)
 * chứ không còn là một tab riêng: ghim trang là dữ liệu học tập cá nhân,
 * cùng loại với tiến độ đọc và câu sai cần ôn. Góc học tập bày sẵn một hàng
 * ảnh thu nhỏ (xem `BookmarkStrip`), trang này là danh sách đầy đủ.
 * `/bookmarks` cũ chuyển thẳng tới đây (xem `redirects` trong next.config.ts).
 *
 * Danh sách sách render ở server (nó cố định, nằm trong code), còn trang nào
 * được ghim thì chỉ trình duyệt biết — xem `BookmarkList`.
 */
export default function BookmarksPage() {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <Link
        href="/my"
        className="inline-flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        Góc học tập
      </Link>
      <div className="-mt-2">
        <PageHeader title="Trang đã đánh dấu" subtitle="Những trang bạn ghim lại để xem kỹ, gom theo từng cuốn." />
      </div>
      <BookmarkList books={BOOKS} />
    </div>
  );
}
