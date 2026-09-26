import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { BOOKS } from "@/lib/books";
import { LANGUAGES } from "@/lib/languages";
import { SERIES, getSeries } from "@/lib/series";
import { LibraryView } from "@/components/library/library-view";

// Trang một bộ sách (KIIP, Sejong…). Mã bộ duy nhất trên mọi ngôn ngữ nên
// URL phẳng `/sach/<id>`; ngôn ngữ suy từ bộ — xem lib/series.ts.
export function generateStaticParams() {
  return SERIES.map((s) => ({ series: s.id }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ series: string }>;
}): Promise<Metadata> {
  const series = getSeries((await params).series);
  if (!series) return { title: "Không tìm thấy trang" };
  return pageMetadata({
    title: series.id === "kiip" ? "Giáo trình KIIP (사회통합프로그램) – đọc online, dịch tiếng Việt" : series.label,
    description: series.blurb,
    path: `/sach/${series.id}`,
  });
}

export default async function SeriesPage({ params }: { params: Promise<{ series: string }> }) {
  const series = getSeries((await params).series);
  const language = LANGUAGES.find((l) => l.code === series?.lang);
  if (!series || !language) notFound();

  return <LibraryView language={language} series={series} books={BOOKS.filter((b) => b.series === series.id)} />;
}
