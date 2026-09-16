import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { BOOKS } from "@/lib/books";
import { LANGUAGES, getLanguage } from "@/lib/languages";
import { LibraryView } from "@/components/library/library-view";

// Tiếng Hàn sống ở route gốc "/" (xem (main)/page.tsx) nên không có slug ""
// ở đây — route này chỉ phục vụ các ngôn ngữ khác trong LANGUAGES.
export function generateStaticParams() {
  return LANGUAGES.filter((l) => l.slug).map((l) => ({ lang: l.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ lang: string }>;
}): Promise<Metadata> {
  const { lang } = await params;
  const language = getLanguage(lang);
  return { title: language ? language.heading : "Không tìm thấy trang" };
}

export default async function LanguageLibraryPage({
  params,
}: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await params;
  const language = getLanguage(lang);
  if (!language) notFound();

  const books = BOOKS.filter((b) => b.lang === language.code);
  return <LibraryView language={language} books={books} />;
}
