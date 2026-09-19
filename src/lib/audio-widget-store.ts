"use client";

import { create } from "zustand";

interface Position {
  x: number;
  y: number;
}

/**
 * Đường kính nút tròn audio lúc thu nhỏ, và khoảng chừa dưới đáy màn hình.
 *
 * Để ở đây chứ không nằm trong chính `audio-widget.tsx` vì thanh công cụ vẽ
 * cũng cần biết: nó neo mặc định ngay PHÍA TRÊN nút audio, mà hai widget
 * chồng lên nhau thì cái nào cũng khó bấm.
 */
export const AUDIO_WIDGET_SIZE = 48;
export const AUDIO_WIDGET_BOTTOM_OFFSET = 84;

interface AudioWidgetState {
  /** Vị trí panel lúc mở rộng. null = chưa mở lần nào (sẽ canh giữa). */
  position: Position | null;
  /**
   * Vị trí nút tròn lúc thu nhỏ — tách riêng khỏi `position` để mỗi trạng
   * thái giữ chỗ của nó: mở ra thì panel canh giữa cho dễ bấm, thu lại thì
   * nút tròn về đúng góc người dùng đã đặt, không nằm chình ình giữa trang.
   * null = chưa từng kéo, dùng góc dưới phải.
   */
  collapsedPosition: Position | null;
  collapsed: boolean;
  activeType: string | null;
  /** Ở chế độ xem 2 trang: audio của trang trái hay phải đang được chọn. */
  side: "left" | "right";
  setPosition: (p: Position) => void;
  setCollapsedPosition: (p: Position) => void;
  setCollapsed: (v: boolean) => void;
  setActiveType: (t: string) => void;
  setSide: (s: "left" | "right") => void;
}

/**
 * Vị trí + trạng thái thu/phóng của widget audio nổi trong Reader.
 *
 * Cố ý KHÔNG dùng `persist` (localStorage) — chỉ cần "sống sót" qua các
 * lần `ReaderView` remount khi chuyển trang (page.tsx render
 * `<ReaderView key={page}/>`), nhờ Zustand store là module-scope, độc
 * lập vòng đời component. Vị trí/trạng thái tự reset khi tải lại hẳn
 * trang trình duyệt — hợp lý cho 1 tuỳ chỉnh UI tạm thời trong phiên đọc.
 */
export const useAudioWidgetStore = create<AudioWidgetState>((set) => ({
  position: null,
  collapsedPosition: null,
  collapsed: true,
  activeType: null,
  side: "left",
  setPosition: (position) => set({ position }),
  setCollapsedPosition: (collapsedPosition) => set({ collapsedPosition }),
  setCollapsed: (collapsed) => set({ collapsed }),
  setActiveType: (activeType) => set({ activeType }),
  setSide: (side) => set({ side }),
}));
