"use client";

import { ListChecks } from "lucide-react";
import { Toggle } from "@/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { answersHidden, useReaderPrefsStore } from "@/lib/reader-prefs-store";
import { useIsClient } from "@/lib/use-is-client";

/**
 * Công tắc bật/tắt chấm đáp án cho RIÊNG một cuốn sách.
 *
 * Chấm đáp án nằm sẵn cạnh từng bài. Với sách bài tập, người muốn tự làm
 * trước rồi mới dò thì đó là cám dỗ ngay trước mắt, nên phải tắt được. Theo
 * từng sách chứ không tắt cả app, vì giáo trình thì người ta vẫn muốn tra
 * nhanh.
 *
 * Trạng thái chỉ đọc sau khi đã chạy ở trình duyệt: store nạp đồng bộ từ
 * localStorage, còn HTML server render không biết gì về nó. Đọc ngay lúc
 * render đầu thì người đã tắt sẽ thấy công tắc "bật" rồi nhảy sang "tắt", và
 * React báo lệch hydration. Trước lúc đó công tắc hiện trạng thái mặc định
 * (bật) — đúng với đa số người dùng.
 */
export function useAnswersShown(bookId: string): { shown: boolean; toggle: () => void } {
  const isClient = useIsClient();
  const hidden = useReaderPrefsStore((s) => answersHidden(s, bookId));
  const toggleAnswersHidden = useReaderPrefsStore((s) => s.toggleAnswersHidden);
  return { shown: !isClient || !hidden, toggle: () => toggleAnswersHidden(bookId) };
}

/** Công tắc bày thẳng trên hàng nút (màn rộng); điện thoại dùng mục trong menu ⋮. */
export function AnswersToggle({ bookId }: { bookId: string }) {
  const { shown, toggle } = useAnswersShown(bookId);

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Toggle
          variant="outline"
          size="sm"
          pressed={shown}
          onPressedChange={toggle}
          aria-label="Hiện chấm đáp án trên trang"
        >
          <ListChecks aria-hidden />
          {/* Chữ nói rõ trạng thái hiện tại: chỉ đổi nền nút thì trên nền
              tối rất khó đoán đang bật hay tắt. */}
          Đáp án: {shown ? "bật" : "tắt"}
        </Toggle>
      </TooltipTrigger>
      <TooltipContent>
        {shown
          ? "Đang hiện chấm xem đáp án cạnh bài tập — bấm để ẩn khi muốn tự làm trước"
          : "Đang ẩn chấm đáp án — bấm để hiện lại"}
      </TooltipContent>
    </Tooltip>
  );
}
