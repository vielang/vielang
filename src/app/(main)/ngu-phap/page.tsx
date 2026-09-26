import Link from "next/link";
import type { Metadata } from "next";
import { getBook } from "@/lib/books";
import { bookSearchName, lessonOfPage } from "@/lib/book-lessons";
import { getAllGrammar, grammarHref, type GrammarEntry } from "@/lib/page-grammar";
import { breadcrumbLd, pageMetadata } from "@/lib/seo";
import { PageHeader, SectionLabel } from "@/components/layout/page-header";
import { JsonLd } from "@/components/seo/json-ld";

export const metadata: Metadata = pageMetadata({
  title: "Ngữ pháp tiếng Hàn KIIP – giải thích tiếng Việt, ví dụ",
  description:
    "Toàn bộ ngữ pháp tiếng Hàn trong giáo trình KIIP (사회통합프로그램) từ Sơ cấp đến Trung cấp: nghĩa tiếng Việt, câu ví dụ có dịch, xếp theo từng bài.",
  path: "/ngu-phap",
});

/**
 * Mục lục mọi điểm ngữ pháp, chia theo sách — đúng thứ tự người ta học. Ô
 * tra cứu nhanh đã có ở trang Thư viện; trang này là bản đầy đủ, link thẳng
 * tới từng điểm để cả người lẫn Google đi hết được.
 */
export default function GrammarListPage() {
  const byBook = new Map<string, GrammarEntry[]>();
  for (const entry of getAllGrammar()) {
    byBook.set(entry.bookId, [...(byBook.get(entry.bookId) ?? []), entry]);
  }

  return (
    <div className="flex max-w-3xl flex-col gap-8">
      <JsonLd data={breadcrumbLd([{ name: "Thư viện", path: "/" }, { name: "Ngữ pháp", path: "/ngu-phap" }])} />
      <PageHeader
        title="Ngữ pháp tiếng Hàn KIIP"
        subtitle="Mỗi điểm ngữ pháp trong giáo trình, kèm nghĩa tiếng Việt và câu ví dụ lấy từ chính trang sách."
      />

      {[...byBook].map(([bookId, entries]) => {
        const book = getBook(bookId);
        if (!book) return null;
        return (
          <section key={bookId} className="flex flex-col gap-3">
            <SectionLabel aside={book.titleKo}>{`${bookSearchName(book)} · ${book.titleVi}`}</SectionLabel>
            <ul className="grid gap-2 sm:grid-cols-2">
              {entries.map((entry) => (
                <li key={entry.slug}>
                  <Link
                    href={grammarHref(entry.slug)}
                    className="block h-full rounded-xl border border-border/70 px-4 py-3 transition-colors hover:border-primary/40 hover:bg-muted/40"
                  >
                    <span className="flex items-baseline justify-between gap-3">
                      <span className="font-korean font-semibold tracking-tight">{entry.title}</span>
                      <span className="shrink-0 text-[11px] text-muted-foreground tabular-nums">
                        Bài {lessonOfPage(bookId, entry.page) ?? "–"}
                      </span>
                    </span>
                    <span className="mt-1 line-clamp-2 block text-sm text-muted-foreground">{entry.vi}</span>
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
