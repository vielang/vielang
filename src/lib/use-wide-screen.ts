"use client";

import { useCallback, useSyncExternalStore } from "react";

function getServerSnapshot(): boolean {
  return false;
}

/**
 * true nếu viewport đủ rộng để xem 2 trang cạnh nhau mà chữ còn đọc được.
 * Chỉ dùng để BẬT/ẨN tuỳ chọn "xem 2 trang" — không tự đổi lựa chọn đã lưu
 * của người dùng, chỉ tạm áp dụng chế độ 1 trang khi màn hình đang hẹp.
 *
 * Dùng useSyncExternalStore thay vì effect + setState: đây đúng là trường
 * hợp "theo dõi 1 nguồn dữ liệu ngoài React" (matchMedia) mà API này sinh ra
 * để xử lý — vừa tránh warning set-state-in-effect, vừa không có nhịp render
 * "false" ban đầu rồi mới cập nhật đúng như cách dùng effect cũ.
 */
export function useIsWideScreen(minWidth = 1024): boolean {
  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      const mq = window.matchMedia(`(min-width: ${minWidth}px)`);
      mq.addEventListener("change", onStoreChange);
      return () => mq.removeEventListener("change", onStoreChange);
    },
    [minWidth]
  );
  const getSnapshot = useCallback(
    () => window.matchMedia(`(min-width: ${minWidth}px)`).matches,
    [minWidth]
  );

  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}
