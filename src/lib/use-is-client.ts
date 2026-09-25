"use client";

import { useSyncExternalStore } from "react";

/** Không có gì để theo dõi — giá trị chỉ khác nhau giữa server và trình duyệt. */
export const NO_SUBSCRIBE = () => () => {};

/**
 * `false` khi render ở server (và lượt hydrate đầu), `true` khi đã chạy ở
 * trình duyệt.
 *
 * Dùng cho phần hiển thị phụ thuộc dữ liệu chỉ trình duyệt có (localStorage,
 * bài làm, tiến độ…): đọc thẳng lúc render thì HTML server và lượt vẽ đầu ở
 * trình duyệt lệch nhau, React báo lỗi hydration.
 *
 * Dùng `useSyncExternalStore` thay vì effect + setState: không có lượt vẽ
 * "false" thừa rồi mới đổi, và không vướng cảnh báo set-state-in-effect.
 * Trước đây đoạn này được chép tay ở 12 component.
 */
export function useIsClient(): boolean {
  return useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
}
