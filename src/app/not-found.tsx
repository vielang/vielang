import Link from "next/link";
import { SearchX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/layout/empty-state";

/** 404 ngoài khu có header (vd đường dẫn trình đọc sai) — chiếm cả màn hình. */
export default function NotFound() {
  return (
    <EmptyState
      as="h1"
      icon={SearchX}
      title="Không tìm thấy trang"
      description="Trang hoặc sách bạn tìm không tồn tại."
      className="min-h-dvh p-6"
      action={
        <Button asChild>
          <Link href="/">Về thư viện</Link>
        </Button>
      }
    />
  );
}
