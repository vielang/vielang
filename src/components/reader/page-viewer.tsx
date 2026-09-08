"use client";

import Image from "next/image";
import { forwardRef, useImperativeHandle, useRef } from "react";
import {
  TransformWrapper,
  TransformComponent,
  type ReactZoomPanPinchContentRef,
} from "react-zoom-pan-pinch";
import { getPageUrl } from "@/lib/books";

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

  useImperativeHandle(ref, () => ({
    resetZoom: () => transformRef.current?.resetTransform(),
  }));

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
  } | null>(null);

  function onPointerDown(e: React.PointerEvent) {
    pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.current.size === 1) {
      gesture.current = {
        id: e.pointerId,
        startX: e.clientX,
        startY: e.clientY,
        cancelled: false,
      };
    } else if (gesture.current) {
      gesture.current.cancelled = true;
    }
  }

  function onPointerUp(e: React.PointerEvent) {
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
      onTap();
      return;
    }
    if (scaleRef.current <= 1.02 && absDx > 60 && absDx > absDy * 1.5) {
      if (dx < 0) onSwipeNext();
      else onSwipePrev();
    }
  }

  function onPointerCancel(e: React.PointerEvent) {
    pointers.current.delete(e.pointerId);
    if (gesture.current?.id === e.pointerId) gesture.current = null;
  }

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
      onTransform={(_ref, state) => {
        scaleRef.current = state.scale;
      }}
    >
      <TransformComponent wrapperClass="!h-full !w-full" contentClass="!h-full !w-full">
        <div
          className="relative h-full w-full touch-none select-none"
          onPointerDown={onPointerDown}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerCancel}
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
        </div>
      </TransformComponent>
    </TransformWrapper>
  );
});
