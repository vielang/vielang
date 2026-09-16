import Image from "next/image";
import { getPageUrl } from "@/lib/books";

/**
 * Preload ảnh các trang liền kề spread đang xem bằng chính next/image, dùng
 * cùng `sizes="100vw"` như ảnh đang đọc để trình duyệt chọn đúng URL srcset
 * sẽ cần khi chuyển trang (đã nằm sẵn trong cache) — chuyển trang/spread
 * mượt, không giật. Ẩn hoàn toàn khỏi layout & màn hình đọc.
 */
export function AdjacentPreload({
  bookId,
  pages,
  totalPages,
}: {
  bookId: string;
  /** Trang (hoặc 2 trang) đang hiện — preload phía trước trang đầu và phía sau trang cuối. */
  pages: number[];
  totalPages: number;
}) {
  const first = Math.min(...pages);
  const last = Math.max(...pages);
  const targets = [first - 2, first - 1, last + 1, last + 2].filter(
    (p) => p >= 1 && p <= totalPages && !pages.includes(p)
  );
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
