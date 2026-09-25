import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * Màn trống / màn báo: biểu tượng lớn, tiêu đề, một đoạn giải thích, nút đi
 * tiếp. Dùng cho 404, mất mạng, "sắp có nội dung", "chưa đánh dấu trang nào".
 * Trước đây chép tay ở 6 chỗ, lệch nhau cỡ chữ tiêu đề và khoảng cách.
 *
 * `dashed`: khung viền đứt — cho màn trống NẰM TRONG một trang (danh sách
 * chưa có gì), để nó đọc là "một ô đang trống" chứ không phải cả trang lỗi.
 * `as`: thẻ tiêu đề — `h1` khi màn này là cả trang (404, mất mạng).
 */
export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  dashed = false,
  as: Heading = "h2",
  className,
}: {
  icon: LucideIcon;
  title: string;
  description?: React.ReactNode;
  action?: React.ReactNode;
  dashed?: boolean;
  as?: "h1" | "h2";
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-4 py-20 text-center",
        dashed && "rounded-xl border border-dashed border-border",
        className
      )}
    >
      <Icon className="size-10 text-muted-foreground" aria-hidden />
      <div>
        <Heading className="text-lg font-semibold">{title}</Heading>
        {description && (
          <p className="mx-auto mt-1 max-w-sm text-sm text-muted-foreground">{description}</p>
        )}
      </div>
      {action}
    </div>
  );
}
