import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BOOKS, getBook } from "@/lib/books";
import { getNotePages } from "@/lib/notes";
import { getAudioPages } from "@/lib/audio";
import { getTranslatedPages } from "@/lib/page-translation";
import { getGrammarPages } from "@/lib/page-grammar";
import { getAnswerPages } from "@/lib/page-answers";
import { getChapters } from "@/lib/chapters";
import { bookLessonHref, bookLessons, bookSearchName } from "@/lib/book-lessons";
import { breadcrumbLd, pageMetadata } from "@/lib/seo";
import { BookDetailHeader } from "@/components/library/book-detail-header";
import { PageGrid } from "@/components/library/page-grid";
import { SectionLabel } from "@/components/layout/page-header";
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
  const name = bookSearchName(book);
  const ko = book.titleKo ? ` (${book.titleKo})` : "";
  return pageMetadata({
    title: name === book.titleVi ? book.titleVi : `${name}: ${book.titleVi}${ko}`,
    description: `Đọc ${book.titleVi}${ko} online, ${book.totalPages} trang, có dịch tiếng Việt, giải thích ngữ pháp và audio theo từng bài.`,
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
  const lessons = bookLessons(book.id);

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
      {/* Bản chữ của từng bài — đọc nhanh không cần lật ảnh, và là chỗ để
          Google thấy nội dung (xem lib/book-lessons.ts). */}
      {lessons.length > 0 && (
        <section className="flex flex-col gap-3">
          <SectionLabel aside={`${lessons.length} bài`}>Dịch tiếng Việt theo bài</SectionLabel>
          <ul className="flex flex-wrap gap-2">
            {lessons.map(({ chapter }) => (
              <li key={chapter.lesson}>
                <Link
                  href={bookLessonHref(book.id, chapter.lesson)}
                  className="inline-flex rounded-lg border border-border px-3 py-1.5 text-sm tabular-nums hover:border-primary/40 hover:bg-muted/40"
                >
                  Bài {chapter.lesson}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      )}
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
