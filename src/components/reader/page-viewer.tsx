"use client";

import Image from "next/image";
import {
  forwardRef,
  useCallback,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import {
  TransformWrapper,
  TransformComponent,
  type ReactZoomPanPinchContentRef,
} from "react-zoom-pan-pinch";
import { getPageUrl, PAGE_ASPECT_RATIO } from "@/lib/books";
import { getPageTranslations } from "@/lib/page-translation";
import { TranslationOverlay } from "@/components/reader/translation-overlay";

export interface PageViewerHandle {
  resetZoom: () => void;
}

interface PageViewerProps {
  bookId: string;
  page: number;
  onTap: () => void;
  onSwipePrev: () => void;
  onSwipeNext: () => void;
}

export const PageViewer = forwardRef<PageViewerHandle, PageViewerProps>(function PageViewer(
  { bookId, page, onTap, onSwipePrev, onSwipeNext },
  ref
) {
  const transformRef = useRef<ReactZoomPanPinchContentRef>(null);
  const scaleRef = useRef(1);
  const regions = getPageTranslations(bookId, page);

  useImperativeHandle(ref, () => ({
    resetZoom: () => transformRef.current?.resetTransform(),
  }));

  /**
   * Khung ẢNH THẬT bên trong khung chứa. Ảnh hiển thị `object-contain` nên
   * luôn có viền trống ở hai bên (hoặc trên dưới) tuỳ tỉ lệ màn hình — vùng
   * bấm xem bản dịch lưu toạ độ theo tỉ lệ của ảnh, đặt theo khung chứa là
   * lệch. Nên phải tự tính.
   *
   * Dùng offsetWidth/offsetHeight chứ KHÔNG dùng getBoundingClientRect: rect
   * đã nhân với transform của react-zoom-pan-pinch, đo bằng nó thì cứ zoom là
   * kích thước lại đổi và tính toán chạy vòng vo.
   */
  const boxRef = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState<{ width: number; height: number } | null>(null);

  useLayoutEffect(() => {
    const el = boxRef.current;
    if (!el) return;

    function measure(target: HTMLElement) {
      const width = Math.min(
        target.offsetWidth,
        target.offsetHeight * PAGE_ASPECT_RATIO
      );
      setBox({ width, height: width / PAGE_ASPECT_RATIO });
    }

    measure(el);
    const observer = new ResizeObserver(() => measure(el));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Theo dõi cử chỉ 1 ngón để phân biệt: tap (bật/tắt toolbar), vuốt ngang
  // (chuyển trang, chỉ khi chưa zoom), hay pinch (2 ngón — bỏ qua, để thư
  // viện zoom xử lý). Chỉ *đọc* sự kiện, không preventDefault/stopPropagation
  // nên không phá cử chỉ pinch/pan gốc của react-zoom-pan-pinch.
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const gesture = useRef<{
    id: number;
    startX: number;
    startY: number;
    cancelled: boolean;
    onRegion: boolean;
  } | null>(null);

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.current.size === 1) {
      gesture.current = {
        id: e.pointerId,
        startX: e.clientX,
        startY: e.clientY,
        cancelled: false,
        // Chạm bắt đầu trên vùng dịch thì để nút đó tự xử lý: nếu vẫn tính là
        // tap thì vừa mở bản dịch vừa ẩn thanh công cụ.
        onRegion: Boolean(
          (e.target as HTMLElement).closest("[data-translate-region]")
        ),
      };
    } else if (gesture.current) {
      gesture.current.cancelled = true;
    }
  }, []);

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      const g = gesture.current;
      pointers.current.delete(e.pointerId);
      if (!g || g.id !== e.pointerId) return;
      gesture.current = null;
      if (g.cancelled) return;

      const dx = e.clientX - g.startX;
      const dy = e.clientY - g.startY;
      const absDx = Math.abs(dx);
      const absDy = Math.abs(dy);

      if (absDx < 10 && absDy < 10) {
        if (!g.onRegion) onTap();
        return;
      }
      if (scaleRef.current <= 1.02 && absDx > 60 && absDx > absDy * 1.5) {
        if (dx < 0) onSwipeNext();
        else onSwipePrev();
      }
    },
    [onTap, onSwipeNext, onSwipePrev]
  );

  const onPointerCancel = useCallback((e: React.PointerEvent) => {
    pointers.current.delete(e.pointerId);
    if (gesture.current?.id === e.pointerId) gesture.current = null;
  }, []);

  return (
    <TransformWrapper
      ref={transformRef}
      key={page}
      initialScale={1}
      minScale={1}
      maxScale={4}
      limitToBounds
      centerOnInit
      doubleClick={{ mode: "toggle", step: 1.8 }}
      // Trackpad 2 ngón vuốt (không giữ Ctrl) = di chuyển vùng xem khi đã
      // phóng to, giống các trình đọc ảnh/PDF thông thường — Ctrl+vuốt (pinch
      // thật, trình duyệt tự gắn ctrlKey) vẫn zoom như cũ. Mặc định thư viện
      // coi MỌI wheel event là zoom nên 2 ngón chỉ zoom chứ không di chuyển
      // được; wheelDisabled tắt nhánh zoom cho wheel không giữ Ctrl,
      // trackPadPanning bật nhánh di chuyển cho đúng nhánh đó.
      wheel={{ wheelDisabled: true }}
      trackPadPanning={{ disabled: false }}
      onTransform={(_ref, state) => {
        scaleRef.current = state.scale;
      }}
    >
      <TransformComponent wrapperClass="!h-full !w-full" contentClass="!h-full !w-full">
        <div
          ref={boxRef}
          className="flex h-full w-full touch-none items-center justify-center select-none"
          onPointerDown={onPointerDown}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerCancel}
        >
          <div
            className="relative"
            style={box ? { width: box.width, height: box.height } : { width: "100%", height: "100%" }}
          >
            <Image
              src={getPageUrl(bookId, page)}
              alt={`Trang ${page}`}
              fill
              sizes="100vw"
              quality={90}
              priority
              draggable={false}
              className="object-contain"
            />
            {box && <TranslationOverlay regions={regions} />}
          </div>
        </div>
      </TransformComponent>
    </TransformWrapper>
  );
});
