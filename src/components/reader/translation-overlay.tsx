"use client";

import { useState } from "react";
import { Languages, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { TranslationRegion } from "@/lib/page-translation";

/**
 * Các vùng bấm xem bản dịch, đặt đè lên ảnh trang.
 *
 * Component này PHẢI nằm trong đúng khung ảnh thật (không phải khung khung
 * chứa): ảnh dùng `object-contain` nên có viền trống hai bên, đặt sai khung
 * là toạ độ lệch hết. Xem `page-viewer.tsx` — nó tự đo khung ảnh rồi mới
 * render component này bên trong.
 *
 * Mỗi vùng gắn `data-translate-region` để `page-viewer` biết cử chỉ bắt đầu
 * trên vùng dịch mà không tính là "chạm để ẩn/hiện thanh công cụ".
 */
export function TranslationOverlay({
  regions,
  visible,
}: {
  regions: TranslationRegion[];
  /** Ẩn cùng thanh công cụ để lúc đọc thuần tuý không bị viền vùng làm rối. */
  visible: boolean;
}) {
  const [active, setActive] = useState<TranslationRegion | null>(null);

  if (regions.length === 0) return null;

  return (
    <>
      {regions.map((region) => {
        const [x, y, w, h] = region.rect;
        return (
          <button
            key={region.id}
            type="button"
            data-translate-region
            onClick={() => setActive(region)}
            aria-label={`Xem bản dịch: ${region.label ?? "đoạn này"}`}
            className={cn(
              "absolute rounded-md border-2 transition-opacity",
              visible
                ? "border-sky-400/80 bg-sky-400/10 hover:bg-sky-400/25"
                : "border-transparent bg-transparent"
            )}
            style={{
              left: `${x * 100}%`,
              top: `${y * 100}%`,
              width: `${w * 100}%`,
              height: `${h * 100}%`,
            }}
          >
            {visible && region.label && (
              <span className="absolute -top-px left-0 rounded-br-md rounded-tl-md bg-sky-500 px-1.5 py-0.5 text-[10px] font-medium whitespace-nowrap text-white">
                {region.label}
              </span>
            )}
          </button>
        );
      })}

      {active && <TranslationCard region={active} onClose={() => setActive(null)} />}
    </>
  );
}

/**
 * Thẻ bản dịch — `fixed` nên KHÔNG bị biến đổi theo zoom/pan của ảnh; chữ
 * luôn ở cỡ đọc được dù người dùng đang phóng to trang tới đâu.
 */
function TranslationCard({
  region,
  onClose,
}: {
  region: TranslationRegion;
  onClose: () => void;
}) {
  return (
    <div className="fixed inset-x-0 bottom-0 z-50 flex justify-center p-3">
      <div className="max-h-[50vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-border bg-popover text-popover-foreground shadow-2xl">
        <div className="flex items-center gap-2 border-b border-border px-3 py-2">
          <Languages className="size-4 shrink-0 text-muted-foreground" aria-hidden />
          <span className="min-w-0 flex-1 truncate text-xs font-medium">
            {region.label ?? "Bản dịch"}
          </span>
          <Button variant="ghost" size="icon-sm" onClick={onClose} aria-label="Đóng">
            <X className="size-4" aria-hidden />
          </Button>
        </div>

        <div className="flex flex-col gap-2 px-3 py-3">
          {region.ko && (
            <p className="font-korean text-sm whitespace-pre-line text-muted-foreground">
              {region.ko}
            </p>
          )}
          <p className="text-sm whitespace-pre-line">{region.vi}</p>
        </div>
      </div>
    </div>
  );
}
