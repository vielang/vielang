import type { Metadata } from "next";
import Link from "next/link";
import { WifiOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/layout/empty-state";

export const metadata: Metadata = { title: "Không có mạng" };

/**
 * Trang service worker trả về khi mất mạng mà trang được xin lại chưa từng
 * xem (xem `public/sw.js`). Không có nó thì người dùng nhận màn báo lỗi trần
 * của trình duyệt và tưởng app hỏng.
 */
export default function OfflinePage() {
  return (
    <EmptyState
      as="h1"
      icon={WifiOff}
      title="Đang không có mạng"
      description="Những trang sách bạn đã mở trước đó vẫn đọc được bình thường. Trang này thì chưa tải về máy lần nào."
      action={
        <Button asChild variant="outline">
          <Link href="/">Về thư viện</Link>
        </Button>
      }
    />
  );
}
