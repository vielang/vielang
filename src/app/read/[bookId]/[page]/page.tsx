import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getBook, isValidPage } from "@/lib/books";
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

  return <ReaderView key={page} book={book} page={page} />;
}
