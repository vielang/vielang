"use client";

import { useEffect } from "react";

/**
 * Đăng ký service worker (xem `public/sw.js`).
 *
 * CHỈ chạy ở bản dựng thật. Ở `next dev`, service worker sẽ cache nhầm chunk
 * của HMR và gây ra những lỗi "sao sửa code mà không thấy đổi" rất khó truy.
 *
 * Lỗi đăng ký thì nuốt: mất offline là tiếc, còn chặn cả app vì không đăng
 * ký được service worker thì vô lý. Một số trình duyệt cũng chặn hẳn service
 * worker ở chế độ riêng tư.
 */
export function ServiceWorker() {
  useEffect(() => {
    if (process.env.NODE_ENV !== "production") return;
    if (!("serviceWorker" in navigator)) return;
    navigator.serviceWorker.register("/sw.js").catch(() => {});
  }, []);

  return null;
}
