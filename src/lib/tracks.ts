import { LANGUAGES } from "@/lib/languages";

/**
 * Các MẢNG KIẾN THỨC của thư viện — nguồn duy nhất cho hàng tab ở đầu trang
 * Thư viện.
 *
 * Trước đây thư viện chỉ chia theo NGÔN NGỮ (tiếng Hàn, tiếng Anh) và mọi
 * "sách" đều là ảnh trang scan. Mảng IT không phải một ngôn ngữ và nội dung
 * là CHỮ + code, nên chia theo mảng kiến thức và ghi rõ `kind` — giao diện
 * nhìn `kind` để biết nên vẽ kệ sách ảnh hay danh sách khoá học.
 *
 * Thêm mảng mới (vd tiếng Nhật, DevOps) = thêm một entry ở đây; tab, điều
 * hướng và trang thư viện tự có theo.
 */
export interface Track {
  /** Đoạn route: "" = trang gốc `/`, khác thì `/<slug>` */
  slug: string;
  /** Nhãn trên tab */
  label: string;
  /** books = kệ sách ảnh (giáo trình scan), courses = khoá học dạng chữ */
  kind: "books" | "courses";
  /** Các nhánh đường dẫn khác cũng thuộc mảng này (vd trang học bài). */
  paths?: string[];
}

export const TRACKS: readonly Track[] = [
  ...LANGUAGES.map((l) => ({ slug: l.slug, label: l.label, kind: "books" as const })),
  { slug: "it", label: "IT", kind: "courses", paths: ["/learn"] },
];

export function trackHref(slug: string): string {
  return slug ? `/${slug}` : "/";
}

/** Mảng kiến thức của đường dẫn hiện tại (để tô tab đang mở). */
export function activeTrack(pathname: string): Track | undefined {
  const match = TRACKS.filter(
    (t) =>
      (t.slug && (pathname === `/${t.slug}` || pathname.startsWith(`/${t.slug}/`))) ||
      t.paths?.some((base) => pathname === base || pathname.startsWith(`${base}/`))
  );
  // Chọn slug dài nhất: "/it" không được ăn mất "/itx" sau này.
  return match.sort((a, b) => b.slug.length - a.slug.length)[0] ?? TRACKS.find((t) => !t.slug);
}
