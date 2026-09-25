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
 */
export function GuideTabs({ active }: { active: GuideSectionId }) {
  return (
    <nav
      aria-label="Mục cẩm nang"
      className="-mx-4 flex gap-0.5 overflow-x-auto px-4 text-sm [scrollbar-width:none] [&::-webkit-scrollbar]:hidden sm:mx-0 sm:px-0"
    >
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
    </nav>
  );
}
