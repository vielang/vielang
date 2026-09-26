import { LANGUAGES, type LanguageConfig } from "@/lib/languages";
import { getBook } from "@/lib/books";
import { seriesOfPath } from "@/lib/series";

/**
 * Các MẢNG KIẾN THỨC của thư viện — nguồn duy nhất cho tab con của Thư viện
 * (hàng tab trong trang, header điện thoại, menu thả xuống trên máy tính).
 *
 * Trước đây thư viện chỉ chia theo NGÔN NGỮ và mọi
 * "sách" đều là ảnh trang scan. Mảng IT không phải một ngôn ngữ và nội dung
 * là CHỮ + code, nên chia theo mảng kiến thức và ghi rõ `kind` — giao diện
 * nhìn `kind` để biết nên vẽ kệ sách ảnh hay danh sách khoá học.
 *
 * Thêm mảng mới (vd DevOps) = thêm một entry ở đây; tab, điều hướng và
 * trang thư viện tự có theo.
 *
 * IT là mảng mặc định, sống ở trang gốc `/` (danh sách khoá học). Trang một
 * khoá vẫn là `/it/<khoá>` nên `/it` nằm trong `paths`.
 */
export interface Track {
  /** Đoạn route: "" = trang gốc `/`, khác thì `/<slug>` */
  slug: string;
  /** Nhãn trên tab */
  label: string;
  /** Một dòng ngắn dưới nhãn trong menu thả xuống của header. */
  blurb: string;
  /** books = kệ sách ảnh (giáo trình scan), courses = khoá học dạng chữ */
  kind: "books" | "courses";
  /** Mã ngôn ngữ của sách thuộc mảng này (mảng sách). */
  lang?: string;
  /** Các nhánh đường dẫn khác cũng thuộc mảng này (vd trang học bài). */
  paths?: string[];
}

export const TRACKS: readonly Track[] = [
  { slug: "", label: "IT", blurb: "Lộ trình .NET developer", kind: "courses", paths: ["/it", "/learn"] },
  ...LANGUAGES.map((l: LanguageConfig) => ({
    slug: l.slug,
    label: l.label,
    blurb: l.blurb,
    kind: "books" as const,
    lang: l.code,
  })),
];

export function trackHref(slug: string): string {
  return slug ? `/${slug}` : "/";
}

/**
 * Mảng kiến thức của đường dẫn hiện tại (để tô tab đang mở), kể cả trang con:
 * trang chi tiết sách `/books/<id>` thuộc mảng theo ngôn ngữ của cuốn đó —
 * đang xem sách tiếng Anh thì header phải tô "Tiếng Anh", không phải mặc
 * định "IT".
 */
export function activeTrack(pathname: string): Track | undefined {
  const book = /^\/books\/([^/]+)/.exec(pathname);
  if (book || pathname.startsWith("/sach/")) {
    const lang = book ? getBook(book[1])?.lang : seriesOfPath(pathname)?.lang;
    const byLang = lang ? TRACKS.find((t) => t.lang === lang) : undefined;
    if (byLang) return byLang;
  }
  const match = TRACKS.filter(
    (t) =>
      (t.slug && (pathname === `/${t.slug}` || pathname.startsWith(`/${t.slug}/`))) ||
      t.paths?.some((base) => pathname === base || pathname.startsWith(`${base}/`))
  );
  // Chọn slug dài nhất: "/en" không được ăn mất "/enx" sau này. Không khớp
  // gì thì là trang gốc — mảng IT.
  return match.sort((a, b) => b.slug.length - a.slug.length)[0] ?? TRACKS.find((t) => !t.slug);
}
