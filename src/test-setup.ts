import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";

/**
 * Vá những chỗ jsdom không có, để test được component vẽ.
 *
 * Đây đúng là phần dễ mất công nhất khi test UI trong jsdom: nó không dựng
 * layout thật nên mọi phép đo đều bằng 0, và không có API con trỏ. Vá ở đây
 * một lần thay vì rắc vào từng file test.
 */

// jsdom chưa có PointerEvent. Kế thừa MouseEvent để giữ nguyên clientX/clientY,
// rồi thêm mấy trường mà lớp vẽ và thanh công cụ thật sự đọc tới.
if (typeof window.PointerEvent === "undefined") {
  class PointerEventPolyfill extends MouseEvent {
    pointerId: number;
    pointerType: string;
    pressure: number;

    constructor(type: string, params: PointerEventInit = {}) {
      super(type, params);
      this.pointerId = params.pointerId ?? 1;
      this.pointerType = params.pointerType ?? "mouse";
      this.pressure = params.pressure ?? 0.5;
    }
  }
  window.PointerEvent = PointerEventPolyfill as unknown as typeof PointerEvent;
}

// Bắt/nhả con trỏ: test không cần hành vi thật, chỉ cần gọi không nổ.
Element.prototype.setPointerCapture ??= () => {};
Element.prototype.releasePointerCapture ??= () => {};
Element.prototype.hasPointerCapture ??= () => false;

if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

/**
 * jsdom trả offsetWidth/offsetHeight = 0 cho mọi thứ, mà thanh công cụ dựa
 * vào đó để tự kẹp trong màn hình — cỡ 0 thì `clamp` tính sai hết. Cho một
 * cỡ cố định hợp lý để test nói được về vị trí.
 */
export const TEST_ELEMENT_WIDTH = 300;
export const TEST_ELEMENT_HEIGHT = 44;

Object.defineProperty(HTMLElement.prototype, "offsetWidth", {
  configurable: true,
  get() {
    return TEST_ELEMENT_WIDTH;
  },
});
Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
  configurable: true,
  get() {
    return TEST_ELEMENT_HEIGHT;
  },
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});
