import Image from "next/image";
import { getPageUrl } from "@/lib/books";

/**
 * Preload ảnh trang kế tiếp/trước bằng chính next/image, dùng cùng
 * `sizes="100vw"` như ảnh đang đọc để trình duyệt chọn đúng URL srcset sẽ
 * cần khi chuyển trang (đã nằm sẵn trong cache) — chuyển trang mượt, không
 * giật. Ẩn hoàn toàn khỏi layout & màn hình đọc.
 */
export function AdjacentPreload({
  bookId,
  page,
  totalPages,
}: {
  bookId: string;
  page: number;
  totalPages: number;
}) {
  const targets = [page - 1, page + 1].filter((p) => p >= 1 && p <= totalPages);
  if (targets.length === 0) return null;

  return (
    <div aria-hidden className="pointer-events-none fixed inset-0 -z-10 overflow-hidden opacity-0">
      {targets.map((p) => (
        <Image
          key={p}
          src={getPageUrl(bookId, p)}
          alt=""
          fill
          sizes="100vw"
          quality={90}
          loading="eager"
        />
      ))}
    </div>
  );
}
