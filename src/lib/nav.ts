import { ChartNoAxesColumn, Compass, GraduationCap, Library, type LucideIcon } from "lucide-react";
import { TRACKS, activeTrack, trackHref } from "@/lib/tracks";
import {
  examLangHref,
  examLanguages,
  examLevelHref,
  examLevelInfo,
  examLevelOfPath,
  levelsOfLang,
} from "@/lib/exam-levels";
import { seriesHref, seriesOfLang } from "@/lib/series";
import { GUIDE_SECTIONS } from "@/lib/guide-sections";

/**
 * Điều hướng chính của app — NGUỒN DUY NHẤT cho thanh tab dưới đáy (điện
 * thoại) và nav trong header (máy tính).
 *
 * Các mục này cố ý đặt theo VIỆC người học làm, không theo nội dung: thêm
 * giáo trình mới, thêm mảng kiến thức mới (lib/tracks) hay thêm kỳ thi mới
 * (IELTS… bên cạnh TOPIK) đều nằm gọn trong "Thư viện" / "Luyện thi", không
 * phải đụng vào thanh điều hướng.
 *
 * Mỗi tab có thể có MỤC CON (mảng kiến thức, cấp đề, mục cẩm nang). Mục con
 * khai báo MỘT lần ở đây, rồi ba nơi cùng đọc: hàng tab trong trang, header
 * điện thoại (thay chỗ logo) và menu thả xuống trên header máy tính.
 */
export interface NavChild {
  href: string;
  label: string;
  /** Một dòng ngắn dưới nhãn trong menu thả xuống. */
  description?: string;
  /**
   * Tầng dưới nữa (bộ sách của một ngôn ngữ, kỳ thi của một ngôn ngữ): menu
   * máy tính xếp thành cột để nhảy thẳng tới. Trên điện thoại tầng này nằm
   * trong trang (hàng chọn ngay dưới tiêu đề), header chỉ hiện tầng trên.
   */
  items?: NavChild[];
}

export interface NavTab {
  href: string;
  label: string;
  icon: LucideIcon;
  /** Đường dẫn này có thuộc tab không (để tô tab đang mở). */
  match: (path: string) => boolean;
  children?: NavChild[];
  /** Mục con nào đang mở ở đường dẫn này (kể cả trang con của nó). */
  activeChild?: (path: string) => string | undefined;
}

const startsWith = (path: string, base: string) => path === base || path.startsWith(`${base}/`);

export const NAV_TABS: NavTab[] = [
  {
    href: "/",
    label: "Thư viện",
    icon: Library,
    match: (p) =>
      p === "/" ||
      startsWith(p, "/books") ||
      startsWith(p, "/sach") ||
      startsWith(p, "/video") ||
      TRACKS.some(
        (t) =>
          (t.slug && startsWith(p, `/${t.slug}`)) ||
          (t.paths?.some((base) => startsWith(p, base)) ?? false)
      ),
    children: TRACKS.map((t) => {
      const href = trackHref(t.slug);
      const series = t.lang ? seriesOfLang(t.lang) : [];
      const items = series.map((s) => ({ href: seriesHref(s, href), label: s.label, description: s.blurb }));
      // "Học qua video" đứng cùng cấp với bộ sách (KIIP…) trong menu.
      if (t.hasVideo) {
        items.push({ href: "/video", label: "Học qua video", description: "Phim có phụ đề Hàn – Việt, bật/tắt được" });
      }
      return { href, label: t.label, description: t.blurb, items };
    }),
    activeChild: (p) => {
      const track = activeTrack(p);
      return track ? trackHref(track.slug) : undefined;
    },
  },
  {
    href: examLangHref(examLanguages()[0]?.code ?? "ko"),
    label: "Luyện thi",
    icon: GraduationCap,
    match: (p) => startsWith(p, "/exam"),
    children: examLanguages().map((lang) => ({
      href: examLangHref(lang.code),
      label: lang.label,
      items: levelsOfLang(lang.code).map((l) => ({ href: examLevelHref(l.id), label: l.id, description: l.hint })),
    })),
    activeChild: (p) => {
      const level = examLevelOfPath(p);
      return level ? examLangHref(examLevelInfo(level).lang) : undefined;
    },
  },
  // Không phải việc HỌC như các tab kia mà là thông tin để sống và làm việc ở
  // Hàn (visa, trường, việc làm) — xem lib/guide.ts.
  {
    href: "/cam-nang/visa",
    label: "Cẩm nang",
    icon: Compass,
    match: (p) => startsWith(p, "/cam-nang"),
    children: GUIDE_SECTIONS.map((s) => ({
      href: `/cam-nang/${s.id}`,
      label: s.title,
      description: s.description,
    })),
    activeChild: (p) => /^\/cam-nang\/[^/]+/.exec(p)?.[0],
  },
  // Gồm cả trang đã đánh dấu (/my/danh-dau) — trước là một tab riêng, gộp vào
  // đây vì cùng là dữ liệu học tập cá nhân, và để thanh tab còn 4 ô rộng hơn.
  { href: "/my", label: "Góc học tập", icon: ChartNoAxesColumn, match: (p) => startsWith(p, "/my") },
];

export function activeTab(path: string): NavTab | undefined {
  return NAV_TABS.find((t) => t.match(path));
}

/**
 * Màn hình KHÔNG có thanh tab: đang làm bài (luyện tập, thi thử) đã có thanh
 * thao tác / phiếu trả lời gắn đáy, thêm thanh tab nữa là chồng nhau và dễ
 * bấm nhầm ra khỏi bài.
 */
export function hidesTabBar(path: string): boolean {
  return /^\/exam\/[^/]+\/(practice|mock)$/.test(path);
}
