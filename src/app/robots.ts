import type { MetadataRoute } from "next";
import { SITE_URL } from "@/lib/seo";

/**
 * Chặn những chỗ chỉ có nghĩa với từng người hoặc không phải trang:
 * - `/my`: Góc học tập — dữ liệu nằm trong trình duyệt, Google chỉ thấy khung rỗng.
 * - `/read/…/note`: cửa sổ bài giảng tách riêng, trùng nội dung trang đọc.
 * - `/exam/…/mock|practice|result`: màn làm bài / chấm điểm, không có gì để đọc.
 * - `/img/`: ảnh trang sách qua rewrite — không cần vào Google Images.
 *
 * Trang nào cũng đồng thời gắn `noindex` (xem `pageMetadata`), vì robots.txt
 * chỉ cấm bò vào, không cấm hiện URL trong kết quả nếu nơi khác có link tới.
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
      disallow: ["/my", "/offline", "/read/*/*/note", "/exam/*/mock", "/exam/*/practice", "/exam/*/result", "/img/"],
    },
    sitemap: `${SITE_URL}/sitemap.xml`,
    host: SITE_URL,
  };
}
