import { groupArticles, type GuideArticle, type GuideSectionId } from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";
import { SectionLabel } from "@/components/layout/page-header";

/**
 * Danh sách bài chia theo nhóm (Visa theo mục đích, Việc làm theo giai
 * đoạn — xem lib/guide-groups.ts). Tên nhóm là dòng chữ nhỏ, không phải
 * khung: nhóm chỉ để mắt lướt tìm đúng chỗ, nội dung vẫn là các dòng bài.
 */
export function GroupedArticleList({
  sectionId,
  articles,
}: {
  sectionId: GuideSectionId;
  articles: GuideArticle[];
}) {
  return (
    <div className="flex flex-col gap-5">
      {groupArticles(sectionId, articles).map((group) => (
        <section key={group.title ?? "all"} className="flex flex-col gap-1">
          {group.title && (
            <SectionLabel aside={<span className="tabular-nums">{group.articles.length}</span>}>
              {group.title}
            </SectionLabel>
          )}
          <GuideArticleList sectionId={sectionId} articles={group.articles} />
        </section>
      ))}
    </div>
  );
}
