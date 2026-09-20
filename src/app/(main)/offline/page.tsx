import type { Metadata } from "next";
import Link from "next/link";
import { WifiOff } from "lucide-react";
import { Button } from "@/components/ui/button";

export const metadata: Metadata = { title: "Không có mạng" };

/**
 * Trang service worker trả về khi mất mạng mà trang được xin lại chưa từng
 * xem (xem `public/sw.js`). Không có nó thì người dùng nhận màn báo lỗi trần
 * của trình duyệt và tưởng app hỏng.
 */
export default function OfflinePage() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-24 text-center">
      <WifiOff className="size-10 text-muted-foreground" aria-hidden />
      <div>
        <h1 className="text-xl font-semibold">Đang không có mạng</h1>
        <p className="mx-auto mt-1 max-w-sm text-sm text-muted-foreground">
          Những trang sách bạn đã mở trước đó vẫn đọc được bình thường. Trang
          này thì chưa tải về máy lần nào.
        </p>
      </div>
      <Button asChild variant="outline">
        <Link href="/">Về thư viện</Link>
      </Button>
    </div>
  );
}
