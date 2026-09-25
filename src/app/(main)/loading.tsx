import { Skeleton } from "@/components/ui/skeleton";

/**
 * Màn chờ chung cho mọi trang trong khu có header và thanh tab (Thư viện,
 * Luyện thi, Cẩm nang, Góc học tập, trang đề thi…).
 *
 * Khung xương TRUNG TÍNH theo bố cục chung của các trang này (PageHeader +
 * danh sách; hàng chọn tầng dưới chỉ vài trang có nên không vẽ) — trước đây
 * vẽ lưới bìa sách, nên mở Luyện thi hay Cẩm nang cũng thấy chớp lên một lưới
 * sách không liên quan. Trang chi tiết
 * sách có màn chờ riêng (books/[bookId]/loading.tsx).
 */
export default function MainLoading() {
  return (
    <div className="flex max-w-3xl flex-col gap-6" role="status" aria-label="Đang tải">
      <div>
        <Skeleton className="h-7 w-40" />
        <Skeleton className="mt-2 h-4 w-72 max-w-full" />
      </div>
      <div className="flex flex-col divide-y divide-border/60">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="flex flex-col gap-1.5 py-3.5">
            <Skeleton className="h-4 w-2/3" />
            <Skeleton className="h-3 w-1/2" />
          </div>
        ))}
      </div>
    </div>
  );
}
