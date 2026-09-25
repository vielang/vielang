"use client";

import { useState, useSyncExternalStore } from "react";
import {
  Check,
  Download,
  MoreVertical,
  Share,
  SquarePlus,
  WifiOff,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  currentPlatform,
  useInstallStore,
  type Platform,
} from "@/lib/install-store";

interface Step {
  /** Icon minh hoạ đúng cái nút người dùng phải tìm trên màn hình của họ. */
  Icon?: typeof Share;
  text: string;
}

const GUIDES: Record<Platform, { label: string; note?: string; steps: Step[] }> = {
  ios: {
    label: "iPhone / iPad",
    // Chỗ vấp phổ biến nhất trên iOS: mở bằng Chrome thì không có mục này.
    note: "Phải mở bằng Safari. Chrome trên iPhone không thêm được vào màn hình chính.",
    steps: [
      { Icon: Share, text: "Bấm nút Chia sẻ ở thanh dưới màn hình" },
      { Icon: SquarePlus, text: "Kéo xuống, chọn “Thêm vào MH chính”" },
      { text: "Bấm “Thêm” ở góc phải trên" },
    ],
  },
  android: {
    label: "Android",
    steps: [
      { Icon: MoreVertical, text: "Bấm nút ba chấm ở góc phải trên Chrome" },
      { Icon: Download, text: "Chọn “Cài đặt ứng dụng” hoặc “Thêm vào Màn hình chính”" },
      { text: "Bấm “Cài đặt” để xác nhận" },
    ],
  },
  desktop: {
    label: "Máy tính",
    steps: [
      { Icon: Download, text: "Bấm biểu tượng cài đặt ở cuối thanh địa chỉ" },
      { text: "Chọn “Cài đặt” trong hộp thoại hiện ra" },
    ],
  },
};

const ORDER: Platform[] = ["ios", "android", "desktop"];

/**
 * Hệ điều hành của thiết bị, đọc an toàn qua ranh giới server/client.
 *
 * Server không biết gì về thiết bị nên trả `null`; client trả kết quả đoán
 * được. `useSyncExternalStore` lo đúng việc đó — dùng effect rồi `setState`
 * thì React phải render hai lượt, và chính nó cũng cảnh báo.
 *
 * Không bao giờ đổi sau khi tải trang, nên `subscribe` là hàm rỗng.
 */
import { NO_SUBSCRIBE } from "@/lib/use-is-client";

function useDetectedPlatform(): Platform | null {
  return useSyncExternalStore(NO_SUBSCRIBE, currentPlatform, () => null);
}

/**
 * Hướng dẫn cài app vào màn hình chính.
 *
 * Tự mở sẵn đúng hệ điều hành người dùng đang dùng, nhưng vẫn cho xem hai
 * cái kia — người ta hay mở trên máy tính rồi làm theo trên điện thoại.
 *
 * Nếu Chrome đã cho phép cài bằng một nút bấm thật thì đưa nút đó lên trước
 * và để hướng dẫn xuống dưới: bấm một cái bao giờ cũng hơn đọc ba bước.
 */
export function InstallGuide() {
  const event = useInstallStore((s) => s.event);
  const installed = useInstallStore((s) => s.installed);
  const setEvent = useInstallStore((s) => s.setEvent);
  const detected = useDetectedPlatform();
  /** Người dùng tự chọn tab khác — hay gặp: xem trên máy tính, làm trên điện thoại. */
  const [picked, setPicked] = useState<Platform | null>(null);

  if (installed) {
    return (
      <div className="flex items-center gap-3 rounded-xl border border-border bg-card p-4">
        <Check className="size-5 shrink-0 text-primary" aria-hidden />
        <div>
          <p className="text-sm font-medium">Bạn đang dùng bản đã cài</p>
          <p className="text-xs text-muted-foreground">
            App đã nằm ở màn hình chính — không cần cài lại.
          </p>
        </div>
      </div>
    );
  }

  const active = picked ?? detected ?? "android";

  return (
    <div className="flex flex-col gap-6">
      {event && (
        <div className="flex flex-col gap-3 rounded-xl border border-border bg-card p-4">
          <div>
            <p className="text-sm font-medium">Cài ngay trên máy này</p>
            <p className="text-xs text-muted-foreground">
              Trình duyệt của bạn cài được chỉ bằng một lần bấm.
            </p>
          </div>
          <Button
            className="w-full sm:w-auto"
            onClick={async () => {
              await event.prompt();
              await event.userChoice;
              // Lời mời của Chrome chỉ dùng được một lần — bỏ đi để nút
              // không còn đó mà bấm vào chẳng ra gì.
              setEvent(null);
            }}
          >
            <Download className="size-4" aria-hidden />
            Cài ứng dụng
          </Button>
        </div>
      )}

      <div className="flex flex-col gap-3">
        <div role="tablist" className="flex gap-1 border-b border-border">
          {ORDER.map((p) => (
            <button
              key={p}
              type="button"
              role="tab"
              aria-selected={active === p}
              onClick={() => setPicked(p)}
              className={cn(
                "-mb-px border-b-2 px-3 py-2 text-sm transition-colors",
                active === p
                  ? "border-primary font-medium text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              )}
            >
              {GUIDES[p].label}
            </button>
          ))}
        </div>

        <div className="flex flex-col gap-3">
          {GUIDES[active].note && (
            <p className="rounded-lg bg-muted px-3 py-2 text-xs text-muted-foreground">
              {GUIDES[active].note}
            </p>
          )}
          <ol className="flex flex-col gap-2.5">
            {GUIDES[active].steps.map(({ Icon, text }, i) => (
              <li key={text} className="flex items-start gap-3">
                <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-secondary text-xs font-medium tabular-nums">
                  {i + 1}
                </span>
                <span className="flex flex-1 items-center gap-2 pt-0.5 text-sm">
                  {text}
                  {Icon && (
                    <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden />
                  )}
                </span>
              </li>
            ))}
          </ol>
        </div>
      </div>

      <div className="flex items-start gap-3 rounded-xl border border-dashed border-border p-4">
        <WifiOff className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden />
        <p className="text-xs text-muted-foreground">
          Cài xong, vào trang bất kỳ cuốn sách nào và bấm{" "}
          <span className="font-medium text-foreground">Tải về đọc offline</span> để
          mang cả cuốn theo — đọc được cả khi không có mạng.
        </p>
      </div>
    </div>
  );
}
