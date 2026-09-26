import Link from "next/link";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BookOpen, ChevronLeft, ChevronRight } from "lucide-react";
import { getBook } from "@/lib/books";
import { bookLessonHref, bookSearchName, lessonOfPage } from "@/lib/book-lessons";
import { getAllGrammar, getGrammarBySlug, grammarHref, grammarPattern } from "@/lib/page-grammar";
import { absoluteUrl, breadcrumbLd, excerpt, pageMetadata } from "@/lib/seo";
import { BackLink } from "@/components/layout/back-link";
import { JsonLd } from "@/components/seo/json-ld";

export function generateStaticParams() {
  return getAllGrammar().map((g) => ({ slug: g.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }): Promise<Metadata> {
  const hit = getGrammarBySlug((await params).slug);
  if (!hit) return { title: "Không tìm thấy ngữ pháp" };
  const { entry } = hit;
  const book = getBook(entry.bookId);
  return pageMetadata({
    title: `Ngữ pháp ${grammarPattern(entry.title)} – nghĩa, cách dùng, ví dụ${book ? ` (${bookSearchName(book)})` : ""}`,
    description: excerpt(`${entry.title}: ${entry.vi} Ví dụ: ${entry.exKo} — ${entry.exVi}`),
    path: grammarHref(entry.slug),
    type: "article",
  });
}

/**
 * Trang riêng cho một điểm ngữ pháp: nghĩa tiếng Việt, câu ví dụ trong sách,
 * đường sang đúng trang sách và bài dịch. Người Việt tìm ngữ pháp Hàn bằng
 * chính cái đuôi câu ("ngữ pháp 아서/어서") — mỗi điểm cần một URL để Google
 * trả về đúng chỗ.
 */
export default async function GrammarPage({ params }: { params: Promise<{ slug: string }> }) {
  const hit = getGrammarBySlug((await params).slug);
  if (!hit) notFound();
  const { entry, prev, next } = hit;
  const book = getBook(entry.bookId);
  const lesson = lessonOfPage(entry.bookId, entry.page);

  const ld = [
    breadcrumbLd([
      { name: "Thư viện", path: "/" },
      { name: "Ngữ pháp", path: "/ngu-phap" },
      { name: entry.title, path: grammarHref(entry.slug) },
    ]),
    {
      "@context": "https://schema.org",
      "@type": "LearningResource",
      name: `Ngữ pháp ${entry.title}`,
      description: entry.vi,
      url: absoluteUrl(grammarHref(entry.slug)),
      inLanguage: ["vi", "ko"],
      learningResourceType: "Grammar explanation",
      educationalLevel: book ? bookSearchName(book) : undefined,
      teaches: entry.title,
      isPartOf: book ? { "@type": "Book", name: book.titleKo ?? book.titleVi } : undefined,
    },
  ];

  return (
    <article className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <JsonLd data={ld} />
      <BackLink href="/ngu-phap">Ngữ pháp tiếng Hàn KIIP</BackLink>

      <header className="-mt-2 flex flex-col gap-2">
        {book && (
          <p className="text-sm text-muted-foreground">
            {bookSearchName(book)}
            {lesson && ` · Bài ${lesson}`} · {book.titleVi}
          </p>
        )}
        <h1 className="font-korean text-3xl font-semibold tracking-tight">{entry.title}</h1>
        {entry.ko && <p className="font-korean text-muted-foreground">{entry.ko}</p>}
      </header>

      <section className="flex flex-col gap-2">
        <h2 className="text-base font-semibold">Nghĩa và cách dùng</h2>
        <p className="leading-relaxed">{entry.vi}</p>
      </section>

      <section className="flex flex-col gap-2">
        <h2 className="text-base font-semibold">Ví dụ trong sách</h2>
        <blockquote className="border-l-2 border-primary/50 pl-4">
          <p className="font-korean whitespace-pre-line">{entry.exKo}</p>
          <p className="mt-1 whitespace-pre-line text-muted-foreground">{entry.exVi}</p>
        </blockquote>
      </section>

      <div className="flex flex-wrap gap-2">
        <Link
          href={`/read/${entry.bookId}/${entry.page}`}
          className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          <BookOpen className="size-4" aria-hidden />
          Xem trang {entry.page} trong sách
        </Link>
        {lesson && (
          <Link
            href={bookLessonHref(entry.bookId, lesson)}
            className="inline-flex items-center rounded-lg border border-border px-4 py-2 text-sm font-medium hover:bg-muted"
          >
            Dịch toàn bộ bài {lesson}
          </Link>
        )}
      </div>

      <nav aria-label="Ngữ pháp trước / sau" className="mt-4 grid grid-cols-2 gap-3 border-t border-border pt-4">
        {prev ? (
          <Link href={grammarHref(prev.slug)} className="flex flex-col gap-0.5 rounded-lg p-2 hover:bg-muted">
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <ChevronLeft className="size-3.5" aria-hidden /> Trước
            </span>
            <span className="font-korean font-medium">{prev.title}</span>
          </Link>
        ) : (
          <span />
        )}
        {next && (
          <Link
            href={grammarHref(next.slug)}
            className="flex flex-col items-end gap-0.5 rounded-lg p-2 text-right hover:bg-muted"
          >
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              Tiếp <ChevronRight className="size-3.5" aria-hidden />
            </span>
            <span className="font-korean font-medium">{next.title}</span>
          </Link>
        )}
      </nav>
    </article>
  );
}
