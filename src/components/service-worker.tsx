"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw, X } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * Đăng ký service worker (xem `public/sw.js`) và mời người dùng cập nhật khi
 * có bản mới đang chờ.
 *
 * CHỈ chạy ở bản dựng thật. Ở `next dev`, service worker sẽ cache nhầm chunk
 * của HMR và gây ra những lỗi "sao sửa code mà không thấy đổi" rất khó truy.
 *
 * Vì sao cần lời mời này, trong khi code app vốn đã tự cập nhật: HTML đi
 * mạng trước nên deploy xong là trang mới tới ngay. Nhưng CHÍNH `sw.js` thì
 * không — bản mới đứng chờ tới khi mọi tab đóng hết, mà PWA đã cài trên điện
 * thoại thì người dùng gần như không bao giờ đóng hẳn. Nên mỗi khi luật cache
 * đổi hay vá lỗi offline, phải có đường để nó tới được người dùng.
 *
 * Nói cách khác: thanh này KHÔNG phải "có tính năng mới", mà là "phần chạy
 * offline có bản mới".
 */
export function ServiceWorker() {
  const [waiting, setWaiting] = useState<ServiceWorker | null>(null);
  const [dismissed, setDismissed] = useState(false);
  /** Chặn tải lại nhiều lần khi service worker đổi. */
  const reloading = useRef(false);

  useEffect(() => {
    if (process.env.NODE_ENV !== "production") return;
    if (!("serviceWorker" in navigator)) return;

    let registration: ServiceWorkerRegistration | null = null;

    // Bản mới tiếp quản -> tải lại ngay, để trang đang chạy không lệch phiên
    // bản với service worker vừa lên.
    const onControllerChange = () => {
      if (reloading.current) return;
      reloading.current = true;
      window.location.reload();
    };
    navigator.serviceWorker.addEventListener("controllerchange", onControllerChange);

    navigator.serviceWorker
      .register("/sw.js")
      .then((reg) => {
        registration = reg;

        // Đã có bản chờ sẵn từ lần mở trước.
        if (reg.waiting && navigator.serviceWorker.controller) setWaiting(reg.waiting);

        reg.addEventListener("updatefound", () => {
          const installing = reg.installing;
          if (!installing) return;
          installing.addEventListener("statechange", () => {
            // `controller` có nghĩa đây không phải lần cài đầu tiên — lần đầu
            // thì không có gì để "cập nhật", đừng làm phiền.
            if (installing.state === "installed" && navigator.serviceWorker.controller) {
              setWaiting(reg.waiting ?? installing);
            }
          });
        });
      })
      .catch(() => {
        // Mất offline là tiếc, nhưng chặn cả app vì không đăng ký được thì vô
        // lý. Vài trình duyệt cũng chặn hẳn service worker ở chế độ riêng tư.
      });

    // PWA đã cài thì hiếm khi được tải lại — hỏi lại mỗi lần người dùng quay
    // về app, nếu không bản mới có thể nằm im hàng tuần.
    const onVisible = () => {
      if (document.visibilityState === "visible") void registration?.update();
    };
    document.addEventListener("visibilitychange", onVisible);

    return () => {
      navigator.serviceWorker.removeEventListener("controllerchange", onControllerChange);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, []);

  const update = useCallback(() => {
    // Nhường chỗ cho bản mới; `controllerchange` ở trên lo việc tải lại.
    waiting?.postMessage("SKIP_WAITING");
  }, [waiting]);

  if (!waiting || dismissed) return null;

  return (
    <div
      role="status"
      className="fixed inset-x-0 bottom-0 z-[70] flex justify-center p-3 pb-[max(0.75rem,env(safe-area-inset-bottom))]"
    >
      <div className="flex items-center gap-2 rounded-full bg-popover py-1.5 pr-1.5 pl-4 text-sm text-popover-foreground ring-1 ring-foreground/10 shadow-lg">
        <span>Đã có bản cập nhật</span>
        <Button size="sm" className="h-8 rounded-full" onClick={update}>
          <RefreshCw className="size-3.5" aria-hidden />
          Cập nhật
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="size-8 rounded-full"
          onClick={() => setDismissed(true)}
          aria-label="Để sau"
        >
          <X className="size-4" aria-hidden />
        </Button>
      </div>
    </div>
  );
}
