import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { GUIDE_SECTIONS, getSection, sectionArticles } from "@/lib/guide";
import { GroupedArticleList } from "@/components/guide/grouped-article-list";
import { PageHeader } from "@/components/layout/page-header";
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
    // Cùng khung với Thư viện và Luyện thi; chọn mục (Visa, Trường, Việc làm)
    // nằm trên header. Danh sách giới hạn bề ngang như Luyện thi.
    <div className="flex max-w-3xl flex-col gap-6">
      <PageHeader title={`Cẩm nang ${found.title}`} subtitle={found.description} />

      {articles.length === 0 ? (
        <p className="py-16 text-center text-sm text-muted-foreground">Mục này đang được biên soạn.</p>
      ) : found.id === "truong" ? (
        <SchoolExplorer articles={articles} />
      ) : (
        <>
          {found.id === "visa" && <VisaRoadmap />}
          <GroupedArticleList sectionId={found.id} articles={articles} />
        </>
      )}
    </div>
  );
}
