import Link from "next/link";
import { SearchX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/layout/empty-state";

/** 404 ngoài khu có header — chiếm cả màn hình. */
export default function NotFound() {
  return (
    <EmptyState
      as="h1"
      icon={SearchX}
      title="Không tìm thấy trang"
      description="Trang bạn tìm không tồn tại."
      className="min-h-dvh p-6"
      action={
        <Button asChild>
          <Link href="/">Về trang chủ</Link>
        </Button>
      }
    />
  );
}
