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
import { getPageUrl, getPageAspectRatio, type Book } from "@/lib/books";
import { getPageTranslations } from "@/lib/page-translation";
import { TranslationOverlay } from "@/components/reader/translation-overlay";
import { AnnotationLayer } from "@/components/reader/annotation-layer";
import { useAnnotationStore } from "@/lib/annotation-store";

export interface PageViewerHandle {
  resetZoom: () => void;
}

/** Khoảng hở giữa 2 trang ở chế độ xem 2 trang, mô phỏng gáy sách. */
const SPREAD_GAP = 4;

interface PageViewerProps {
  book: Book;
  /** 1 trang (chế độ 1 trang) hoặc 2 trang [trái, phải] (chế độ 2 trang). */
  pages: number[];
  onTap: () => void;
  onSwipePrev: () => void;
  onSwipeNext: () => void;
}

/** 1 ảnh trang + vùng dịch + nét vẽ tay của nó, trong khung đã tính đúng cỡ. */
function PageImage({
  bookId,
  page,
  box,
  aspectRatio,
}: {
  bookId: string;
  page: number;
  box: { width: number; height: number };
  aspectRatio: number;
}) {
  const regions = getPageTranslations(bookId, page);
  return (
    <div className="relative" style={{ width: box.width, height: box.height }}>
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
      <TranslationOverlay regions={regions} />
      {/* Nằm SAU vùng dịch: đang bật chế độ vẽ thì nét vẽ phải nhận được
          chạm trước, nếu không bấm trúng vùng dịch là bật bản dịch thay vì
          vẽ. Tắt chế độ vẽ thì lớp này `pointer-events: none` nên vùng dịch
          bên dưới lại nhận chạm như cũ. */}
      <AnnotationLayer bookId={bookId} page={page} aspectRatio={aspectRatio} />
    </div>
  );
}

export const PageViewer = forwardRef<PageViewerHandle, PageViewerProps>(function PageViewer(
  { book, pages, onTap, onSwipePrev, onSwipeNext },
  ref
) {
  const transformRef = useRef<ReactZoomPanPinchContentRef>(null);
  const scaleRef = useRef(1);
  const aspectRatio = getPageAspectRatio(book);
  // Đang vẽ lên trang: một ngón/chuột thuộc về cây bút, không còn là chạm để
  // ẩn thanh công cụ, vuốt lật trang hay kéo di chuyển trang nữa.
  const drawing = useAnnotationStore((s) => s.active);

  useImperativeHandle(ref, () => ({
    resetZoom: () => transformRef.current?.resetTransform(),
  }));

  /**
   * Khung ẢNH THẬT bên trong khung chứa, cho MỖI trang trong `pages`. Ảnh
   * hiển thị `object-contain` nên luôn có viền trống ở hai bên (hoặc trên
   * dưới) tuỳ tỉ lệ màn hình — vùng bấm xem bản dịch lưu toạ độ theo tỉ lệ
   * của ảnh, đặt theo khung chứa là lệch. Nên phải tự tính.
   *
   * Ở chế độ 2 trang, khung chứa chia đôi theo chiều rộng (trừ khoảng hở
   * SPREAD_GAP) trước khi áp cùng công thức — 2 trang vẫn giữ đúng tỉ lệ,
   * không bị méo.
   *
   * Dùng offsetWidth/offsetHeight chứ KHÔNG dùng getBoundingClientRect: rect
   * đã nhân với transform của react-zoom-pan-pinch, đo bằng nó thì cứ zoom là
   * kích thước lại đổi và tính toán chạy vòng vo.
   */
  const boxRef = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState<{ width: number; height: number } | null>(null);
  const count = pages.length;

  useLayoutEffect(() => {
    const el = boxRef.current;
    if (!el) return;

    function measure(target: HTMLElement) {
      const gapTotal = SPREAD_GAP * (count - 1);
      const maxWidthPerPage = (target.offsetWidth - gapTotal) / count;
      const width = Math.min(maxWidthPerPage, target.offsetHeight * aspectRatio);
      setBox({ width, height: width / aspectRatio });
    }

    measure(el);
    const observer = new ResizeObserver(() => measure(el));
    observer.observe(el);
    return () => observer.disconnect();
  }, [count, aspectRatio]);

  // Theo dõi cử chỉ 1 ngón để phân biệt: tap (bật/tắt toolbar), vuốt ngang
  // (chuyển trang/spread, chỉ khi chưa zoom), hay pinch (2 ngón — bỏ qua, để
  // thư viện zoom xử lý). Chỉ *đọc* sự kiện, không preventDefault/
  // stopPropagation nên không phá cử chỉ pinch/pan gốc của react-zoom-pan-pinch.
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const gesture = useRef<{
    id: number;
    startX: number;
    startY: number;
    cancelled: boolean;
    onRegion: boolean;
  } | null>(null);

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    if (drawing) return;
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
  }, [drawing]);

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
      key={pages.join("-")}
      initialScale={1}
      minScale={1}
      maxScale={4}
      limitToBounds
      centerOnInit
      // Chế độ vẽ: khoá kéo-thả và bấm đúp (chúng nuốt mất nét bút), nhưng
      // CỐ Ý để nguyên pinch 2 ngón và cuộn trackpad — phóng to rồi khoanh
      // chú thích vào chữ nhỏ là chuyện thường xuyên nhất ở đây.
      panning={{ disabled: drawing }}
      doubleClick={{ mode: "toggle", step: 1.8, disabled: drawing }}
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
          style={count > 1 ? { gap: SPREAD_GAP } : undefined}
          onPointerDown={onPointerDown}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerCancel}
        >
          {box &&
            pages.map((p) => (
              <PageImage
                key={p}
                bookId={book.id}
                page={p}
                box={box}
                aspectRatio={aspectRatio}
              />
            ))}
        </div>
      </TransformComponent>
    </TransformWrapper>
  );
});
