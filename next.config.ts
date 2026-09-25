import type { NextConfig } from "next";

/**
 * Next.js load .env.local trước khi evaluate file này, nên đọc trực tiếp
 * NEXT_PUBLIC_IMAGE_BASE_URL để suy ra remotePattern cho next/image.
 * Nếu chưa cấu hình (vd lúc mới scaffold, chưa tạo R2 bucket), bỏ qua thay
 * vì throw để `next dev`/`next build` vẫn chạy được.
 */
function imageRemotePatterns(): NonNullable<
  NextConfig["images"]
>["remotePatterns"] {
  const base = process.env.NEXT_PUBLIC_IMAGE_BASE_URL;
  if (!base) {
    console.warn(
      "⚠ NEXT_PUBLIC_IMAGE_BASE_URL chưa được cấu hình — ảnh sách sẽ không tải được. " +
        "Xem web/.env.local.example."
    );
    return [];
  }
  const url = new URL(base);
  return [
    {
      protocol: url.protocol.replace(":", "") as "http" | "https",
      hostname: url.hostname,
      pathname: "/books/**",
    },
  ];
}

/**
 * Ảnh sách đi qua chính origin của app rồi mới tới R2, KHÔNG qua
 * `/_next/image`.
 *
 * Ba lý do, theo thứ tự quan trọng:
 *
 * 1. Ảnh nguồn đã tối ưu sẵn rồi — `prepare-images.ts` xuất webp quality 85,
 *    một trang chỉ 96–155KB. Đẩy qua bộ tối ưu của Vercel là mã hoá lại một
 *    file đã mất chất lượng lên quality 90: không lấy lại được gì, chỉ tốn
 *    thêm byte và một lượt transformation.
 * 2. Hết một điểm chết. 2.699 trang ảnh đều chui qua `/_next/image`; cạn hạn
 *    mức transformation là ảnh của CẢ web vỡ cùng lúc.
 * 3. Vẫn CÙNG ORIGIN nên service worker giữ nguyên tác dụng. Trỏ thẳng sang
 *    `pub-*.r2.dev` thì hỏng: `sw.js` chỉ chặn request cùng origin, mà R2 lại
 *    không gửi header CORS nào (đã thử `curl`), nên bản trả về sẽ là `opaque`
 *    — `response.ok` luôn false, cache không lưu được gì, và tính năng tải
 *    sách về đọc offline chết theo.
 *
 * Đánh đổi: băng thông chạy qua Vercel thay vì dùng egress miễn phí của R2.
 * Chấp nhận được vì mỗi trang chỉ hơn trăm KB và service worker cache rất
 * chặt; đổi lại không phải đụng gì tới cấu hình CORS bên Cloudflare.
 */
function imageRewrites() {
  const base = process.env.NEXT_PUBLIC_IMAGE_BASE_URL;
  if (!base) return [];
  return [
    {
      source: "/img/books/:path*",
      destination: `${base.replace(/\/$/, "")}/books/:path*`,
    },
    // Ảnh trang đề thi — cùng cách với ảnh sách. Lúc dev, file có sẵn trong
    // public/img/exams/ được phục vụ trước rewrite (xem lib/exams.ts).
    {
      source: "/img/exams/:path*",
      destination: `${base.replace(/\/$/, "")}/exams/:path*`,
    },
  ];
}

const nextConfig: NextConfig = {
  rewrites: async () => imageRewrites(),
  // Cẩm nang không có trang tổng: mở thẳng mục đầu tiên. Tạm thời (307) chứ
  // không vĩnh viễn — sau này có thể thêm trang tổng mà trình duyệt không
  // nhớ mãi đường chuyển cũ.
  redirects: async () => [{ source: "/cam-nang", destination: "/cam-nang/visa", permanent: false }],
  images: {
    // Còn lại cho ảnh từ xa nếu về sau cần; ảnh sách nay đi qua `imageRewrites`.
    remotePatterns: imageRemotePatterns(),
    // 75 = mặc định (thumbnail, ảnh bìa); 90 = trang đọc full-screen, cần nét
    // hơn để zoom đọc chữ Hàn nhỏ trong ảnh scan.
    qualities: [75, 90],
  },
};

export default nextConfig;
