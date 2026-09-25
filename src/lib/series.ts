import type { LanguageCode } from "@/lib/languages";

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
  { id: "kiip", lang: "ko", label: "KIIP", blurb: "Chương trình hội nhập xã hội — giáo trình và sách bài tập" },
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
