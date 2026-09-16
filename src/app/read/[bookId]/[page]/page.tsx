import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getBook, isValidPage } from "@/lib/books";
import { getNoteContent } from "@/lib/notes";
import { ReaderView } from "@/components/reader/reader-view";

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
  const { book, page } = resolved;
  return { title: `Trang ${page}/${book.totalPages} — ${book.titleVi}` };
}

export default async function ReadPage({ params }: { params: Promise<Params> }) {
  const resolved = resolve(await params);
  if (!resolved) notFound();
  const { book, page } = resolved;
  // Chuẩn bị sẵn note của CẢ 2 trang có thể hiện (trang này + trang kế) —
  // `page` luôn là trang trái/anchor khi ở chế độ 2 trang (xem
  // reader-view.tsx), nên không cần biết trước client đang xem 1 hay 2 trang.
  // `getNoteContent` chỉ là tra cứu JSON tĩnh trong bộ nhớ, gọi thêm 1 lần
  // không đáng kể.
  const noteContentByPage: Record<number, string | null> = {
    [page]: getNoteContent(book.id, page),
    [page + 1]: getNoteContent(book.id, page + 1),
  };

  return (
    <ReaderView
      key={page}
      book={book}
      page={page}
      noteContentByPage={noteContentByPage}
    />
  );
}
