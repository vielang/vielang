import type { Metadata } from "next";
import { BOOKS } from "@/lib/books";
import { BookmarkList } from "@/components/library/bookmark-list";

export const metadata: Metadata = { title: "Trang đã đánh dấu" };

/**
 * Trang đã đánh dấu, gom theo sách.
 *
 * Danh sách sách render ở server (nó cố định, nằm trong code), còn trang nào
 * được ghim thì chỉ trình duyệt biết — xem `BookmarkList`.
 */
export default function BookmarksPage() {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Trang đã đánh dấu</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Những trang bạn ghim lại để xem kỹ, gom theo từng cuốn.
        </p>
      </div>
      <BookmarkList books={BOOKS} />
    </div>
  );
}
