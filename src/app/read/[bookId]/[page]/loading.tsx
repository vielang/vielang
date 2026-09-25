import { Loader2 } from "lucide-react";

/**
 * Màn chờ khi mở trang đọc. Trang đọc dựng ở máy chủ mỗi lần (route động)
 * và lưới trang sách cố ý không tải trước từng ô (xem PageGrid) — thiếu màn
 * này thì bấm vào một ô xong màn hình đứng im tới khi máy chủ trả lời, người
 * dùng tưởng chưa bấm trúng.
 *
 * Nền đen như chính trình đọc: chuyển sang không bị chớp trắng. Vòng xoay chỉ
 * hiện sau một nhịp (animation-delay) — mạng nhanh thì trang đọc tới trước,
 * không có gì nháy lên.
 */
export default function ReaderLoading() {
  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black" role="status" aria-label="Đang mở trang">
      <span className="show-after-delay">
        <Loader2 className="size-8 animate-spin text-white/60" aria-hidden />
      </span>
    </div>
  );
}
