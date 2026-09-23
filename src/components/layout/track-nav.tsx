"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { TRACKS, activeTrack, trackHref } from "@/lib/tracks";
import { cn } from "@/lib/utils";

/**
 * Chọn MẢNG KIẾN THỨC của thư viện (tiếng Hàn, tiếng Anh, IT, …) — nút chọn
 * đặt ở đầu trang thư viện, vì nó chỉ áp dụng cho thư viện. Nguồn lấy từ
 * lib/tracks.ts: thêm mảng mới ở đó là có thêm mục, khỏi sửa ở đây.
 */
export function TrackNav() {
  const pathname = usePathname();
  const current = activeTrack(pathname);

  return (
    <nav
      aria-label="Mảng kiến thức"
      className="-mx-4 flex gap-0.5 overflow-x-auto px-4 text-sm [scrollbar-width:none] [&::-webkit-scrollbar]:hidden sm:mx-0 sm:px-0"
    >
      <span className="flex gap-0.5 rounded-lg bg-muted p-0.5">
        {TRACKS.map((track) => {
          const isActive = track.slug === current?.slug;
          return (
            <Link
              key={track.slug || "default"}
              href={trackHref(track.slug)}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "shrink-0 rounded-md px-3 py-1.5 transition-colors",
                isActive ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground"
              )}
            >
              {track.label}
            </Link>
          );
        })}
      </span>
    </nav>
  );
}
