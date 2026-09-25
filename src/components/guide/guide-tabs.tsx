import Link from "next/link";
import { GUIDE_SECTIONS, type GuideSectionId } from "@/lib/guide";
import { cn } from "@/lib/utils";

/**
 * Hàng tab chọn mục của cẩm nang (Visa, Trường đại học, Việc làm) — cùng
 * kiểu với hàng chọn mảng của Thư viện (TrackNav).
 *
 * Mỗi mục một trang riêng thay vì xếp chồng trên một trang: mục nào cũng sẽ
 * có hàng chục bài, xếp chồng thì muốn xem Việc làm phải cuộn qua hết Visa
 * và Trường. Nguồn lấy từ GUIDE_SECTIONS: thêm mục ở đó là có thêm tab.
 *
 * DÍNH ngay dưới header khi cuộn: danh sách dài vẫn đổi mục được mà không
 * phải cuộn ngược lên đầu.
 */
export function GuideTabs({ active }: { active: GuideSectionId }) {
  return (
    <nav
      aria-label="Mục cẩm nang"
      className="sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-30 -mx-4 bg-background/95 px-4 py-2 backdrop-blur supports-[backdrop-filter]:bg-background/80"
    >
      <div className="flex gap-0.5 overflow-x-auto text-sm [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <span className="flex gap-0.5 rounded-lg bg-muted p-0.5">
          {GUIDE_SECTIONS.map((section) => {
            const isActive = section.id === active;
            return (
              <Link
                key={section.id}
                href={`/cam-nang/${section.id}`}
                aria-current={isActive ? "page" : undefined}
                className={cn(
                  "shrink-0 rounded-md px-3 py-1.5 transition-colors",
                  isActive ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground"
                )}
              >
                {section.title}
              </Link>
            );
          })}
        </span>
      </div>
    </nav>
  );
}
