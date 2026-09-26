import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BOOKS, getBook } from "@/lib/books";
import { getNotePages } from "@/lib/notes";
import { getAudioPages } from "@/lib/audio";
import { getTranslatedPages } from "@/lib/page-translation";
import { getGrammarPages } from "@/lib/page-grammar";
import { getAnswerPages } from "@/lib/page-answers";
import { getChapters } from "@/lib/chapters";
import { breadcrumbLd, pageMetadata } from "@/lib/seo";
import { BookDetailHeader } from "@/components/library/book-detail-header";
import { PageGrid } from "@/components/library/page-grid";
import { JsonLd } from "@/components/seo/json-ld";

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
  if (!book) return { title: "Không tìm thấy sách" };
  return pageMetadata({
    title: book.titleVi,
    description: `Đọc ${book.titleVi} online, ${book.totalPages} trang, có audio nghe theo trang, ghi chú và đánh dấu ngay trên sách.`,
    path: `/books/${book.id}`,
  });
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
      <JsonLd
        data={[
          breadcrumbLd([
            { name: "Thư viện", path: "/" },
            { name: book.titleVi, path: `/books/${book.id}` },
          ]),
          {
            "@context": "https://schema.org",
            "@type": "Book",
            name: book.titleKo ?? book.titleVi,
            alternateName: book.titleVi,
            inLanguage: book.lang,
            numberOfPages: book.totalPages,
          },
        ]}
      />
      <BookDetailHeader
        book={book}
        hasAnswers={getAnswerPages(book.id).length > 0}
        hasAudio={getAudioPages(book.id).length > 0}
      />
      <PageGrid
        bookId={book.id}
        totalPages={book.totalPages}
        chapters={getChapters(book.id, book.totalPages)}
        notePages={getNotePages(book.id)}
        audioPages={getAudioPages(book.id)}
        translatedPages={getTranslatedPages(book.id)}
        grammarPages={getGrammarPages(book.id)}
        answerPages={getAnswerPages(book.id)}
      />
    </div>
  );
}
