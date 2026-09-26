import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BookOpen, ChevronLeft, ChevronRight } from "lucide-react";
import { BOOKS } from "@/lib/books";
import { bookLessonHref, bookLessons, bookSearchName, getBookLesson } from "@/lib/book-lessons";
import { grammarHref, grammarPattern } from "@/lib/page-grammar";
import { absoluteUrl, breadcrumbLd, excerpt, pageMetadata } from "@/lib/seo";
import { BackLink } from "@/components/layout/back-link";
import { JsonLd } from "@/components/seo/json-ld";

type Params = { bookId: string; lesson: string };

export function generateStaticParams(): Params[] {
  return BOOKS.flatMap((book) =>
    bookLessons(book.id).map((l) => ({ bookId: book.id, lesson: String(l.chapter.lesson) }))
  );
}

export const dynamicParams = false;

async function resolve(params: Promise<Params>) {
  const { bookId, lesson } = await params;
  return getBookLesson(bookId, Number(lesson));
}

export async function generateMetadata({ params }: { params: Promise<Params> }): Promise<Metadata> {
  const hit = await resolve(params);
  if (!hit) return { title: "Không tìm thấy bài" };
  const { book, chapter, pages } = hit;
  const grammar = pages.flatMap((p) => p.grammar.map((g) => grammarPattern(g.title)));
  const firstVi = pages.flatMap((p) => p.translations)[0]?.vi ?? "";
  return pageMetadata({
    title: `${bookSearchName(book)} Bài ${chapter.lesson} – Dịch tiếng Việt${grammar.length ? `, ngữ pháp ${grammar.join(", ")}` : ""}`,
    description: excerpt(
      `Dịch tiếng Việt bài ${chapter.lesson} ${book.titleVi} (${book.titleKo ?? ""}): hội thoại, bài đọc và ngữ pháp, đối chiếu tiếng Hàn. ${firstVi}`
    ),
    path: bookLessonHref(book.id, chapter.lesson),
    type: "article",
  });
}

/**
 * Bài N của một cuốn: toàn bộ bản dịch và ngữ pháp trong bài, dạng chữ đọc
 * liền mạch — xem `lib/book-lessons.ts` để biết vì sao có trang này.
 */
export default async function BookLessonPage({ params }: { params: Promise<Params> }) {
  const hit = await resolve(params);
  if (!hit) notFound();
  const { book, chapter, pages } = hit;
  const all = bookLessons(book.id);
  const i = all.findIndex((l) => l.chapter.lesson === chapter.lesson);
  const prev = all[i - 1];
  const next = all[i + 1];
  const path = bookLessonHref(book.id, chapter.lesson);
  const name = `${bookSearchName(book)} – Bài ${chapter.lesson}`;

  const ld = [
    breadcrumbLd([
      { name: "Thư viện", path: "/" },
      { name: book.titleVi, path: `/books/${book.id}` },
      { name: `Bài ${chapter.lesson}`, path },
    ]),
    {
      "@context": "https://schema.org",
      "@type": "LearningResource",
      name,
      url: absoluteUrl(path),
      inLanguage: ["vi", "ko"],
      learningResourceType: "Translation",
      educationalLevel: bookSearchName(book),
      isPartOf: { "@type": "Book", name: book.titleKo ?? book.titleVi },
    },
  ];

  return (
    <article className="mx-auto flex w-full max-w-2xl flex-col gap-8">
      <JsonLd data={ld} />
      <div className="flex flex-col gap-3">
        <BackLink href={`/books/${book.id}`}>{book.titleVi}</BackLink>
        <header className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">
            {book.titleKo} · trang {chapter.startPage}–{chapter.endPage}
          </p>
          <h1 className="text-2xl font-semibold tracking-tight">{name}: dịch tiếng Việt và ngữ pháp</h1>
        </header>
      </div>

      {pages.map(({ page, translations, grammar }) => (
        <section key={page} className="flex flex-col gap-4" aria-labelledby={`p${page}`}>
          <div className="flex items-baseline justify-between gap-3 border-b border-border pb-2">
            <h2 id={`p${page}`} className="text-lg font-semibold">
              Trang {page}
            </h2>
            <Link
              href={`/read/${book.id}/${page}`}
              className="inline-flex items-center gap-1 text-sm text-primary hover:underline"
            >
              <BookOpen className="size-4" aria-hidden />
              Mở trong sách
            </Link>
          </div>

          {grammar.map((g) => (
            <div key={g.id} className="rounded-xl border border-border/70 px-4 py-3">
              <h3 className="font-korean font-semibold">
                <Link href={grammarHref(g.slug)} className="hover:underline">
                  Ngữ pháp {g.title}
                </Link>
              </h3>
              <p className="mt-1 text-sm leading-relaxed">{g.vi}</p>
              <p className="font-korean mt-2 text-sm text-muted-foreground">{g.exKo}</p>
              <p className="text-sm text-muted-foreground/80">{g.exVi}</p>
            </div>
          ))}

          {translations.map((t) => (
            <div key={t.id} className="flex flex-col gap-2">
              {t.label && <h3 className="text-sm font-semibold text-muted-foreground">{t.label}</h3>}
              {t.ko && <p className="font-korean whitespace-pre-line text-muted-foreground">{t.ko}</p>}
              <p className="whitespace-pre-line leading-relaxed">{t.vi}</p>
            </div>
          ))}
        </section>
      ))}

      <nav aria-label="Bài trước / sau" className="grid grid-cols-2 gap-3 border-t border-border pt-4">
        {prev ? (
          <Link
            href={bookLessonHref(book.id, prev.chapter.lesson)}
            className="inline-flex items-center gap-1 rounded-lg p-2 text-sm hover:bg-muted"
          >
            <ChevronLeft className="size-4" aria-hidden /> Bài {prev.chapter.lesson}
          </Link>
        ) : (
          <span />
        )}
        {next && (
          <Link
            href={bookLessonHref(book.id, next.chapter.lesson)}
            className="inline-flex items-center justify-end gap-1 rounded-lg p-2 text-sm hover:bg-muted"
          >
            Bài {next.chapter.lesson} <ChevronRight className="size-4" aria-hidden />
          </Link>
        )}
      </nav>
    </article>
  );
}
