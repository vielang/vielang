import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BOOKS, getBook } from "@/lib/books";
import { getNotePages } from "@/lib/notes";
import { BookDetailHeader } from "@/components/library/book-detail-header";
import { PageGrid } from "@/components/library/page-grid";

export function generateStaticParams() {
  return BOOKS.map((book) => ({ bookId: book.id }));
}

// Danh sách sách cố định, chỉ thêm qua code + deploy lại — bookId không có
// trong generateStaticParams luôn là không hợp lệ, trả 404 thay vì render
// on-demand (tránh Next trả 200 kèm nội dung "not found").
export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ bookId: string }>;
}): Promise<Metadata> {
  const { bookId } = await params;
  const book = getBook(bookId);
  return { title: book ? book.titleVi : "Không tìm thấy sách" };
}

export default async function BookDetailPage({
  params,
}: {
  params: Promise<{ bookId: string }>;
}) {
  const { bookId } = await params;
  const book = getBook(bookId);
  if (!book) notFound();

  return (
    <div className="flex flex-col gap-6">
      <BookDetailHeader book={book} />
      <PageGrid
        bookId={book.id}
        totalPages={book.totalPages}
        notePages={getNotePages(book.id)}
      />
    </div>
  );
}
