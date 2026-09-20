import type { MetadataRoute } from "next";

/**
 * Manifest để cài app vào màn hình chính.
 *
 * `display: standalone` bỏ thanh địa chỉ — trang đọc vốn đã chiếm trọn màn
 * hình, thêm thanh địa chỉ chỉ tổ ăn mất chiều cao, đúng chiều đang thiếu.
 *
 * `orientation` để tự do: sách đọc dọc là chính, nhưng chế độ 2 trang cần
 * xoay ngang.
 */
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "KIIP Reader – Đọc sách tiếng Hàn",
    short_name: "KIIP Reader",
    description:
      "Đọc sách văn hóa – xã hội Hàn Quốc (chương trình KIIP) dành cho người Việt học tiếng Hàn.",
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
