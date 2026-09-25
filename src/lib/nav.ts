import { Bookmark, ChartNoAxesColumn, Compass, GraduationCap, Library, type LucideIcon } from "lucide-react";
import { TRACKS } from "@/lib/tracks";

/**
 * Điều hướng chính của app — NGUỒN DUY NHẤT cho thanh tab dưới đáy (điện
 * thoại) và nav trong header (máy tính).
 *
 * Các mục này cố ý đặt theo VIỆC người học làm, không theo nội dung: thêm
 * giáo trình mới, thêm mảng kiến thức mới (lib/tracks) hay thêm kỳ thi mới
 * (IELTS… bên cạnh TOPIK) đều nằm gọn trong "Thư viện" / "Luyện thi", không
 * phải đụng vào thanh điều hướng. Chọn mảng kiến thức nằm trong trang thư
 * viện vì nó chỉ áp dụng cho thư viện.
 */
export interface NavTab {
  href: string;
  label: string;
  icon: LucideIcon;
  /** Đường dẫn này có thuộc tab không (để tô tab đang mở). */
  match: (path: string) => boolean;
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
      TRACKS.some(
        (t) =>
          (t.slug && startsWith(p, `/${t.slug}`)) ||
          (t.paths?.some((base) => startsWith(p, base)) ?? false)
      ),
  },
  { href: "/exam", label: "Luyện thi", icon: GraduationCap, match: (p) => startsWith(p, "/exam") },
  // Không phải việc HỌC như bốn tab kia mà là thông tin để sống và làm việc ở
  // Hàn (visa, trường, việc làm) — xem lib/guide.ts.
  { href: "/cam-nang", label: "Cẩm nang", icon: Compass, match: (p) => startsWith(p, "/cam-nang") },
  { href: "/my", label: "Góc học tập", icon: ChartNoAxesColumn, match: (p) => startsWith(p, "/my") },
  { href: "/bookmarks", label: "Đánh dấu", icon: Bookmark, match: (p) => startsWith(p, "/bookmarks") },
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
