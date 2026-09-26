/**
 * Danh sách ngôn ngữ/giáo trình mà site phục vụ — nguồn duy nhất cho cả nav
 * header lẫn route thư viện theo ngôn ngữ.
 *
 * Mỗi ngôn ngữ sống ở `/<slug>` — xem `(main)/[lang]/page.tsx`. Trang gốc
 * `/` là mảng IT (xem lib/tracks.ts), không phải một ngôn ngữ. Thêm ngôn ngữ mới: thêm 1 entry ở đây, rồi thêm
 * sách vào BOOKS (lib/books.ts) với `lang` khớp `code` — không cần đụng route
 * hay component nào khác.
 */
export interface LanguageConfig {
  /** Khớp với `Book.lang` để lọc sách theo ngôn ngữ */
  code: string;
  /** Đoạn route: "" = trang gốc `/`, khác thì `/<slug>` */
  slug: string;
  /** Nhãn hiển thị trên nav header */
  label: string;
  /** Tiêu đề trang thư viện của ngôn ngữ này */
  heading: string;
  description: string;
  /** Một dòng ngắn trong menu thả xuống của header — các bộ sách đang có. */
  blurb: string;
}

export const LANGUAGES = [
  {
    code: "en",
    slug: "en",
    label: "Tiếng Anh",
    heading: "Thư viện tiếng Anh",
    blurb: "Giáo trình English File",
    description:
      "Giáo trình học tiếng Anh English File theo từng cấp độ, có audio nghe theo trang, ghi chú và đánh dấu ngay trên sách.",
  },
] as const satisfies readonly LanguageConfig[];

/** Union tự suy ra từ LANGUAGES — thêm ngôn ngữ mới ở trên là type này rộng ra theo, khỏi sửa 2 chỗ. */
export type LanguageCode = (typeof LANGUAGES)[number]["code"];

export function getLanguage(slug: string): LanguageConfig | undefined {
  return LANGUAGES.find((l) => l.slug === slug);
}
