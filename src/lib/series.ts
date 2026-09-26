import type { LanguageCode, LanguageConfig } from "@/lib/languages";

/**
 * BỘ SÁCH — tầng thứ ba của thư viện: Thư viện → ngôn ngữ → bộ sách (KIIP,
 * Sejong…). Mỗi cuốn trong BOOKS ghi `series` khớp `id` ở đây.
 *
 * Thêm bộ mới = thêm một entry ở đây rồi thêm sách với `series` tương ứng:
 * menu header, hàng chọn bộ sách và trang `/sach/<id>` tự có theo. Ngôn ngữ
 * mới có MỘT bộ thì chưa hiện hàng chọn (một nút thì chọn gì) — link của bộ
 * trỏ thẳng về trang ngôn ngữ.
 *
 * Module nhẹ (không import BOOKS) — header dùng ở mọi trang.
 */
export interface BookSeries {
  /** Đoạn route `/sach/<id>` — duy nhất trên mọi ngôn ngữ. */
  id: string;
  lang: LanguageCode;
  label: string;
  /** Một dòng ngắn: dưới tên bộ trong menu, dòng phụ dưới tiêu đề trang. */
  blurb: string;
}

export const SERIES: readonly BookSeries[] = [
  { id: "english-file", lang: "en", label: "English File", blurb: "Giáo trình English File theo từng cấp độ" },
];

export function getSeries(id: string): BookSeries | undefined {
  return SERIES.find((s) => s.id === id);
}

export function seriesOfLang(lang: string): BookSeries[] {
  return SERIES.filter((s) => s.lang === lang);
}

/** Trang của bộ sách; ngôn ngữ chỉ có một bộ thì chính là trang ngôn ngữ. */
export function seriesHref(series: BookSeries, languageHref: string): string {
  return seriesOfLang(series.lang).length > 1 ? `/sach/${series.id}` : languageHref;
}

/** Bộ sách của đường dẫn `/sach/<id>`. */
export function seriesOfPath(path: string): BookSeries | undefined {
  const m = /^\/sach\/([^/]+)/.exec(path);
  return m ? getSeries(m[1]) : undefined;
}

export interface LibraryTabItem {
  key: string;
  label: string;
  href: string;
  active: boolean;
}

/**
 * Hàng chọn tầng dưới của trang Thư viện — dùng CHUNG bởi trang ngôn ngữ
 * (`/`, `/sach/<id>`) và trang `/video`, để "Học qua video" hiện làm một tab
 * CÙNG CẤP với bộ sách (KIIP…), bấm qua lại giữa hai bên được.
 *
 * `activeKey`: id bộ sách đang xem, hoặc `"video"`.
 */
export function libraryTabs(language: LanguageConfig, activeKey: string): LibraryTabItem[] {
  const languageHref = language.slug ? `/${language.slug}` : "/";
  const allSeries = seriesOfLang(language.code);
  const items: LibraryTabItem[] =
    allSeries.length > 1
      ? [
          { key: "all", label: "Tất cả", href: languageHref, active: activeKey === "all" },
          ...allSeries.map((s) => ({
            key: s.id,
            label: s.label,
            href: seriesHref(s, languageHref),
            active: activeKey === s.id,
          })),
        ]
      : allSeries.map((s) => ({ key: s.id, label: s.label, href: languageHref, active: activeKey === s.id }));
  return items;
}
