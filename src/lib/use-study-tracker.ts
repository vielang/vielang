"use client";

import { useEffect, useRef } from "react";
import { useActivityStore } from "@/lib/activity-store";

/** Cứ ngần này lại xét một lần xem có đang học không, và cộng đúng ngần này. */
export const TICK_MS = 15_000;

/**
 * Không thao tác gì quá ngần này thì thôi tính giờ. Đọc một trang sách tiếng
 * Hàn mất cả phút mà không chạm gì là bình thường, nên không để ngắn hơn;
 * nhưng để máy đó đi nấu cơm thì phải dừng.
 */
export const IDLE_MS = 90_000;

/**
 * Ở lại trang ít nhất ngần này mới tính là "đã học" trang đó. Lật lướt để
 * tìm bài thì mỗi trang chỉ dừng một hai giây — không tính.
 */
export const DWELL_MS = 12_000;

/** Nhịp này có được tính là thời gian học không. */
export function shouldCountTick(now: number, lastInteraction: number, visible: boolean): boolean {
  return visible && now - lastInteraction <= IDLE_MS;
}

const INTERACTIONS = ["pointerdown", "keydown", "wheel", "touchstart"] as const;

/**
 * Ghi lịch sử học từ trang đọc: thời gian học thật và trang đã học.
 *
 * "Thời gian học thật" = trang đọc đang hiện trên màn hình VÀ người dùng có
 * thao tác trong `IDLE_MS` gần nhất. Chỉ đo lúc tab mở thì chuyển tab khác,
 * hay mở sách rồi bỏ đó, cũng thành hàng giờ "học".
 *
 * Mở trang được tính là một thao tác: người ta thường vào trang rồi ngồi đọc
 * luôn, không chạm gì thêm.
 */
export function useStudyTracker(bookId: string, pages: number[]) {
  const addActiveTime = useActivityStore((s) => s.addActiveTime);
  const markPageStudied = useActivityStore((s) => s.markPageStudied);
  const lastInteraction = useRef(0);
  const pagesKey = pages.join(",");

  useEffect(() => {
    lastInteraction.current = Date.now();
    const touch = () => {
      lastInteraction.current = Date.now();
    };
    INTERACTIONS.forEach((type) =>
      window.addEventListener(type, touch, { passive: true, capture: true })
    );
    const timer = window.setInterval(() => {
      if (
        shouldCountTick(
          Date.now(),
          lastInteraction.current,
          document.visibilityState === "visible"
        )
      ) {
        addActiveTime(TICK_MS);
      }
    }, TICK_MS);
    return () => {
      INTERACTIONS.forEach((type) =>
        window.removeEventListener(type, touch, { capture: true })
      );
      window.clearInterval(timer);
    };
  }, [addActiveTime]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      // Tab bị ẩn suốt lúc chờ thì không tính: người ta mở trang rồi đi chỗ khác.
      if (document.visibilityState !== "visible") return;
      pagesKey
        .split(",")
        .map(Number)
        .forEach((p) => markPageStudied(bookId, p));
    }, DWELL_MS);
    return () => window.clearTimeout(timer);
  }, [bookId, pagesKey, markPageStudied]);
}
