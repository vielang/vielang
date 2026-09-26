/**
 * Cẩm nang (`/cam-nang`) — kiến thức nghề cho người học lập trình: lộ trình,
 * phỏng vấn, công cụ, việc làm IT.
 *
 * Khác khoá học: đây KHÔNG phải bài dạy một khái niệm mà là thông tin để làm
 * theo (chuẩn bị phỏng vấn, viết CV, dựng môi trường). Mọi bài bắt buộc có
 * ngày cập nhật và mục Nguồn — bước build từ chối
 * bài thiếu hai thứ đó (xem `buildGuide` trong scripts/build-content.ts).
 *
 * Nguồn: `content/cam-nang/<mục>/<bài>.md`, gộp thành `guide.json` lúc build
 * rồi `import` TĨNH, cùng lý do với courses.json. Chuẩn viết bài:
 * `content/cam-nang/FORMAT.md`.
 */
import data from "../../content/cam-nang/guide.json";
import type { LessonHeading } from "@/lib/courses";
import { GUIDE_GROUPS } from "@/lib/guide-groups";

export interface GuideArticle {
  /** Phần đuôi URL: `/cam-nang/<mục>/<slug>` */
  slug: string;
  title: string;
  summary: string;
  /** Ngày cập nhật nội dung, dạng YYYY-MM-DD. */
  updated: string;
  /** Các khoá khai báo riêng của từng mục (vd `group`). */
  facts: Record<string, string>;
  html: string;
  headings: LessonHeading[];
  /** Số dòng checklist tick được trong bài (vd việc cần làm trước buổi phỏng vấn). */
  checks: number;
}

// Danh sách mục nằm ở module nhẹ riêng (không kèm nội dung bài) để header dùng được.
import { GUIDE_SECTIONS, type GuideSection, type GuideSectionId } from "@/lib/guide-sections";
export { GUIDE_SECTIONS, type GuideSection, type GuideSectionId };

const ARTICLES = data as unknown as Record<GuideSectionId, GuideArticle[]>;

export function getSection(id: string): GuideSection | undefined {
  return GUIDE_SECTIONS.find((s) => s.id === id);
}

export function sectionArticles(id: GuideSectionId): GuideArticle[] {
  return ARTICLES[id] ?? [];
}

export function getArticle(sectionId: string, slug: string): GuideArticle | undefined {
  const section = getSection(sectionId);
  return section ? sectionArticles(section.id).find((a) => a.slug === slug) : undefined;
}

export function articleHref(sectionId: GuideSectionId, slug: string): string {
  return `/cam-nang/${sectionId}/${slug}`;
}

/** Khoá lưu checklist đã tick của một bài — gồm cả mục để slug trùng giữa các mục không đè nhau. */
export function checklistId(sectionId: GuideSectionId, slug: string): string {
  return `${sectionId}/${slug}`;
}

/**
 * Chia bài của một mục theo nhóm (GUIDE_GROUPS), giữ thứ tự nhóm đã định.
 * Mục không chia nhóm thì trả về một nhóm không tên chứa tất cả.
 */
export function groupArticles(
  sectionId: GuideSectionId,
  articles: GuideArticle[]
): { title: string | null; articles: GuideArticle[] }[] {
  const groups = GUIDE_GROUPS[sectionId];
  if (!groups) return [{ title: null, articles }];
  return groups
    .map((title) => ({ title, articles: articles.filter((a) => a.facts.group === title) }))
    .filter((g) => g.articles.length > 0);
}

/** "2026-09-25" -> "25/09/2026" */
export function formatUpdated(date: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : date;
}
