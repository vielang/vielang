import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { notFound } from "next/navigation";
import { getBook, isValidPage } from "@/lib/books";
import { getNoteContent } from "@/lib/notes";
import { NoteWindowView } from "@/components/reader/note-window-view";

interface Params {
  bookId: string;
  page: string;
}

function resolve(params: Params) {
  const book = getBook(params.bookId);
  const page = Number(params.page);
  if (!book || !isValidPage(book, page)) return null;
  return { book, page };
}

export async function generateMetadata({
  params,
}: {
  params: Promise<Params>;
}): Promise<Metadata> {
  const resolved = resolve(await params);
  if (!resolved) return { title: "Không tìm thấy trang" };
  // Cửa sổ phụ trùng nội dung trang đọc — không cho vào kết quả tìm kiếm.
  return pageMetadata({
    title: `Bài giảng trang ${resolved.page}`,
    path: `/read/${resolved.book.id}/${resolved.page}/note`,
    noindex: true,
  });
}

/**
 * Cửa sổ bài giảng riêng — mở bằng `window.open` từ Reader để kéo sang màn
 * hình thứ hai, vừa xem trang sách bên cửa sổ chính vừa đọc/sửa bài giảng ở
 * đây. Đồng bộ 2 chiều qua localStorage, xem lib/note-window.ts.
 */
export default async function NoteWindowPage({
  params,
}: {
  params: Promise<Params>;
}) {
  const resolved = resolve(await params);
  if (!resolved) notFound();
  const { book, page } = resolved;

  return (
    <NoteWindowView
      book={book}
      page={page}
      originalContent={getNoteContent(book.id, page)}
    />
  );
}
