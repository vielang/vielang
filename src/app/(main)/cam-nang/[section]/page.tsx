import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { ArrowLeft } from "lucide-react";
import { GUIDE_SECTIONS, getSection, sectionArticles } from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";

export function generateStaticParams() {
  return GUIDE_SECTIONS.map((s) => ({ section: s.id }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ section: string }>;
}): Promise<Metadata> {
  const { section } = await params;
  const found = getSection(section);
  return found
    ? { title: `${found.title} — Cẩm nang`, description: found.description }
    : { title: "Không tìm thấy mục" };
}

export default async function GuideSectionPage({ params }: { params: Promise<{ section: string }> }) {
  const found = getSection((await params).section);
  if (!found) notFound();
  const articles = sectionArticles(found.id);

  return (
    <div className="flex flex-col gap-6">
      <Link
        href="/cam-nang"
        className="inline-flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        Cẩm nang
      </Link>
      <div className="-mt-2">
        <h1 className="text-2xl font-semibold tracking-tight">{found.title}</h1>
        <p className="mt-1 text-sm text-muted-foreground">{found.description}</p>
      </div>
      {articles.length > 0 ? (
        <GuideArticleList sectionId={found.id} articles={articles} />
      ) : (
        <p className="rounded-xl border border-dashed border-border py-16 text-center text-sm text-muted-foreground">
          Mục này đang được biên soạn.
        </p>
      )}
    </div>
  );
}
