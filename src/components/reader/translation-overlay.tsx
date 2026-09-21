"use client";

import { useState } from "react";
import { Languages, X } from "lucide-react";
import type { TranslationRegion } from "@/lib/page-translation";
import { HintBubble } from "@/components/reader/hint-bubble";

interface ActiveBubble {
  region: TranslationRegion;
  /** Toạ độ điểm bấm, theo hệ toạ độ màn hình (clientX/clientY). */
  x: number;
  y: number;
}

/**
 * Chấm xem bản dịch, đặt đè lên ảnh trang.
 *
 * Vùng đoạn văn KHÔNG hiển thị gì và cũng KHÔNG bấm được — không viền, không
 * nhãn, không nền. Trang sách phải trông y như bản in. Thứ bấm được chỉ là
 * cái chấm nhỏ ngay dưới đoạn, với lề chạm 44px quanh nó.
 *
 * Trước đây cả khung đoạn văn là nút. Nghe thì tiện hơn, nhưng một vùng dịch
 * chiếm trung bình 18% diện tích trang (trang nặng nhất tới 56%) — chừng đó
 * trang thành nút vô hình, nuốt mất cử chỉ chạm để bật thanh công cụ và hay
 * bung bản dịch lúc người ta chỉ định chạm cho hiện nút lật trang.
 *
 * Component này PHẢI nằm trong đúng khung ảnh thật (không phải khung chứa):
 * ảnh dùng `object-contain` nên có viền trống hai bên, đặt sai khung là toạ
 * độ lệch hết. Xem `page-viewer.tsx` — nó tự đo khung ảnh rồi mới render
 * component này bên trong.
 *
 * Mỗi vùng gắn `data-translate-region` để `page-viewer` biết cử chỉ bắt đầu
 * trên vùng dịch mà không tính là "chạm để ẩn/hiện thanh công cụ".
 */
