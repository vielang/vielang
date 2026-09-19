"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useTheme } from "next-themes";
// Phải đứng trước import Excalidraw bên dưới — xem `excalidraw-assets`.
import "@/lib/excalidraw-assets";
import { Excalidraw, MainMenu, WelcomeScreen, getSceneVersion } from "@excalidraw/excalidraw";
import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import type {
  AppState,
  BinaryFiles,
  ExcalidrawInitialDataState,
  UIOptions,
} from "@excalidraw/excalidraw/types";
import { AlertTriangle, Check, Loader2 } from "lucide-react";
import { drawKey, useDrawStore } from "@/lib/draw-store";

// Excalidraw phát hành CSS riêng — theo tài liệu Next.js, stylesheet của gói
// bên ngoài import được ở bất kỳ component nào trong app/. File này lại chỉ
// được nạp động (xem note-panel) nên CSS cũng chỉ tải khi mở tab Vẽ.
import "@excalidraw/excalidraw/index.css";

/** Cùng nhịp với `note-editor`: ghi sau khi ngừng thao tác, không có nút lưu. */
const AUTOSAVE_DELAY_MS = 800;

/**
 * Bớt các mục không có chỗ dùng ở đây, giữ canvas gọn như FigJam:
 * - `loadScene`/`saveToActiveFile`: app không có khái niệm "tệp .excalidraw".
 * - `toggleTheme`: sáng/tối bám theo chủ đề của app (xem `theme` bên dưới),
 *   để Excalidraw tự đổi riêng thì panel lệch màu với phần còn lại.
 * - `export`: hộp thoại chia sẻ/xuất tệp; `saveAsImage` (xuất PNG/SVG) là
 *   thứ người học thật sự cần nên vẫn giữ.
 */
const UI_OPTIONS: Partial<UIOptions> = {
  canvasActions: {
    changeViewBackgroundColor: true,
    clearCanvas: true,
    export: false,
    loadScene: false,
    saveToActiveFile: false,
    saveAsImage: true,
    toggleTheme: false,
  },
  tools: { image: true },
};

/**
 * Bảng vẽ tự do cho 1 trang sách — tab "Vẽ" trong panel bài giảng.
 *
 * Mỗi trang một bảng riêng, lưu thẳng vào localStorage (xem `draw-store`).
 * Component được `NotePanel` nạp động với `ssr: false` và gắn `key` theo
 * `bookId:page`, nên: (1) Excalidraw — vốn cần `window` và nặng vài trăm KB
 * — không lọt vào bundle của trang đọc, (2) lật trang là remount, nạp lại
 * đúng bản vẽ của trang mới qua `initialData` (Excalidraw chỉ đọc prop này
 * lúc khởi tạo, không theo dõi về sau).
 */
