import { ChartNoAxesColumn, Compass, Library, type LucideIcon } from "lucide-react";
import { GUIDE_SECTIONS } from "@/lib/guide-sections";

/**
 * Điều hướng chính của app — NGUỒN DUY NHẤT cho thanh tab dưới đáy (điện
 * thoại) và nav trong header (máy tính).
 *
 * Tab có thể có MỤC CON (mục cẩm nang). Mục con khai báo MỘT lần ở đây, rồi
 * ba nơi cùng đọc: hàng tab trong trang, header điện thoại (thay chỗ logo)
 * và menu thả xuống trên header máy tính.
 */
export interface NavChild {
  href: string;
  label: string;
  /** Một dòng ngắn dưới nhãn trong menu thả xuống. */
  description?: string;
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

/** Cẩm nang chỉ hiện khi đã có mục bài — tab dẫn vào trang trống thì thà không có. */
const hasGuide = GUIDE_SECTIONS.length > 0;

const ALL_TABS: (NavTab | false)[] = [
  // Trang chủ là danh sách khoá học; trang khoá `/it/<khoá>`, bài `/learn/…`.
  {
    href: "/",
    label: "Khoá học",
    icon: Library,
    match: (p) => p === "/" || startsWith(p, "/it") || startsWith(p, "/learn"),
  },
  // Không phải bài HỌC mà là kiến thức nghề — lộ trình, phỏng vấn, công cụ,
  // việc làm IT (xem lib/guide.ts).
  hasGuide && {
    href: `/cam-nang/${GUIDE_SECTIONS[0]?.id ?? ""}`,
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
  { href: "/my", label: "Góc học tập", icon: ChartNoAxesColumn, match: (p) => startsWith(p, "/my") },
];

export const NAV_TABS: NavTab[] = ALL_TABS.filter((t): t is NavTab => t !== false);

export function activeTab(path: string): NavTab | undefined {
  return NAV_TABS.find((t) => t.match(path));
}
