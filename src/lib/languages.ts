/**
 * Danh sách ngôn ngữ/giáo trình mà site phục vụ — nguồn duy nhất cho cả nav
 * header lẫn route thư viện theo ngôn ngữ.
 *
 * `slug: ""` là ngôn ngữ mặc định, sống ở route gốc `/` (đang là tiếng Hàn,
 * để không phá các link cũ). Ngôn ngữ khác sống ở `/<slug>` — xem
 * `(main)/[lang]/page.tsx`. Thêm ngôn ngữ mới: thêm 1 entry ở đây, rồi thêm
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
    code: "ko",
    slug: "",
    label: "Tiếng Hàn",
    heading: "Thư viện tiếng Hàn",
    blurb: "Giáo trình KIIP và sách bài tập",
    description:
      "Sách văn hóa – xã hội Hàn Quốc (chương trình KIIP), dành cho người Việt học tiếng Hàn. Mỗi cấp độ gồm giáo trình chính và sách bài tập đi kèm.",
  },
  {
    code: "en",
    slug: "en",
    label: "Tiếng Anh",
    heading: "Thư viện tiếng Anh",
    blurb: "Giáo trình English File",
    description:
      "Giáo trình học tiếng Anh, tổ chức theo cấp độ tương tự chương trình KIIP tiếng Hàn — giáo trình chính đi kèm sách bài tập ở mỗi cấp.",
  },
] as const satisfies readonly LanguageConfig[];

/** Union tự suy ra từ LANGUAGES — thêm ngôn ngữ mới ở trên là type này rộng ra theo, khỏi sửa 2 chỗ. */
export type LanguageCode = (typeof LANGUAGES)[number]["code"];

export function getLanguage(slug: string): LanguageConfig | undefined {
  return LANGUAGES.find((l) => l.slug === slug);
}
