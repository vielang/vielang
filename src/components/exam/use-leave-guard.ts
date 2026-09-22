"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";

/** Dấu của mục lịch sử "chặn Back" (xem bên dưới). */
const GUARD = "__examLeaveGuard";

/**
 * Hỏi lại trước khi rời trang lúc đang thi thử — bài làm đã lưu nhưng đồng
 * hồ vẫn chạy, bấm nhầm một link là mất vài phút mà không biết.
 *
 * - Bấm link trong web: chặn lại, trả về `pending` để trang hiện hộp thoại
 *   của app; "Vẫn thoát" thì đi tiếp tới đúng link đó.
 * - Nút Back: đặt thêm một mục lịch sử trùng trang hiện tại; Back chỉ lùi về
 *   mục thật → đặt lại mục chặn và hỏi. Mục chặn chép nguyên `history.state`
 *   của Next (thiếu nó Next sẽ tải lại cả trang).
 * - Đóng tab / tải lại / gõ địa chỉ khác: hộp xác nhận có sẵn của trình
 *   duyệt (không tự viết nội dung được).
 *
 * Link cùng trang (neo "#q-12" của phiếu trả lời), link mở tab mới, bấm kèm
 * Ctrl/⌘ thì để yên.
 */
export function useLeaveGuard(active: boolean, fallbackHref: string) {
  const router = useRouter();
  const [pending, setPending] = useState<string | null>(null);
  const bypass = useRef(false);

  useEffect(() => {
    if (!active) return;
    bypass.current = false;

    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      if (bypass.current) return;
      e.preventDefault();
      e.returnValue = "";
    };

    const onClick = (e: MouseEvent) => {
      if (bypass.current || e.defaultPrevented || e.button !== 0) return;
      if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      const a = (e.target as Element | null)?.closest?.("a[href]") as HTMLAnchorElement | null;
      if (!a || (a.target && a.target !== "_self") || a.hasAttribute("download")) return;
      const url = new URL(a.href, window.location.href);
      if (url.origin !== window.location.origin) return; // ra ngoài web: để hộp của trình duyệt lo
      if (url.pathname === window.location.pathname && url.search === window.location.search) return;
      // Chặn trước khi React/Next nhận cú bấm (nghe ở pha capture của window).
      e.preventDefault();
      e.stopImmediatePropagation();
      setPending(url.pathname + url.search + url.hash);
    };

    const pushGuard = () => window.history.pushState({ ...window.history.state, [GUARD]: true }, "", window.location.href);
    if (!window.history.state?.[GUARD]) pushGuard();
    const onPopState = () => {
      if (bypass.current || window.history.state?.[GUARD]) return;
      // Nhảy tới neo trong trang (bấm số câu ở phiếu trả lời) cũng bắn
      // popstate, nhưng mục lịch sử đó không có state của Next — không phải Back.
      if (window.history.state === null) return;
      pushGuard();
      setPending("back");
    };

    window.addEventListener("beforeunload", onBeforeUnload);
    window.addEventListener("click", onClick, true);
    window.addEventListener("popstate", onPopState);
    return () => {
      window.removeEventListener("beforeunload", onBeforeUnload);
      window.removeEventListener("click", onClick, true);
      window.removeEventListener("popstate", onPopState);
    };
  }, [active]);

  const stay = () => setPending(null);

  const leave = () => {
    const to = pending;
    setPending(null);
    bypass.current = true;
    if (to !== "back") {
      if (to) router.push(to);
      return;
    }
    // Lùi qua mục chặn và mục của trang này. Mở thẳng trang thi (không có
    // trang trước) thì lùi không được — về trang đề.
    const here = window.location.href;
    window.history.go(-2);
    window.setTimeout(() => {
      if (window.location.href === here) router.push(fallbackHref);
    }, 400);
  };

  return { pending: pending !== null, stay, leave };
}
