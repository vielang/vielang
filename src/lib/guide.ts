/**
 * Cẩm nang (`/cam-nang`) — thông tin thực tế cho người Việt ở Hàn: visa,
 * trường đại học, việc làm.
 *
 * Khác sách và khoá học: đây KHÔNG phải bài để học mà là thông tin để làm
 * theo (nộp hồ sơ, chọn trường, đi làm), nên sai là người đọc mất tiền, mất
 * visa. Mọi bài bắt buộc có ngày cập nhật và mục Nguồn — bước build từ chối
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
  /** Các khoá khai báo riêng của từng mục (mã visa, thông tin trường…). */
  facts: Record<string, string>;
  html: string;
  headings: LessonHeading[];
  /** Số dòng checklist tick được trong bài (hồ sơ visa). */
  checks: number;
}

export type GuideSectionId = "visa" | "truong" | "viec-lam";

export interface GuideSection {
  id: GuideSectionId;
  title: string;
  /** Một câu dưới tiêu đề mục. */
  description: string;
}

/** Thứ tự hiển thị các mục. Thêm mục = thêm thư mục + một dòng ở đây và ở build-content. */
export const GUIDE_SECTIONS: readonly GuideSection[] = [
  {
    id: "visa",
    title: "Visa",
    description: "Điều kiện, hồ sơ và các bước cho từng loại visa.",
  },
  {
    id: "truong",
    title: "Trường đại học",
    description: "Tuyển sinh, học phí, học bổng và trường tiếng của từng trường.",
  },
  {
    id: "viec-lam",
    title: "Việc làm",
    description: "Đi làm hợp pháp, quyền lợi người lao động và tìm việc an toàn.",
  },
];

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

/** Nhãn hiển thị cho các khoá thông tin trường (xem FORMAT.md). */
export const SCHOOL_FACT_LABELS: Record<string, string> = {
  nameKo: "Tên tiếng Hàn",
  city: "Thành phố",
  kind: "Loại trường",
  certified: "Chứng nhận quốc tế hoá",
  topik: "TOPIK tối thiểu",
  tuition: "Học phí",
  languageSchool: "Trường tiếng (어학당)",
};

/**
 * TOPIK ghi bằng số thì đổi ra chữ ("0" là không yêu cầu). Có ngoại lệ theo
 * ngành thì bài ghi thẳng bằng chữ — một con số trần như "2" mà thật ra chỉ
 * đúng với ngành nghệ thuật là dễ làm người ta chọn nhầm trường.
 */
export function formatSchoolFact(key: string, value: string): string {
  if (key === "topik" && /^\d$/.test(value)) {
    return value === "0" ? "Không yêu cầu" : `Cấp ${value}`;
  }
  return value;
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

/**
 * Lộ trình visa thường gặp, vẽ ở đầu tab Visa. Mỗi bước trỏ tới bài nói về
 * visa đó (có thể ở mục khác, vd E-9 nằm ở Việc làm).
 *
 * CHỈ ghi những lối chuyển mà chính các bài đã nêu kèm nguồn — sơ đồ là lời
 * khẳng định "đi được đường này", vẽ thêm một mũi tên không có trong bài là
 * dắt người đọc đi sai.
 */
export interface VisaStep {
  code: string;
  sectionId: GuideSectionId;
  slug: string;
}

export const VISA_PATHS: readonly { title: string; steps: readonly VisaStep[] }[] = [
  {
    title: "Du học",
    steps: [
      { code: "D-4", sectionId: "visa", slug: "d-4-hoc-tieng" },
      { code: "D-2", sectionId: "visa", slug: "d-2-du-hoc" },
      { code: "D-10", sectionId: "visa", slug: "d-10-tim-viec" },
      { code: "F-2-7", sectionId: "visa", slug: "f-2-7-cu-tru-theo-diem" },
      { code: "F-5", sectionId: "visa", slug: "f-5-vinh-tru" },
    ],
  },
  {
    title: "Lao động",
    steps: [
      { code: "E-9", sectionId: "viec-lam", slug: "lao-dong-eps-e-9" },
      { code: "E-7-4", sectionId: "visa", slug: "e-7-4-lao-dong-lanh-nghe" },
    ],
  },
  {
    title: "Kết hôn",
    steps: [
      { code: "F-6", sectionId: "visa", slug: "f-6-ket-hon" },
      { code: "F-5", sectionId: "visa", slug: "f-5-vinh-tru" },
    ],
  },
];

/** Vùng của trường, suy từ `city` — dùng cho bộ lọc. */
export type SchoolRegion = "Seoul" | "Gyeonggi – Incheon" | "Tỉnh khác";

export function schoolRegion(city: string | undefined): SchoolRegion {
  if (!city) return "Tỉnh khác";
  if (/^Seoul\b/.test(city)) return "Seoul";
  if (/Gyeonggi|Incheon/.test(city)) return "Gyeonggi – Incheon";
  return "Tỉnh khác";
}

/**
 * Mức TOPIK tối thiểu CHUNG của trường, dạng số để lọc: "Cấp 3 (…)" -> 3,
 * "0" / "Không bắt buộc…" -> 0. Không ghi thì null — chưa rõ, không loại
 * trường đó ra khỏi kết quả lọc nhưng cũng không khẳng định là đạt.
 */
export function schoolTopik(topik: string | undefined): number | null {
  if (topik === undefined) return null;
  if (/^\d$/.test(topik)) return Number(topik);
  if (/^Không/i.test(topik)) return 0;
  const m = /^Cấp (\d)/.exec(topik);
  return m ? Number(m[1]) : null;
}

/** Học phí thấp nhất (triệu won/học kỳ) từ "3,3–5,1 triệu won/học kỳ", để sắp xếp. */
export function schoolTuitionFrom(tuition: string | undefined): number | null {
  const m = tuition ? /^(\d+(?:,\d+)?)/.exec(tuition) : null;
  return m ? Number(m[1].replace(",", ".")) : null;
}

/** "2026-09-25" -> "25/09/2026" */
export function formatUpdated(date: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : date;
}
