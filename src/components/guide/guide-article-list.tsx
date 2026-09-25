import Link from "next/link";
import { ChevronRight, ListChecks } from "lucide-react";
import {
  articleHref,
  formatSchoolFact,
  type GuideArticle,
  type GuideSectionId,
} from "@/lib/guide";

/**
 * Dòng phụ dưới tên bài: với trường thì là mấy thông số người ta lọc trong
 * đầu khi chọn trường (ở đâu, cần TOPIK mấy, có chứng nhận không); bài khác
 * thì là tóm tắt.
 */
function Meta({ sectionId, article }: { sectionId: GuideSectionId; article: GuideArticle }) {
  if (sectionId !== "truong") {
    return <span className="line-clamp-2 text-sm text-muted-foreground">{article.summary}</span>;
  }
  const { city, topik, certified } = article.facts;
  const parts = [
    city,
    topik !== undefined ? `TOPIK: ${formatSchoolFact("topik", topik)}` : null,
    certified === "Có" ? "Đạt chứng nhận quốc tế hoá" : null,
  ].filter(Boolean);
  return <span className="text-sm text-muted-foreground">{parts.join(" · ")}</span>;
}

export function GuideArticleList({
  sectionId,
  articles,
}: {
  sectionId: GuideSectionId;
  articles: GuideArticle[];
}) {
  return (
    <ul className="flex flex-col divide-y divide-border rounded-xl border border-border">
      {articles.map((article) => (
        <li key={article.slug}>
          <Link
            href={articleHref(sectionId, article.slug)}
            className="group flex items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/60"
          >
            {article.facts.code && (
              <span className="w-14 shrink-0 rounded-md bg-primary/10 py-1 text-center font-mono text-xs font-semibold text-primary">
                {article.facts.code}
              </span>
            )}
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className="font-medium">{article.title}</span>
              <Meta sectionId={sectionId} article={article} />
            </span>
            {article.checks > 0 && (
              <span
                className="hidden shrink-0 items-center gap-1 text-xs text-muted-foreground sm:inline-flex"
                title="Có checklist hồ sơ tick được"
              >
                <ListChecks className="size-3.5" aria-hidden />
                {article.checks}
              </span>
            )}
            <ChevronRight
              className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
              aria-hidden
            />
          </Link>
        </li>
      ))}
    </ul>
  );
}
