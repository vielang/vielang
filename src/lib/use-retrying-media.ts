"use client";

import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Tự thử lại khi ảnh hoặc audio tải hỏng.
 *
 * Vì sao cần: ảnh trang sách là TOÀN BỘ nội dung của màn hình đọc, còn audio
 * là cả phần bài nghe. Hỏng một lần là trang trắng hoặc trình phát chết, mà
 * trước đây không có gì thử lại — người dùng phải lật sang trang khác rồi lật
 * về mới thấy lại chữ. Một cú chập mạng lúc đi tàu điện là đủ.
 *
 * Bên audio có một điểm khác ảnh mà hook này KHÔNG tự lo: dựng lại thẻ
 * <audio> là mất chỗ đang nghe. Bên gọi phải tự nhớ `currentTime` rồi đặt
 * lại — xem `TrackPlayer` trong `audio-widget.tsx`.
 *
 * KHÔNG thêm tham số phá cache vào URL. Nghe thì chắc ăn hơn, nhưng lại hỏng
 * đúng lúc cần nhất: `sw.js` tra cache theo nguyên URL, nên
 * `...0020.webp?retry=1` sẽ trượt cả cache ảnh lẫn cache sách đã tải về —
 * tức đang offline mà thử lại là chắc chắn trượt, dù trang đó đã nằm sẵn
 * trong máy. Thay vào đó giữ nguyên URL, chỉ tăng `attempt` để bên gọi đổi
 * `key` của <img>, ép React dựng phần tử mới. Vẫn xin lại thật, vì lần hỏng
 * trước không được lưu ở đâu cả (`cacheFirst` chỉ lưu bản trả về `ok`), nên
 * không có gì cũ để trình duyệt đem ra dùng lại.
 *
 * KHÔNG tự đặt lại trạng thái khi `src` đổi. Bên gọi phải gắn `key` theo số
 * trang để React tự dựng lại component khi lật trang (xem `PageImage` trong
 * `page-viewer.tsx`). Làm vậy gọn hơn: không cần effect đặt lại state, tránh
 * luôn cái hẹn giờ của trang cũ bắn nhầm vào trang mới.
 */
export type MediaLoadStatus = "loading" | "ok" | "failed";

/** Số lần TỰ thử lại trước khi chịu thua và hỏi người dùng. */
export const MAX_AUTO_RETRIES = 3;

/** Giãn cách giữa các lần thử: 1s, 2s, 4s. Lùi dần để không nện máy chủ đang ốm. */
export function retryDelay(attempt: number): number {
  return 1000 * 2 ** attempt;
}

export interface RetryingImage {
  /** Đưa vào `key` của <img> — đổi giá trị là ép xin lại. */
  attempt: number;
  status: MediaLoadStatus;
  /** true khi đang chờ hết giãn cách để tự thử lại. */
  retrying: boolean;
  onLoad: () => void;
  onError: () => void;
  /** Người dùng tự bấm "Tải lại" sau khi đã thua hết các lần tự thử. */
  retry: () => void;
}

export function useRetryingMedia(): RetryingImage {
  const [attempt, setAttempt] = useState(0);
  const [status, setStatus] = useState<MediaLoadStatus>("loading");
  const [retrying, setRetrying] = useState(false);
  // Đếm bằng ref chứ không đọc state trong `onError`: `onError` có thể bắn
  // nhiều lần trước khi React vẽ lại, đọc state cũ là đếm thiếu.
  const attemptRef = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clear = useCallback(() => {
    if (timer.current !== null) {
      clearTimeout(timer.current);
      timer.current = null;
    }
  }, []);

  useEffect(() => clear, [clear]);

  const onLoad = useCallback(() => {
    clear();
    setRetrying(false);
    setStatus("ok");
  }, [clear]);

  const onError = useCallback(() => {
    clear();
    const n = attemptRef.current;
    if (n >= MAX_AUTO_RETRIES) {
      setRetrying(false);
      setStatus("failed");
      return;
    }
    setStatus("loading");
    setRetrying(true);
    timer.current = setTimeout(() => {
      timer.current = null;
      attemptRef.current = n + 1;
      setRetrying(false);
      setAttempt(n + 1);
    }, retryDelay(n));
  }, [clear]);

  const retry = useCallback(() => {
    clear();
    // Đếm lại từ đầu: người dùng chủ động bấm thì cho đủ lượt tự thử lại như
    // lần đầu, chứ không phải bấm một cái rồi lại chịu thua ngay.
    attemptRef.current = 0;
    setStatus("loading");
    setRetrying(false);
    setAttempt((n) => n + 1);
  }, [clear]);

  return { attempt, status, retrying, onLoad, onError, retry };
}
