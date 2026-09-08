"use client";

import { create } from "zustand";

interface Position {
  x: number;
  y: number;
}

interface AudioWidgetState {
  /** null = chưa từng kéo, dùng vị trí mặc định (góc dưới phải). */
  position: Position | null;
  collapsed: boolean;
  activeType: string | null;
  setPosition: (p: Position) => void;
  setCollapsed: (v: boolean) => void;
  setActiveType: (t: string) => void;
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
  collapsed: true,
  activeType: null,
  setPosition: (position) => set({ position }),
  setCollapsed: (collapsed) => set({ collapsed }),
  setActiveType: (activeType) => set({ activeType }),
}));
