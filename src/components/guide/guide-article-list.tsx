import Link from "next/link";
import { ChevronRight, ListChecks } from "lucide-react";
import {
  articleHref,
  formatSchoolFact,
  type GuideArticle,
  type GuideSectionId,
} from "@/lib/guide";

/**
 * Phần đầu của một thông số, bỏ phần giải thích trong ngoặc: "Cấp 3 (ngành
 * thể thao: cấp 2)" -> "Cấp 3". Dòng trong danh sách chỉ đủ chỗ cho con số
 * chính; phần ngoại lệ đã có đầy đủ trong bài.
 */
function brief(value: string): string {
  return value.replace(/\s*\(.*$/, "");
}

/**
 * Dòng phụ dưới tên bài: với trường là mấy thông số người ta so sánh khi
 * chọn trường (ở đâu, cần TOPIK mấy, học phí bao nhiêu); bài khác là tóm tắt.
 *
 * Không nhắc "đạt chứng nhận quốc tế hoá" khi trường có — hiện gần như trường
 * nào trong danh sách cũng có, nhắc lại chỉ thêm chữ. Trường KHÔNG có mới
 * đáng nói, vì xin visa khó hơn hẳn.
 */
function Meta({ sectionId, article }: { sectionId: GuideSectionId; article: GuideArticle }) {
  if (sectionId !== "truong") {
    return <span className="line-clamp-2 text-sm text-muted-foreground">{article.summary}</span>;
  }
  const { city, topik, tuition, certified } = article.facts;
  const parts = [
    city ? brief(city) : null,
    topik !== undefined ? `TOPIK ${brief(formatSchoolFact("topik", topik)).toLowerCase()}` : null,
    tuition ? brief(tuition) : null,
  ].filter(Boolean);
  return (
    <span className="text-sm text-muted-foreground">
      {parts.join(" · ")}
      {certified === "Không" && (
        <span className="text-amber-600 dark:text-amber-400"> · Chưa có chứng nhận quốc tế hoá</span>
      )}
    </span>
  );
}

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
    <ul className="-mx-4 flex flex-col divide-y divide-border/60 sm:mx-0">
      {articles.map((article) => (
        <li key={article.slug}>
          <Link
            href={articleHref(sectionId, article.slug)}
            className="group flex items-center gap-3 px-4 py-3.5 transition-colors hover:bg-muted/50 active:bg-muted sm:-mx-3 sm:rounded-lg sm:px-3"
          >
            {article.facts.code && (
              <span className="flex h-7 min-w-14 shrink-0 items-center justify-center self-start rounded-md bg-primary/10 px-1.5 text-xs font-semibold text-primary">
                {article.facts.code}
              </span>
            )}
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className="font-medium leading-snug">{article.title}</span>
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
              className="size-4 shrink-0 text-muted-foreground/60 transition-transform group-hover:translate-x-0.5"
              aria-hidden
            />
          </Link>
        </li>
      ))}
    </ul>
  );
}
