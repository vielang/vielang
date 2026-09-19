"use client";

import { useEffect } from "react";
import { create } from "zustand";
import { createJSONStorage, persist, type StateStorage } from "zustand/middleware";

/** Bút mực, bút dạ quang, hay tẩy. Tẩy xoá nguyên nét chứ không gặm từng đoạn. */
export type AnnotationTool = "pen" | "highlighter" | "eraser";

/**
 * Một nét vẽ trên ảnh trang sách.
 *
 * Mọi con số đều là TỈ LỆ so với khung ảnh thật (không phải pixel): `x` theo
 * chiều rộng, `y` theo chiều cao, đều trong [0,1]; `width` theo chiều rộng
 * trang. Nhờ vậy nét vẽ bám đúng chỗ trên chữ dù người dùng đổi cỡ cửa sổ,
 * xoay máy, phóng to hay chuyển qua lại 1 trang/2 trang — cùng cách vùng dịch
 * lưu toạ độ (xem `page-translation`).
 */
export interface Stroke {
  id: string;
  tool: "pen" | "highlighter";
  color: string;
  width: number;
  /** [x0,y0,x1,y1,…] — mảng phẳng cho gọn khi lưu xuống localStorage. */
  points: number[];
  /**
   * Lực bút tại từng điểm (0–1), một phần tử cho mỗi điểm trong `points`.
   *
   * CHỈ có khi vẽ bằng bút cảm ứng thật (`pointerType === "pen"`). Vẽ bằng
   * chuột hay ngón tay thì trình duyệt báo lực giả (thường đúng 0.5), lưu
   * lại là tốn gấp đôi chỗ mà không thêm thông tin gì — để trống rồi suy bề
   * dày theo tốc độ viết, xem `penOutlinePath`.
   */
  pressures?: number[];
}

/** Bút: mảnh / vừa / đậm, theo phần chiều rộng trang. */
export const PEN_WIDTHS = [0.0022, 0.0038, 0.0065];
/** Bút dạ quang dày hơn hẳn — đủ phủ kín một dòng chữ Hàn cỡ thường. */
export const HIGHLIGHTER_WIDTHS = [0.014, 0.024, 0.04];

export const PEN_COLORS = ["#ef4444", "#2563eb", "#16a34a", "#111827"];
export const HIGHLIGHTER_COLORS = ["#fde047", "#86efac", "#93c5fd", "#fda4af"];

/** Bút dạ quang vẽ đè lên ảnh scan nên phải trong suốt, nếu không mất chữ. */
export const HIGHLIGHTER_OPACITY = 0.38;

interface AnnotationState {
  /** key = `${bookId}:${page}` — xem `annotationKey()`. */
  strokes: Record<string, Stroke[]>;
  hasHydrated: boolean;
  /** localStorage từ chối lần ghi gần nhất — xem `safeStorage`. */
  quotaExceeded: boolean;

  /** Đang bật chế độ vẽ lên trang sách (không lưu — mỗi phiên tự bật lại). */
  active: boolean;
  /**
   * Đang tạm giấu hết nét để nhìn ảnh trang sạch. KHÔNG đụng tới dữ liệu —
   * đây là "hé xem bên dưới", đối lập với `clearPage` vốn xoá thật.
   */
  peeking: boolean;
  tool: AnnotationTool;
  penColor: string;
  highlighterColor: string;
  /** Chỉ số 0/1/2 trong PEN_WIDTHS và HIGHLIGHTER_WIDTHS, dùng chung cho 2 bút. */
  size: number;
  /**
   * Trang người dùng vừa chạm bút vào. Ở chế độ 2 trang, "hoàn tác" và "xoá
   * hết" phải biết nói về trang nào — lấy trang vừa chạm là cách đoán đúng ý
   * nhất, và thanh công cụ ghi rõ số trang lên nút nên không đoán mò.
   */
  lastPage: number | null;

  setHasHydrated: (v: boolean) => void;
  setActive: (v: boolean) => void;
  setPeeking: (v: boolean) => void;
  setTool: (t: AnnotationTool) => void;
  setColor: (c: string) => void;
  setSize: (i: number) => void;
  touchPage: (page: number) => void;

  addStroke: (bookId: string, page: number, stroke: Stroke) => void;
  /** Xoá theo id — tẩy gom cả cụm nét chạm phải rồi xoá một lượt. */
  eraseStrokes: (bookId: string, page: number, ids: string[]) => void;
  undo: (bookId: string, page: number) => void;
  clearPage: (bookId: string, page: number) => void;
}

export function annotationKey(bookId: string, page: number): string {
  return `${bookId}:${page}`;
}

export const ANNOTATION_STORAGE_KEY = "kiip-annotations-v1";

/** Cùng lý do với `draw-store`: hết chỗ thì báo, đừng ném lỗi giữa lúc đang vẽ. */
const safeStorage: StateStorage = {
  getItem: (name) => {
    try {
      return localStorage.getItem(name);
    } catch {
      return null;
    }
  },
  setItem: (name, value) => {
    try {
      localStorage.setItem(name, value);
      markQuota(false);
    } catch {
      markQuota(true);
    }
  },
  removeItem: (name) => {
    try {
      localStorage.removeItem(name);
    } catch {
      /* localStorage bị chặn — không còn gì để dọn */
    }
  },
};

