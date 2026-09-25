"use client";

import { usePathname } from "next/navigation";
import { TRACKS, activeTrack, trackHref } from "@/lib/tracks";
import { SegmentedNav } from "@/components/layout/segmented-nav";

/**
 * Chọn MẢNG KIẾN THỨC của thư viện (tiếng Hàn, tiếng Anh, IT, …) — nút chọn
 * đặt ở đầu trang thư viện, vì nó chỉ áp dụng cho thư viện. Nguồn lấy từ
 * lib/tracks.ts: thêm mảng mới ở đó là có thêm mục, khỏi sửa ở đây.
 */
export function TrackNav() {
  const current = activeTrack(usePathname());
  return (
    <SegmentedNav
      label="Mảng kiến thức"
      items={TRACKS.map((track) => ({
        key: track.slug || "default",
        label: track.label,
        href: trackHref(track.slug),
        active: track.slug === current?.slug,
      }))}
    />
  );
}
