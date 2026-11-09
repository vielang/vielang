import Link from "next/link";
import { SearchX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/layout/empty-state";

/**
 * 404 trong khu có header và thanh tab — cho mọi thứ không tìm thấy: khoá
 * học, bài học, bài cẩm nang.
 */
export default function MainNotFound() {
  return (
    <EmptyState
      as="h1"
      icon={SearchX}
      title="Không tìm thấy trang"
      description="Nội dung bạn tìm không tồn tại hoặc đã được đổi địa chỉ."
      action={
        <Button asChild>
          <Link href="/">Về trang chủ</Link>
        </Button>
      }
    />
  );
}