/** So sánh trước khi `set` để không thành vòng lặp ghi/lỗi/ghi — xem `draw-store`. */
function markQuota(exceeded: boolean): void {
  if (useAnnotationStore.getState().quotaExceeded === exceeded) return;
  useAnnotationStore.setState({ quotaExceeded: exceeded });
}

function withStrokes(
  state: AnnotationState,
  key: string,
  next: Stroke[]
): Pick<AnnotationState, "strokes"> | AnnotationState {
  if (next.length === 0) {
    if (!(key in state.strokes)) return state;
    const strokes = { ...state.strokes };
    delete strokes[key];
    return { strokes };
  }
  return { strokes: { ...state.strokes, [key]: next } };
}

/**
 * Nét vẽ tay đè lên ảnh trang sách — khoanh từ mới, gạch chân ngữ pháp, nối
 * mũi tên giữa hai vế câu. Khác hẳn tab "Vẽ" trong bài giảng (xem
 * `draw-store`): bên kia là khung trắng để vẽ sơ đồ, bên này bám dính vào
 * đúng con chữ trên ảnh scan.
 *
 * Vẫn là localStorage, không backend — giống tiến độ và bài giảng. Dữ liệu
 * nhẹ hơn bảng vẽ nhiều (chỉ là danh sách toạ độ, không có ảnh), nhưng dùng
 * chung cách bọc storage an toàn và `skipHydration` cho nhất quán: chấm báo
 * "trang này có nét vẽ" nằm trên thanh công cụ vốn được server render.
 */
export const useAnnotationStore = create<AnnotationState>()(
  persist(
    (set) => ({
      strokes: {},
      hasHydrated: false,
      quotaExceeded: false,

      active: false,
      peeking: false,
      tool: "pen",
      penColor: PEN_COLORS[0],
      highlighterColor: HIGHLIGHTER_COLORS[0],
      size: 1,
      lastPage: null,

      setHasHydrated: (v) => set({ hasHydrated: v }),
      // Tắt chế độ vẽ thì bỏ luôn trạng thái hé xem — nếu không, lần sau bật
      // bút lên là trang trống trơn mà không rõ vì sao.
      setActive: (active) => set(active ? { active } : { active, peeking: false }),
      setPeeking: (peeking) => set({ peeking }),
      setTool: (tool) => set({ tool }),
      setColor: (c) =>
        set((state) =>
          state.tool === "highlighter" ? { highlighterColor: c } : { penColor: c }
        ),
      setSize: (size) => set({ size }),
      touchPage: (lastPage) => set({ lastPage }),

      addStroke: (bookId, page, stroke) =>
        set((state) => {
          const key = annotationKey(bookId, page);
          return { strokes: { ...state.strokes, [key]: [...(state.strokes[key] ?? []), stroke] } };
        }),

      eraseStrokes: (bookId, page, ids) =>
        set((state) => {
          const key = annotationKey(bookId, page);
          const current = state.strokes[key];
          if (!current) return state;
          const gone = new Set(ids);
          const next = current.filter((s) => !gone.has(s.id));
          if (next.length === current.length) return state;
          return withStrokes(state, key, next);
        }),

      undo: (bookId, page) =>
        set((state) => {
          const key = annotationKey(bookId, page);
          const current = state.strokes[key];
          if (!current || current.length === 0) return state;
          return withStrokes(state, key, current.slice(0, -1));
        }),

      clearPage: (bookId, page) =>
        set((state) => withStrokes(state, annotationKey(bookId, page), [])),
    }),
    {
      name: ANNOTATION_STORAGE_KEY,
      storage: createJSONStorage(() => safeStorage),
      // `active` cố tình không lưu: mở sách ra mà rơi ngay vào chế độ vẽ thì
      // chạm vào trang là vẽ bậy lên chứ không lật trang như người ta chờ đợi.
      partialize: (state) => ({
        strokes: state.strokes,
        tool: state.tool,
        penColor: state.penColor,
        highlighterColor: state.highlighterColor,
        size: state.size,
      }),
      skipHydration: true,
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

/** Nạp nét vẽ từ localStorage sau lần render đầu — gọi ở `ReaderView`. */
export function useAnnotationHydration(): boolean {
  const hasHydrated = useAnnotationStore((s) => s.hasHydrated);
  useEffect(() => {
    if (!useAnnotationStore.getState().hasHydrated)
      void useAnnotationStore.persist.rehydrate();
  }, []);
  return hasHydrated;
}

/** Bề rộng nét của công cụ hiện tại, theo phần chiều rộng trang. */
export function activeWidth(tool: AnnotationTool, size: number): number {
  const widths = tool === "highlighter" ? HIGHLIGHTER_WIDTHS : PEN_WIDTHS;
  return widths[size] ?? widths[1];
}

/** Bảng màu tương ứng công cụ — tẩy dùng chung bảng của bút cho khỏi nhảy layout. */
export function paletteFor(tool: AnnotationTool): string[] {
  return tool === "highlighter" ? HIGHLIGHTER_COLORS : PEN_COLORS;
}

/**
 * Trong các trang đang hiện, có trang nào đã được vẽ lên chưa — cho chấm báo
 * trên nút bút. Selector trả về boolean nên `pages` là mảng mới mỗi lượt
 * render cũng không làm component render lại thừa.
 */
export function useHasAnnotations(bookId: string, pages: number[]): boolean {
  return useAnnotationStore(
    (s) =>
      s.hasHydrated &&
      pages.some((p) => (s.strokes[annotationKey(bookId, p)]?.length ?? 0) > 0)
  );
}
