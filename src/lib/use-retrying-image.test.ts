import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  MAX_AUTO_RETRIES,
  retryDelay,
  useRetryingImage,
} from "./use-retrying-image";

/**
 * Ảnh trang sách là toàn bộ nội dung màn hình đọc — hỏng một lần là trang
 * trắng. Trước đây không có gì thử lại, nên chốt hành vi ở đây.
 */
beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

/** Chạy hết giãn cách của lần thử thứ `n` (0-based). */
function waitForRetry(n: number) {
  act(() => {
    vi.advanceTimersByTime(retryDelay(n));
  });
}

describe("tải bình thường", () => {
  it("bắt đầu ở trạng thái đang tải", () => {
    const { result } = renderHook(() => useRetryingImage());

    expect(result.current.status).toBe("loading");
    expect(result.current.attempt).toBe(0);
  });

  it("ảnh lên là xong, không thử lại gì cả", () => {
    const { result } = renderHook(() => useRetryingImage());
    act(() => result.current.onLoad());

    expect(result.current.status).toBe("ok");
    expect(result.current.attempt).toBe(0);
  });
});

describe("hỏng rồi tự thử lại", () => {
  it("đổi attempt sau khi hết giãn cách, để bên gọi ép xin lại", () => {
    const { result } = renderHook(() => useRetryingImage());

    act(() => result.current.onError());
    expect(result.current.retrying).toBe(true);
    // Chưa hết giãn cách thì chưa đổi — nếu đổi ngay là nện máy chủ đang ốm.
    expect(result.current.attempt).toBe(0);

    waitForRetry(0);
    expect(result.current.attempt).toBe(1);
    expect(result.current.status).toBe("loading");
    expect(result.current.retrying).toBe(false);
  });

  it("giãn cách lùi dần 1s, 2s, 4s", () => {
    expect(retryDelay(0)).toBe(1000);
    expect(retryDelay(1)).toBe(2000);
    expect(retryDelay(2)).toBe(4000);
  });

  it("thử lại giữa chừng mà ảnh lên thì dừng hẳn", () => {
    const { result } = renderHook(() => useRetryingImage());
    act(() => result.current.onError());
    waitForRetry(0);
    act(() => result.current.onLoad());

    expect(result.current.status).toBe("ok");

    // Hẹn giờ cũ không được phép bắn tiếp và kéo ngược về "đang tải".
    act(() => vi.advanceTimersByTime(60_000));
    expect(result.current.status).toBe("ok");
  });
});

describe("thua hết các lần tự thử", () => {
  function exhaust(result: { current: ReturnType<typeof useRetryingImage> }) {
    for (let i = 0; i <= MAX_AUTO_RETRIES; i++) {
      act(() => result.current.onError());
      if (i < MAX_AUTO_RETRIES) waitForRetry(i);
    }
  }

  it("chịu thua và báo cho người dùng chứ không thử mãi", () => {
    const { result } = renderHook(() => useRetryingImage());
    exhaust(result);

    expect(result.current.status).toBe("failed");
    expect(result.current.retrying).toBe(false);
  });

  it("không hẹn thêm lần thử nào nữa", () => {
    const { result } = renderHook(() => useRetryingImage());
    exhaust(result);
    const stuck = result.current.attempt;

    act(() => vi.advanceTimersByTime(60_000));
    expect(result.current.attempt).toBe(stuck);
  });

  it("người dùng bấm Tải lại thì chạy lại và được đủ lượt như lần đầu", () => {
    const { result } = renderHook(() => useRetryingImage());
    exhaust(result);

    act(() => result.current.retry());
    expect(result.current.status).toBe("loading");

    // Phải còn nguyên MAX_AUTO_RETRIES lượt, chứ không phải bấm xong hỏng
    // một cái là lại chịu thua ngay.
    act(() => result.current.onError());
    expect(result.current.status).toBe("loading");
    expect(result.current.retrying).toBe(true);
  });
});

describe("dọn dẹp", () => {
  it("huỷ hẹn giờ khi component biến mất", () => {
    const { result, unmount } = renderHook(() => useRetryingImage());
    act(() => result.current.onError());
    expect(vi.getTimerCount()).toBe(1);

    unmount();

    // Phải bằng 0 NGAY lúc gỡ. Đếm sau khi đã chạy hết giờ là vô nghĩa: lúc
    // đó hẹn giờ tự bắn rồi nên đằng nào cũng bằng 0, quên dọn cũng không
    // lòi ra. Lật trang nhanh là dựng/gỡ liên tục, rò mỗi lần một cái.
    expect(vi.getTimerCount()).toBe(0);
  });
});
