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

const nextConfig: NextConfig = {
  images: {
    remotePatterns: imageRemotePatterns(),
    // 75 = mặc định (thumbnail, ảnh bìa); 90 = trang đọc full-screen, cần nét
    // hơn để zoom đọc chữ Hàn nhỏ trong ảnh scan.
    qualities: [75, 90],
  },
};

export default nextConfig;
