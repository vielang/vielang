import Link from "next/link";
import { SearchX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/layout/empty-state";

/**
 * 404 trong khu có header và thanh tab. Chữ nói CHUNG ("trang") chứ không
 * nói "sách": trang này hiện cho mọi thứ không tìm thấy — sách, đề thi, khoá
 * học, bài cẩm nang — trước đây ghi "Không tìm thấy sách" cho tất cả.
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
          <Link href="/">Về thư viện</Link>
        </Button>
      }
    />
  );
}
