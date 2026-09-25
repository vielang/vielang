"use client";

import { create } from "zustand";

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

/**
 * - `closed`    — chỉ còn nút bài giảng trên thanh công cụ.
 * - `floating`  — panel nổi, kéo thả/đổi cỡ được, nằm đè lên trang sách.
 * - `fullscreen`— sheet chiếm gần hết màn hình (hợp với điện thoại).
 * - `popped`    — đã bật ra cửa sổ trình duyệt riêng; cửa sổ chính không
 *                 hiện panel nữa để không có 2 editor sửa cùng 1 note.
 */
export type NoteMode = "closed" | "floating" | "fullscreen" | "popped";

/** Tab trong panel: bài giảng (văn xuôi), luyện tập (câu hỏi) hay bảng vẽ. */
export type NoteTab = "note" | "quiz" | "draw";

/** Ở chế độ xem 2 trang: bài giảng/bài tập đang xem của trang trái hay phải. */
export type NoteSide = "left" | "right";

interface NoteWidgetState {
  mode: NoteMode;
  /** null = chưa từng kéo, dùng vị trí mặc định (mép TRÁI, dưới thanh trên — xem `notePanelAnchor`). */
  position: Position | null;
  /** null = chưa từng đổi cỡ, dùng kích thước mặc định theo màn hình. */
  size: Size | null;
  /** Đang soạn thảo (thay vì chỉ đọc) — giữ nguyên khi đổi mode/lật trang. */
  editing: boolean;
  /** Tab đang xem trong panel. Giữ nguyên khi lật trang: đang luyện tập mà
   *  lật trang thì vẫn ở tab luyện tập, không bị đá về bài giảng. */
  tab: NoteTab;
  /** Trang trái/phải đang xem khi ở chế độ 2 trang — giữ nguyên khi lật
   *  spread, giống `tab`, cho tới khi người dùng tự đổi. */
  side: NoteSide;
  setMode: (m: NoteMode) => void;
  setTab: (t: NoteTab) => void;
  setSide: (s: NoteSide) => void;
  setPosition: (p: Position) => void;
  setSize: (s: Size) => void;
  setEditing: (v: boolean) => void;
}

/**
 * Trạng thái panel bài giảng trong Reader — cùng lý do và cùng kiểu với
 * `audio-widget-store`: KHÔNG dùng `persist`, chỉ cần sống sót qua các lần
 * `ReaderView` remount khi chuyển trang (page.tsx render
 * `<ReaderView key={page}/>`), nhờ Zustand store là module-scope.
 *
 * Nhờ vậy panel không bị đóng lại mỗi lần lật trang: mở 1 lần rồi lật trang,
 * panel đứng yên và tự đổi sang bài giảng của trang mới — khác hẳn Sheet cũ
 * (state cục bộ nên đóng theo mỗi lần remount).
 */
export const useNoteWidgetStore = create<NoteWidgetState>((set) => ({
  mode: "closed",
  position: null,
  size: null,
  editing: false,
  tab: "note",
  side: "left",
  setMode: (mode) => set({ mode }),
  setTab: (tab) => set({ tab }),
  setSide: (side) => set({ side }),
  setPosition: (position) => set({ position }),
  setSize: (size) => set({ size }),
  setEditing: (editing) => set({ editing }),
}));
