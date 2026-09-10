import { act, createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TranslationOverlay } from "./translation-overlay";
import type { TranslationRegion } from "@/lib/page-translation";

/**
 * Test hồi quy cho một lỗi đã xảy ra thật: kéo bong bóng bản dịch làm trang tự
 * lật sang trang kế tiếp.
 *
 * Bong bóng dựng qua portal nên nằm ở `document.body`, NHƯNG portal của React
 * vẫn cho sự kiện nổi bọt theo CÂY COMPONENT chứ không theo cây DOM. Vì
 * `TranslationOverlay` là con của khung ảnh trong `page-viewer` — nơi bắt cử
 * chỉ vuốt để lật trang — nên pointer event khi kéo bong bóng chạy thẳng vào
 * handler đó và bị tính là vuốt ngang.
 */

const REGION: TranslationRegion = {
  id: "r1",
  rect: [0.1, 0.1, 0.5, 0.2],
  label: "Bài đọc",
  vi: "Xin chào.",
};

let container: HTMLDivElement;
let root: Root;

beforeEach(() => {
  // Cờ React dùng để biết đang chạy trong `act()`; không có kiểu sẵn nên gán qua cast.
  (globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = true;
  // jsdom không có API pointer capture; component gọi nó khi bắt đầu kéo.
  HTMLElement.prototype.setPointerCapture = () => {};
  HTMLElement.prototype.releasePointerCapture = () => {};
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
});

/** jsdom chưa có PointerEvent; React chỉ cần đúng TÊN sự kiện là dựng được. */
function pointerEvent(type: string, x = 0, y = 0) {
  return new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y });
}

function renderInsideGestureArea(onParentPointerDown: () => void) {
  act(() => {
    root.render(
      createElement(
        "div",
        { onPointerDown: onParentPointerDown },
        createElement(TranslationOverlay, { regions: [REGION] })
      )
    );
  });
}

describe("TranslationOverlay", () => {
  it("mở bong bóng khi bấm vào vùng", () => {
    renderInsideGestureArea(() => {});
    const region = container.querySelector("[data-translate-region]") as HTMLElement;
    act(() => {
      region.dispatchEvent(pointerEvent("click", 100, 100));
    });

    const bubble = document.querySelector('[role="tooltip"]');
    expect(bubble?.textContent).toBe("Xin chào.");
    // Dựng ra ngoài body, không nằm trong cây DOM đã bị transform.
    expect(bubble?.parentElement).toBe(document.body);
  });

  it("kéo bong bóng KHÔNG được rò pointer event lên vùng bắt cử chỉ lật trang", () => {
    const onParentPointerDown = vi.fn();
    renderInsideGestureArea(onParentPointerDown);

    const region = container.querySelector("[data-translate-region]") as HTMLElement;
    act(() => {
      region.dispatchEvent(pointerEvent("click", 100, 100));
    });
    onParentPointerDown.mockClear();

    const bubble = document.querySelector('[role="tooltip"]') as HTMLElement;
    act(() => {
      bubble.dispatchEvent(pointerEvent("pointerdown", 100, 120));
      bubble.dispatchEvent(pointerEvent("pointermove", 400, 120));
      bubble.dispatchEvent(pointerEvent("pointerup", 400, 120));
    });

    expect(onParentPointerDown).not.toHaveBeenCalled();
  });
});
