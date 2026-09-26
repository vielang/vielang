import { ListChecks } from "lucide-react";
import { RowLink, RowList } from "@/components/layout/row-list";
import { articleHref, type GuideArticle, type GuideSectionId } from "@/lib/guide";

/**
 * Danh sách bài của một mục cẩm nang.
 *
 * KHÔNG có khung viền bao ngoài: mỗi bài là một dòng, trên điện thoại tràn
 * hết bề ngang, ngăn nhau bằng đường kẻ mảnh — nhìn như danh sách của ứng
 * dụng chứ không như một cái bảng, và dành thêm chỗ cho chữ.
 */
export function GuideArticleList({
  sectionId,
  articles,
}: {
  sectionId: GuideSectionId;
  articles: GuideArticle[];
}) {
  return (
    <RowList>
      {articles.map((article) => (
        <RowLink key={article.slug} href={articleHref(sectionId, article.slug)}>
            {article.facts.code && (
              <span className="flex h-7 min-w-14 shrink-0 items-center justify-center self-start rounded-md bg-primary/10 px-1.5 text-xs font-semibold text-primary">
                {article.facts.code}
              </span>
            )}
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className="font-medium leading-snug">{article.title}</span>
              <span className="line-clamp-2 text-sm text-muted-foreground">{article.summary}</span>
            </span>
            {article.checks > 0 && (
              <span
                className="hidden shrink-0 items-center gap-1 text-xs text-muted-foreground sm:inline-flex"
                title="Có checklist tick được"
              >
                <ListChecks className="size-3.5" aria-hidden />
                {article.checks}
              </span>
            )}
        </RowLink>
      ))}
    </RowList>
  );
}
