import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { GUIDE_SECTIONS, getSection, sectionArticles } from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";
import { GuideTabs } from "@/components/guide/guide-tabs";

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

/**
 * Một mục của cẩm nang. `/cam-nang` không có trang riêng mà chuyển thẳng tới
 * mục đầu tiên (xem `redirects` trong next.config.ts) — như Thư viện mở sẵn
 * mảng tiếng Hàn.
 */
export default async function GuideSectionPage({ params }: { params: Promise<{ section: string }> }) {
  const found = getSection((await params).section);
  if (!found) notFound();
  const articles = sectionArticles(found.id);

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Cẩm nang sống ở Hàn</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Tổng hợp từ nguồn chính thức, mỗi bài ghi rõ ngày cập nhật và nguồn để bạn đối chiếu.
        </p>
      </div>

      <GuideTabs active={found.id} />

      <p className="-mt-2 text-sm text-muted-foreground">{found.description}</p>

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
