import type { MetadataRoute } from "next";

/**
 * Manifest để cài app vào màn hình chính.
 *
 * `display: standalone` bỏ thanh địa chỉ — thêm thanh địa chỉ chỉ tổ ăn mất
 * chiều cao đọc bài.
 */
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "VieLang – Học lập trình .NET",
    short_name: "VieLang",
    description:
      "Học lập trình bằng tiếng Việt: lộ trình .NET developer (C#, OOP, SQL, ASP.NET Core, WinForms, cấu trúc dữ liệu, kiến trúc phần mềm) và cẩm nang nghề IT.",
    start_url: "/",
    display: "standalone",
    background_color: "#0a0a0a",
    theme_color: "#0a0a0a",
    lang: "vi",
    icons: [
      { src: "/icons/icon-192.png", sizes: "192x192", type: "image/png" },
      { src: "/icons/icon-512.png", sizes: "512x512", type: "image/png" },
      {
        src: "/icons/icon-512.png",
        sizes: "512x512",
        type: "image/png",
        // Android cắt icon theo hình của nó (tròn, squircle…) — hình đã chừa
        // sẵn lề 20% nên cắt kiểu gì cũng không phạm vào nét.
        purpose: "maskable",
      },
    ],
  };
}
