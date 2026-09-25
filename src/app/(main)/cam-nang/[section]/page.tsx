import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { GUIDE_SECTIONS, getSection, sectionArticles } from "@/lib/guide";
import { GroupedArticleList } from "@/components/guide/grouped-article-list";
import { GuideTabs } from "@/components/guide/guide-tabs";
import { SchoolExplorer } from "@/components/guide/school-explorer";
import { VisaRoadmap } from "@/components/guide/visa-roadmap";

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
 *
 * Mỗi mục trình bày theo cách người ta tìm trong mục đó: Visa có sơ đồ lộ
 * trình và nhóm theo mục đích, Trường có bộ lọc, Việc làm nhóm theo giai đoạn.
 */
export default async function GuideSectionPage({ params }: { params: Promise<{ section: string }> }) {
  const found = getSection((await params).section);
  if (!found) notFound();
  const articles = sectionArticles(found.id);

  return (
    <div className="flex flex-col gap-3">
      <h1 className="text-xl font-semibold tracking-tight">Cẩm nang sống ở Hàn</h1>

      <GuideTabs active={found.id} />

      <p className="text-sm text-muted-foreground">{found.description}</p>

      {articles.length === 0 ? (
        <p className="py-16 text-center text-sm text-muted-foreground">Mục này đang được biên soạn.</p>
      ) : found.id === "truong" ? (
        <SchoolExplorer articles={articles} />
      ) : (
        <>
          {found.id === "visa" && <VisaRoadmap />}
          <div className="mt-2">
            <GroupedArticleList sectionId={found.id} articles={articles} />
          </div>
        </>
      )}
    </div>
  );
}
