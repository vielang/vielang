"use client";

import { create } from "zustand";

/** Giới hạn phóng to của trang đọc — dùng chung cho khung ảnh và thanh phóng to. */
export const MIN_ZOOM = 1;
export const MAX_ZOOM = 4;
/** Mỗi nấc của nút ＋/－, thanh kéo và phím tắt: 25%. */
export const ZOOM_STEP = 0.25;

/**
 * Bấm đúp khi đang ở 100% thì phóng lên bao nhiêu (cộng thêm vào mức 1).
 *
 * Máy tính: +0,5 → 150%. Trang đang vừa khít chiều cao màn hình, chữ chỉ hơi
 * nhỏ; 150% là đọc rõ mà vẫn thấy gần hết bề ngang trang. Bản cũ nhảy thẳng
 * lên 280%, phải kéo qua kéo lại mới đọc hết một dòng.
 *
 * Điện thoại: +1,5 → 250%. Trang chỉ rộng ~390px, chữ rất nhỏ — 150% vẫn
 * chưa đọc được.
 */
export const DOUBLE_TAP_STEP_WIDE = 0.5;
export const DOUBLE_TAP_STEP_NARROW = 1.5;

/**
 * Mức phóng HIỆN TẠI của trang đang đọc, chỉ để hiển thị (thanh phóng to).
 *
 * Tách thành store riêng thay vì state của trang đọc: lúc chụm hai ngón mức
 * phóng đổi từng khung hình, để trong state của `ReaderView` là cả trang đọc
 * vẽ lại 60 lần/giây. Ở đây chỉ thanh phóng to — thứ duy nhất cần con số —
 * vẽ lại. Không lưu xuống đâu: lật trang là về 100% như cũ.
 */
interface ZoomState {
  scale: number;
  setScale: (scale: number) => void;
}

export const useZoomStore = create<ZoomState>()((set) => ({
  scale: 1,
  setScale: (scale) => set({ scale }),
}));

/** Mức kế tiếp khi bấm ＋/－, bám vào lưới 25% (125, 150, …) chứ không cộng dồn lệch. */
export function nextZoom(scale: number, direction: 1 | -1): number {
  const snapped =
    direction > 0
      ? Math.floor(scale / ZOOM_STEP + 1e-6) * ZOOM_STEP + ZOOM_STEP
      : Math.ceil(scale / ZOOM_STEP - 1e-6) * ZOOM_STEP - ZOOM_STEP;
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Math.round(snapped * 100) / 100));
}
