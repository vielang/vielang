import type { MetadataRoute } from "next";
import { SITE_URL } from "@/lib/seo";

/**
 * Chặn những chỗ chỉ có nghĩa với từng người hoặc không phải trang:
 * - `/my`: Góc học tập — dữ liệu nằm trong trình duyệt, Google chỉ thấy khung rỗng.
 * - `/offline`: trang dự phòng của service worker.
 *
 * Trang nào cũng đồng thời gắn `noindex` (xem `pageMetadata`), vì robots.txt
 * chỉ cấm bò vào, không cấm hiện URL trong kết quả nếu nơi khác có link tới.
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
      disallow: ["/my", "/offline"],
    },
    sitemap: `${SITE_URL}/sitemap.xml`,
    host: SITE_URL,
  };
}