export function NoteDraw({ bookId, page }: { bookId: string; page: number }) {
  const { resolvedTheme } = useTheme();
  const saveDrawing = useDrawStore((s) => s.saveDrawing);
  const quotaExceeded = useDrawStore((s) => s.quotaExceeded);

  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");

  // Ảnh chụp bản vẽ lúc mở — cố tình KHÔNG reactive: store đổi vì chính
  // người dùng đang vẽ, để nó chảy ngược vào `initialData` là canvas nhảy.
  const [initialData] = useState<ExcalidrawInitialDataState>(() => {
    const saved = useDrawStore.getState().drawings[drawKey(bookId, page)];
    return {
      elements: saved?.elements ?? [],
      files: saved?.files ?? {},
      appState: saved?.background ? { viewBackgroundColor: saved.background } : {},
      // Bản vẽ cũ có thể nằm ngoài khung nhìn mặc định — kéo về giữa thay vì
      // mở ra một canvas trắng trông như mất bài.
      scrollToContent: true,
    };
  });

  const savedRef = useRef({
    version: getSceneVersion(initialData.elements ?? []),
    background: initialData.appState?.viewBackgroundColor ?? null,
  });
  const pendingRef = useRef<{
    elements: readonly ExcalidrawElement[];
    files: BinaryFiles;
    background: string | null;
    version: number;
  } | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    const pending = pendingRef.current;
    if (!pending) return;
    pendingRef.current = null;
    saveDrawing(bookId, page, pending.elements, pending.files, pending.background);
    savedRef.current = { version: pending.version, background: pending.background };
    setStatus("saved");
  }, [bookId, page, saveDrawing]);

  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[], appState: AppState, files: BinaryFiles) => {
      // Excalidraw gọi `onChange` cả khi chỉ di chuột / đổi vùng chọn. So
      // phiên bản scene để chỉ ghi khi hình vẽ thật sự đổi, nếu không là
      // ghi localStorage liên tục suốt lúc rê chuột.
      const version = getSceneVersion(elements);
      const background = appState.viewBackgroundColor ?? null;
      if (version === savedRef.current.version && background === savedRef.current.background)
        return;

      pendingRef.current = { elements, files, background, version };
      setStatus("saving");
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(flush, AUTOSAVE_DELAY_MS);
    },
    [flush]
  );

  // Đóng panel / lật trang ngay sau nét vẽ cuối: ghi nốt phần còn treo trong
  // debounce thay vì để mất — giống `note-editor`.
  useEffect(() => flush, [flush]);

  // Trang đọc nghe phím trên `window` (mũi tên lật trang, `b` đánh dấu, `n`
  // mở bài giảng...) và chỉ bỏ qua khi con trỏ nằm trong input/contentEditable.
  // Canvas của Excalidraw không phải hai thứ đó, nên không chặn ở đây thì gõ
  // chữ vào bản vẽ là sách lật trang theo. Chặn ở pha nổi bọt: handler bên
  // trong Excalidraw đã chạy xong, chỉ cắt đường lên `window`.
  const shellRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    const el = shellRef.current;
    if (!el) return;
    const stop = (e: Event) => e.stopPropagation();
    el.addEventListener("keydown", stop);
    el.addEventListener("keyup", stop);
    return () => {
      el.removeEventListener("keydown", stop);
      el.removeEventListener("keyup", stop);
    };
  }, []);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* `isolate`: Excalidraw dựng khá nhiều lớp chồng (thanh công cụ, bảng
          màu, hộp thoại) với z-index lớn — nhốt chúng vào một stacking
          context để không leo lên trên panel/sheet đang chứa nó. */}
      <div ref={shellRef} className="relative min-h-0 flex-1 isolate">
        <Excalidraw
          initialData={initialData}
          onChange={handleChange}
          theme={resolvedTheme === "dark" ? "dark" : "light"}
          langCode="vi-VN"
          UIOptions={UI_OPTIONS}
          // Phím tắt chỉ ăn khi canvas đang được focus — trang đọc vẫn giữ
          // được phím lật trang lúc người dùng không ở trong bản vẽ.
          handleKeyboardGlobally={false}
          autoFocus={false}
        >
          <MainMenu>
            <MainMenu.DefaultItems.SaveAsImage />
            <MainMenu.DefaultItems.ClearCanvas />
            <MainMenu.Separator />
            <MainMenu.DefaultItems.ChangeCanvasBackground />
          </MainMenu>
          <WelcomeScreen>
            <WelcomeScreen.Center>
              <WelcomeScreen.Center.Heading>
                Vẽ sơ đồ, gạch ý, ghi chú cho trang này
              </WelcomeScreen.Center.Heading>
            </WelcomeScreen.Center>
            <WelcomeScreen.Hints.ToolbarHint>
              <p>Chọn công cụ rồi vẽ lên khung</p>
            </WelcomeScreen.Hints.ToolbarHint>
            <WelcomeScreen.Hints.MenuHint />
          </WelcomeScreen>
        </Excalidraw>
      </div>

      <div
        className="flex shrink-0 items-center gap-1.5 border-t border-border px-3 py-1.5 text-xs text-muted-foreground"
        aria-live="polite"
      >
        {quotaExceeded ? (
          <span className="flex items-center gap-1.5 text-destructive">
            <AlertTriangle className="size-3.5" aria-hidden />
            Chưa lưu được bản vẽ — bộ nhớ trình duyệt bị chặn hoặc đã đầy. Thử xoá
            bớt ảnh trong bản vẽ.
          </span>
        ) : status === "saving" ? (
          <>
            <Loader2 className="size-3.5 animate-spin" aria-hidden />
            Đang lưu…
          </>
        ) : status === "saved" ? (
          <>
            <Check className="size-3.5" aria-hidden />
            Đã lưu trên thiết bị này
          </>
        ) : (
          "Bản vẽ tự lưu trên thiết bị này."
        )}
      </div>
    </div>
  );
}
