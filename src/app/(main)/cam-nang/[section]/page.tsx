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
 *
 * Phần đầu cố ý gọn (tiêu đề nhỏ, một dòng mô tả): trên điện thoại, mỗi dòng
 * chữ ở đây đẩy danh sách — thứ người ta vào để xem — xuống thêm một đoạn.
 */
export default async function GuideSectionPage({ params }: { params: Promise<{ section: string }> }) {
  const found = getSection((await params).section);
  if (!found) notFound();
  const articles = sectionArticles(found.id);

  return (
    <div className="flex flex-col gap-3">
      <h1 className="text-xl font-semibold tracking-tight">Cẩm nang sống ở Hàn</h1>

      <GuideTabs active={found.id} />

      <p className="text-sm text-muted-foreground">
        {found.description} <span className="whitespace-nowrap">{articles.length} bài.</span>
      </p>

      {articles.length > 0 ? (
        <GuideArticleList sectionId={found.id} articles={articles} />
      ) : (
        <p className="py-16 text-center text-sm text-muted-foreground">Mục này đang được biên soạn.</p>
      )}
    </div>
  );
}
