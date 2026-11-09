"use client";

import { useEffect, useRef } from "react";
import { useActivityStore } from "@/lib/activity-store";

/** Cứ ngần này lại xét một lần xem có đang học không, và cộng đúng ngần này. */
export const TICK_MS = 15_000;

/**
 * Không thao tác gì quá ngần này thì thôi tính giờ. Đọc một đoạn giải thích
 * dài mất cả phút mà không chạm gì là bình thường, nên không để ngắn hơn;
 * nhưng để máy đó đi nấu cơm thì phải dừng.
 */
export const IDLE_MS = 90_000;

/** Nhịp này có được tính là thời gian học không. */
export function shouldCountTick(now: number, lastInteraction: number, visible: boolean): boolean {
  return visible && now - lastInteraction <= IDLE_MS;
}

const INTERACTIONS = ["pointerdown", "keydown", "wheel", "touchstart"] as const;

/**
 * Đếm "thời gian học thật" = trang đang hiện trên màn hình VÀ người dùng có
 * thao tác trong `IDLE_MS` gần nhất. Chỉ đo lúc tab mở thì chuyển tab khác,
 * hay mở bài rồi bỏ đó, cũng thành hàng giờ "học".
 *
 * Mở trang được tính là một thao tác: người ta thường vào trang rồi ngồi đọc
 * luôn, không chạm gì thêm.
 */
export function useActiveTime() {
  const addActiveTime = useActivityStore((s) => s.addActiveTime);
  const lastInteraction = useRef(0);

  useEffect(() => {
    lastInteraction.current = Date.now();
    const touch = () => {
      lastInteraction.current = Date.now();
    };
    INTERACTIONS.forEach((type) =>
      window.addEventListener(type, touch, { passive: true, capture: true })
    );
    const timer = window.setInterval(() => {
      const visible = document.visibilityState === "visible";
      if (shouldCountTick(Date.now(), lastInteraction.current, visible)) {
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
}

