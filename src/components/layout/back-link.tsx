import Link from "next/link";
import { ArrowLeft } from "lucide-react";

/**
 * Lối quay lại trang cha ở đầu trang con ("← Luyện thi TOEIC", "← Góc học
 * tập"…). Một kiểu cho mọi trang — trước đây chép tay ở 7 chỗ, lệch nhau cả
 * `w-fit` (thiếu thì vùng bấm kéo dài hết hàng).
 */
export function BackLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link
      href={href}
      className="inline-flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
    >
      <ArrowLeft className="size-4 shrink-0" aria-hidden />
      {children}
    </Link>
  );
}
