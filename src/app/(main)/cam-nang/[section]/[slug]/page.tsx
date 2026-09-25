import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { GUIDE_SECTIONS, getArticle, getSection, sectionArticles } from "@/lib/guide";
import { GuideArticleView } from "@/components/guide/guide-article-view";

export function generateStaticParams() {
  return GUIDE_SECTIONS.flatMap((s) =>
    sectionArticles(s.id).map((a) => ({ section: s.id, slug: a.slug }))
  );
}

// Bài cẩm nang chỉ thêm qua code + deploy lại — slug lạ luôn là 404.
export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ section: string; slug: string }>;
}): Promise<Metadata> {
  const { section, slug } = await params;
  const article = getArticle(section, slug);
  return article
    ? { title: article.title, description: article.summary || undefined }
    : { title: "Không tìm thấy bài" };
}

export default async function GuideArticlePage({
  params,
}: {
  params: Promise<{ section: string; slug: string }>;
}) {
  const { section, slug } = await params;
  const foundSection = getSection(section);
  const article = getArticle(section, slug);
  if (!foundSection || !article) notFound();

  return <GuideArticleView section={foundSection} article={article} />;
}
