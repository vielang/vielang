import Link from "next/link";
import { ChevronRight } from "lucide-react";

/**
 * Danh sách dạng dòng của các trang cấp một (đề thi, bài cẩm nang): không
 * khung bao ngoài, kẻ mảnh giữa các dòng, trên điện thoại tràn hết bề ngang.
 * Trước đây đề thi và bài cẩm nang mỗi bên một kiểu dòng (khác lề, khác kẻ).
 */
export function RowList({ children }: { children: React.ReactNode }) {
  return <ul className="-mx-4 flex flex-col divide-y divide-border/60 sm:mx-0">{children}</ul>;
}

/** Một dòng bấm được: nội dung tuỳ ý, mũi tên ở cuối. Đặt trong `RowList`. */
export function RowLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <li>
      <Link
        href={href}
        className="group flex items-center gap-3 px-4 py-3.5 transition-colors hover:bg-muted/50 active:bg-muted sm:-mx-3 sm:rounded-lg sm:px-3"
      >
        {children}
        <ChevronRight
          className="size-4 shrink-0 text-muted-foreground/60 transition-transform group-hover:translate-x-0.5"
          aria-hidden
        />
      </Link>
    </li>
  );
}
