import Link from "next/link";
import { ChevronRight, Clapperboard } from "lucide-react";

/**
 * Lối vào "Học tiếng Hàn qua video" ngay trên trang Thư viện tiếng Hàn —
 * xem `library-view.tsx`. Một dòng banner, không phải một mục trong lưới
 * sách, vì đây là loại nội dung khác hẳn (phim, không phải trang ảnh).
 */
export function VideoPromoCard() {
  return (
    <Link
      href="/video"
      className="group focus-visible:ring-ring flex items-center gap-3 rounded-xl border border-border bg-card p-3 transition-shadow hover:shadow-md focus-visible:outline-none focus-visible:ring-2 sm:p-4"
    >
      <div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
        <Clapperboard className="size-5" aria-hidden />
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">Học tiếng Hàn qua video</p>
        <p className="text-xs text-muted-foreground">
          Xem phim có phụ đề tiếng Hàn, bật/tắt phụ đề tiếng Việt khi cần
        </p>
      </div>
      <ChevronRight
        className="size-4 shrink-0 text-muted-foreground/60 transition-transform group-hover:translate-x-0.5"
        aria-hidden
      />
    </Link>
  );
}
