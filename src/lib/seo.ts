import type { Metadata } from "next";

/**
 * Mọi thứ máy tìm kiếm và mạng xã hội đọc về site.
 *
 * Tên miền thật lấy từ env để bản preview của Vercel không tự nhận mình là
 * trang chính (canonical, sitemap đều trỏ về đây). Chưa khai thì dùng tên
 * miền production Vercel tự cấp (`VERCEL_PROJECT_PRODUCTION_URL`, có sẵn
 * lúc build trên Vercel), chạy máy nhà thì localhost.
 */
export const SITE_URL = (
  process.env.NEXT_PUBLIC_SITE_URL ??
  (process.env.VERCEL_PROJECT_PRODUCTION_URL
    ? `https://${process.env.VERCEL_PROJECT_PRODUCTION_URL}`
    : "http://localhost:3000")
).replace(/\/$/, "");

export const SITE_NAME = "VieLang";

export const SITE_DESCRIPTION =
  "Học tiếng Hàn chương trình KIIP (사회통합프로그램) cho người Việt: đọc sách có dịch tiếng Việt, giải thích ngữ pháp, audio, luyện đề TOPIK và cẩm nang visa, việc làm, du học ở Hàn Quốc.";

/** Ảnh chia sẻ mặc định — sinh bằng `npm run make-icons`. */
const OG_IMAGE = { url: "/og.png", width: 1200, height: 630, alt: "VieLang – Học tiếng Hàn KIIP & TOPIK cho người Việt" };

export function absoluteUrl(path: string): string {
  return `${SITE_URL}${path.startsWith("/") ? path : `/${path}`}`;
}

/**
 * Metadata đầy đủ cho một trang: title, description, canonical, Open Graph.
 *
 * Phải gọi ở TỪNG trang chứ không đặt một lần ở layout: Next gộp metadata
 * nông, `openGraph` của trang con thay nguyên cục của cha, còn canonical đặt
 * ở layout thì mọi trang con đều trỏ về trang chủ — Google coi cả site là
 * bản sao của một trang.
 *
 * `noindex` cho trang chỉ có nghĩa với từng người (Góc học tập, kết quả thi):
 * nội dung nằm trong trình duyệt, Google vào chỉ thấy khung rỗng.
 */
export function pageMetadata({
  title,
  description,
  path,
  noindex = false,
  type = "website",
}: {
  title: string;
  description?: string;
  path: string;
  noindex?: boolean;
  type?: "website" | "article";
}): Metadata {
  return {
    title,
    description,
    alternates: { canonical: path },
    openGraph: {
      type,
      siteName: SITE_NAME,
      locale: "vi_VN",
      url: path,
      title,
      description,
      images: [OG_IMAGE],
    },
    twitter: { card: "summary_large_image", title, description, images: [OG_IMAGE.url] },
    ...(noindex && { robots: { index: false, follow: true } }),
  };
}

/**
 * Cắt văn bản thành một đoạn mô tả ~155 ký tự — cỡ Google hiện trọn trong
 * kết quả tìm kiếm. Cắt ở ranh giới từ để khỏi đứt giữa chữ.
 */
export function excerpt(text: string, max = 155): string {
  const flat = text.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
  if (flat.length <= max) return flat;
  const cut = flat.slice(0, max - 1);
  return `${cut.slice(0, cut.lastIndexOf(" ") > max * 0.6 ? cut.lastIndexOf(" ") : cut.length)}…`;
}

/** BreadcrumbList cho JSON-LD — `items` theo thứ tự từ gốc xuống. */
export function breadcrumbLd(items: { name: string; path: string }[]) {
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((item, i) => ({
      "@type": "ListItem",
      position: i + 1,
      name: item.name,
      item: absoluteUrl(item.path),
    })),
  };
}
