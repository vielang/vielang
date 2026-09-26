import type { Metadata } from "next";
import { pageMetadata } from "@/lib/seo";
import { notFound } from "next/navigation";
import { GUIDE_SECTIONS, getArticle, getSection, sectionArticles } from "@/lib/guide";
import { GuideArticleView } from "@/components/guide/guide-article-view";
import { JsonLd } from "@/components/seo/json-ld";
import { absoluteUrl, breadcrumbLd } from "@/lib/seo";

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
    ? pageMetadata({
        title: article.title,
        description: article.summary || undefined,
        path: `/cam-nang/${section}/${slug}`,
        type: "article",
      })
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

  const path = `/cam-nang/${section}/${slug}`;
  return (
    <>
      <JsonLd
        data={[
          breadcrumbLd([
            { name: `Cẩm nang ${foundSection.title}`, path: `/cam-nang/${section}` },
            { name: article.title, path },
          ]),
          {
            "@context": "https://schema.org",
            "@type": "Article",
            headline: article.title,
            description: article.summary || undefined,
            dateModified: article.updated,
            inLanguage: "vi",
            mainEntityOfPage: absoluteUrl(path),
            image: absoluteUrl("/og.png"),
            author: { "@type": "Organization", name: "VieTopik", url: absoluteUrl("/") },
          },
        ]}
      />
      <GuideArticleView section={foundSection} article={article} />
    </>
  );
}