export function TranslationOverlay({ regions }: { regions: TranslationRegion[] }) {
  const [active, setActive] = useState<ActiveBubble | null>(null);

  if (regions.length === 0) return null;

  return (
    <>
      {regions.map((region) => {
        const [x, y, w, h] = region.rect;
        const isOpen = active?.region.id === region.id;
        const what = region.label ?? "đoạn này";
        return (
          // Khung này CHỈ để định vị cái chấm vào đúng mép dưới của đoạn —
          // nó không bấm được. `pointer-events-none` là chủ ý, không phải
          // thừa: trước đây cả khung là nút, mà một vùng dịch chiếm trung
          // bình 18% diện tích trang (trang nặng nhất tới 56%). Chừng đó
          // trang biến thành nút vô hình, kéo theo hai chuyện:
          //
          //  - Chạm vào đó không bật/tắt được thanh công cụ nữa, vì
          //    `page-viewer` bỏ qua tap bắt đầu trên vùng dịch.
          //  - Chỉ định chạm để hiện nút lật trang thì lại bung bản dịch.
          //
          // Thứ NHÌN THẤY là cái chấm 20px, nên thứ BẤM ĐƯỢC cũng phải là
          // nó. Vùng bấm vô hình to gấp trăm lần thứ vẽ ra là nói dối người
          // dùng.
          <div
            key={region.id}
            className="pointer-events-none absolute"
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          >
            {/* Dấu hiệu DUY NHẤT cho biết đoạn này có bản dịch. Vùng bấm vẫn
                trong suốt như cũ để trang giữ nguyên dáng bản in — chỉ một
                chấm nhỏ ở góc, đủ để người ta biết mà chạm vào. Không có nó
                thì cả tính năng này tàng hình.

                Đặt NGAY DƯỚI vùng, canh giữa: người ta đọc hết đoạn rồi mới
                cần bản dịch, nên chấm nằm ở chỗ mắt vừa dừng lại. Canh giữa
                thay vì nép vào góc để nó không đụng chữ của cột bên cạnh —
                nhiều trang có hai cột sát nhau.

                `top-full` chứ không phải `-bottom-*`: lệch âm chỉ đẩy chấm
                ra một phần, phần còn lại vẫn nằm đè lên dòng cuối của đoạn.
                Neo mép TRÊN của chấm vào mép DƯỚI của vùng thì nó ra hẳn
                ngoài, không che chữ nào.

                Đang mở thì chấm đổi thành dấu X. Bấm ra ngoài vốn đã đóng
                được (xem hiệu ứng trong TranslationBubble), nhưng không có
                gì nói ra điều đó — đổi icon là cách rẻ nhất để người dùng
                thấy có đường đóng, ngay tại chỗ họ vừa bấm để mở.

                KHÔNG nền, chỉ nét mực đen — để trang giữ được dáng bản in.
                Đổi lại icon phải tự lo tương phản: nó nằm trên đủ thứ nền
                của ảnh scan, có trang là bảng nền đen (vd trang 189) thì nét
                đen trơn biến mất hẳn. Viền sáng quanh nét giải quyết việc đó,
                cùng cách `audio-widget` đang dùng cho chữ của nó.

                Màu cố định chứ KHÔNG dùng token theme: ảnh trang sách lúc
                nào cũng là giấy in sáng, kể cả khi app đang ở chế độ tối —
                dùng `text-foreground` thì chế độ tối sẽ lật nét thành trắng
                và mất hút trên giấy trắng. */}
            {/* Vùng chạm 44px (size-11) nhưng nét vẽ vẫn chỉ 20px như cũ —
                phần dôi ra là lề vô hình quanh chấm. 20px là quá nhỏ cho
                ngón tay (Apple khuyên 44, Google 48), mà phóng to cái chấm
                cho dễ bấm thì lại chọc vào dáng bản in của trang.

                `-mt-2` kéo nút lên 8px để TÂM nút trùng đúng tâm cái chấm ở
                chỗ cũ (mép dưới vùng + 4px lề + nửa của 20px = +14px; nút
                44px đặt ở -8px thì tâm cũng rơi vào +14px). Nhờ vậy đổi vùng
                chạm mà không xê dịch thứ người dùng nhìn thấy. */}
            <button
              type="button"
              data-translate-region
              onClick={(e) =>
                setActive(isOpen ? null : { region, x: e.clientX, y: e.clientY })
              }
              aria-label={isOpen ? `Đóng bản dịch: ${what}` : `Xem bản dịch: ${what}`}
              aria-expanded={isOpen}
              className="group pointer-events-auto absolute top-full left-1/2 -mt-2 flex size-11 -translate-x-1/2 cursor-help items-center justify-center"
            >
              <span
                className="flex size-5 items-center justify-center text-neutral-900 transition-transform group-hover:scale-110"
                style={{
                  filter:
                    "drop-shadow(0 0 1.5px rgb(255 255 255)) drop-shadow(0 0 1.5px rgb(255 255 255))",
                }}
                aria-hidden
              >
                {isOpen ? <X className="size-4" /> : <Languages className="size-4" />}
              </span>
            </button>
          </div>
        );
      })}

      {active && (
        // key gồm cả toạ độ bấm: mở lại cùng một vùng ở chỗ khác thì bong bóng
        // dựng mới, xoá vị trí người dùng đã kéo lần trước.
        <HintBubble
          key={`${active.region.id}:${active.x}:${active.y}`}
          x={active.x}
          y={active.y}
          onClose={() => setActive(null)}
          ownTriggerSelector="[data-translate-region]"
        >
          {/* `whitespace-pre-line`: bản dịch hội thoại xuống dòng theo từng
              lượt nói, giữ nguyên mới đọc ra ai nói câu nào. */}
          <span className="whitespace-pre-line">{active.region.vi}</span>
        </HintBubble>
      )}
    </>
  );
}

