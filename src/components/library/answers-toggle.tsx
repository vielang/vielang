"use client";

import { useSyncExternalStore } from "react";
import { ListChecks } from "lucide-react";
import { Toggle } from "@/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { answersHidden, useReaderPrefsStore } from "@/lib/reader-prefs-store";

const NO_SUBSCRIBE = () => () => {};

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
export function AnswersToggle({ bookId }: { bookId: string }) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const hidden = useReaderPrefsStore((s) => answersHidden(s, bookId));
  const toggle = useReaderPrefsStore((s) => s.toggleAnswersHidden);
  const shown = !isClient || !hidden;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Toggle
          variant="outline"
          size="sm"
          pressed={shown}
          onPressedChange={() => toggle(bookId)}
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
