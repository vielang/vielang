import { groupArticles, type GuideArticle, type GuideSectionId } from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";

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
            <h2 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
              {group.title}
              <span className="ml-1.5 font-normal normal-case tabular-nums">· {group.articles.length}</span>
            </h2>
          )}
          <GuideArticleList sectionId={sectionId} articles={group.articles} />
        </section>
      ))}
    </div>
  );
}
