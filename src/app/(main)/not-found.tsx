import Link from "next/link";
import { SearchX } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * 404 trong khu có header và thanh tab. Chữ nói CHUNG ("trang") chứ không
 * nói "sách": trang này hiện cho mọi thứ không tìm thấy — sách, đề thi, khoá
 * học, bài cẩm nang — trước đây ghi "Không tìm thấy sách" cho tất cả.
 */
export default function MainNotFound() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-24 text-center">
      <SearchX className="size-10 text-muted-foreground" aria-hidden />
      <div>
        <h1 className="text-xl font-semibold">Không tìm thấy trang</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Nội dung bạn tìm không tồn tại hoặc đã được đổi địa chỉ.
        </p>
      </div>
      <Button asChild>
        <Link href="/">Về thư viện</Link>
      </Button>
    </div>
  );
}
