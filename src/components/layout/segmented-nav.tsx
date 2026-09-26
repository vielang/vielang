import Link from "next/link";
import { cn } from "@/lib/utils";

export type SegmentedItem = {
  key: string;
  label: string;
  active: boolean;
} & ({ href: string; onSelect?: never } | { onSelect: () => void; href?: never });

const ITEM = "shrink-0 rounded-md px-3 py-1.5 transition-colors";
const ACTIVE = "bg-background font-medium shadow-sm";
const IDLE = "text-muted-foreground hover:text-foreground";

/**
 * Hàng chọn tầng dưới cùng của một trang: bộ sách ở Thư viện, kỳ thi ở Luyện
 * thi (tầng ngôn ngữ / mục đã nằm trên header). Cùng kiểu, cùng chỗ (ngay
 * dưới tiêu đề trang), dính dưới header khi cuộn để danh sách dài vẫn đổi
 * được mà không phải cuộn ngược lên.
 *
 * Mục có `href` là link (đổi trang), mục có `onSelect` là nút (đổi trạng thái
 * ngay trong trang, vd bộ lọc).
 */
export function SegmentedNav({
  items,
  label,
  sticky = true,
}: {
  items: SegmentedItem[];
  /** Nhãn cho trình đọc màn hình, vd "Mảng kiến thức". */
  label: string;
  sticky?: boolean;
}) {
  const buttons = items.some((i) => i.onSelect);
  return (
    <nav
      aria-label={label}
      className={cn(
        "-mx-4 px-4",
        sticky &&
          "sticky top-[calc(3.5rem+env(safe-area-inset-top))] z-30 bg-background/95 py-2 backdrop-blur supports-[backdrop-filter]:bg-background/80"
      )}
    >
      <div className="flex overflow-x-auto text-sm [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <span className="flex gap-0.5 rounded-lg bg-muted p-0.5" role={buttons ? "tablist" : undefined}>
          {items.map((item) =>
            item.href !== undefined ? (
              <Link
                key={item.key}
                href={item.href}
                aria-current={item.active ? "page" : undefined}
                className={cn(ITEM, item.active ? ACTIVE : IDLE)}
              >
                {item.label}
              </Link>
            ) : (
              <button
                key={item.key}
                type="button"
                role="tab"
                aria-selected={item.active}
                onClick={item.onSelect}
                className={cn(ITEM, item.active ? ACTIVE : IDLE)}
              >
                {item.label}
              </button>
            )
          )}
        </span>
      </div>
    </nav>
  );
}
