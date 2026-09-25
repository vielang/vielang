import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { AudioWidget } from "./audio-widget";
import { useAudioWidgetStore } from "@/lib/audio-widget-store";
import { MAX_AUTO_RETRIES, retryDelay } from "@/lib/use-retrying-media";

/**
 * Trình duyệt tính `touch-action` bằng cách đi ngược lên cây cha. Đặt
 * `touch-none` ở hộp ngoài cùng của widget là chặn luôn cử chỉ chạm của thẻ
 * <audio> bên trong — trên điện thoại, chạm nút play lại bật ra menu ba chấm
 * "Download / Playback speed".
 *
 * Lỗi kiểu này không công cụ nào bắt được: kiểu dữ liệu đúng, lint sạch,
 * build chạy, và trên máy tính thì không tái hiện. Nên chốt lại ở đây.
 */
const TRACKS = {
  10: [{ type: "intro", label: "Mở bài", url: "https://x/a.mp3" }],
};

function widget() {
  return render(<AudioWidget pages={[10]} tracksByPage={TRACKS} />);
}

/** Đi ngược lên cây cha, gom class của mọi tổ tiên trong widget. */
function ancestorClasses(el: Element): string[] {
  const out: string[] = [];
  for (let node = el.parentElement; node && node !== document.body; node = node.parentElement) {
    out.push(node.className);
  }
  return out;
}

beforeEach(() => {
  useAudioWidgetStore.setState({
    collapsed: false,
    position: { x: 10, y: 10 },
    collapsedPosition: { x: 10, y: 10 },
    activeType: null,
    side: "left",
  });
});

describe("thẻ audio của trình duyệt", () => {
  it("không nằm dưới bất kỳ tổ tiên nào chặn cử chỉ chạm", () => {
    const { container } = widget();
    const audio = container.querySelector("audio")!;

    expect(audio).toBeTruthy();
    for (const className of ancestorClasses(audio)) {
      expect(className).not.toMatch(/\btouch-none\b/);
    }
  });

  it("không bị ép chiều cao", () => {
    // Chrome dựng bộ điều khiển ở ~54px; bóp xuống 32px là mấy vùng chạm bên
    // trong dồn cục lại, chạm nút play rất dễ trúng nút ba chấm bên cạnh.
    const { container } = widget();
    const audio = container.querySelector("audio")!;

    expect(audio.className).not.toMatch(/\bh-\d/);
  });
});

describe("vùng kéo vẫn phải chặn cử chỉ chạm", () => {
  it("thanh kéo có touch-none", () => {
    // Thiếu thì kéo widget trên điện thoại sẽ thành cuộn trang.
    const { container } = widget();
    const grip = container.querySelector(".cursor-grab");

    expect(grip?.className).toMatch(/\btouch-none\b/);
  });

  it("nút tròn lúc thu nhỏ cũng vậy", () => {
    useAudioWidgetStore.setState({ collapsed: true });
    const { container } = widget();
    const button = container.querySelector("button")!;

    expect(button.className).toMatch(/\btouch-none\b/);
  });
});

/**
 * Mạng chập giữa bài nghe.
 *
 * Bài nghe dài vài phút. Dựng lại thẻ <audio> để xin lại file là về mốc 0
 * giây — đang nghe dở mà bị kéo về đầu thì còn tệ hơn cả đứng im, nên phần
 * nhớ chỗ đang nghe là thứ phải chốt lại.
 */
describe("bài nghe hỏng giữa chừng", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  /** Nghe tới giây thứ `t` rồi mạng đứt. */
  function playUntilThenFail(container: HTMLElement, t: number) {
    const audio = container.querySelector("audio")!;
    audio.currentTime = t;
    fireEvent.timeUpdate(audio);
    fireEvent.error(audio);
  }

  it("quay lại đúng chỗ đang nghe sau khi thử lại", () => {
    const { container } = widget();
    playUntilThenFail(container, 83);

    act(() => vi.advanceTimersByTime(retryDelay(0)));

    // Thẻ mới dựng luôn bắt đầu ở 0 — phải tự kéo về chỗ cũ.
    const retried = container.querySelector("audio")!;
    expect(retried.currentTime).toBe(0);
    fireEvent.loadedMetadata(retried);
    expect(retried.currentTime).toBe(83);
  });

  it("chưa nghe gì thì không nhảy lung tung", () => {
    const { container } = widget();
    fireEvent.error(container.querySelector("audio")!);

    act(() => vi.advanceTimersByTime(retryDelay(0)));

    const retried = container.querySelector("audio")!;
    fireEvent.loadedMetadata(retried);
    expect(retried.currentTime).toBe(0);
  });

  it("thua hết các lần tự thử thì hỏi người dùng, không thử mãi", () => {
    const { container } = widget();
    for (let i = 0; i <= MAX_AUTO_RETRIES; i++) {
      fireEvent.error(container.querySelector("audio")!);
      if (i < MAX_AUTO_RETRIES) act(() => vi.advanceTimersByTime(retryDelay(i)));
    }

    expect(screen.getByText(/Không tải được/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /Tải lại/ })).toBeTruthy();
  });

  it("chưa thua thì chưa làm phiền người dùng", () => {
    const { container } = widget();
    fireEvent.error(container.querySelector("audio")!);

    // Đang tự thử lại trong im lặng — hiện lời nhắn lỗi lúc này là doạ
    // người dùng vì một cú chập mạng mà máy sắp tự xử lý xong.
    expect(screen.queryByText(/Không tải được/)).toBeNull();
  });
});

describe("nút tròn lúc thu nhỏ", () => {
  it("cử chỉ bị huỷ giữa chừng thì KHÔNG tự bung panel", () => {
    useAudioWidgetStore.setState({ collapsed: true });
    widget();
    const button = screen.getByRole("button", { name: "Mở audio trang này" });
    fireEvent.pointerDown(button, { pointerId: 1, clientX: 10, clientY: 10 });
    fireEvent.pointerCancel(button, { pointerId: 1, clientX: 10, clientY: 10 });
    expect(useAudioWidgetStore.getState().collapsed).toBe(true);
  });

  it("nhấn rồi nhả (không kéo) thì bung panel", () => {
    useAudioWidgetStore.setState({ collapsed: true });
    widget();
    const button = screen.getByRole("button", { name: "Mở audio trang này" });
    fireEvent.pointerDown(button, { pointerId: 1, clientX: 10, clientY: 10 });
    fireEvent.pointerUp(button, { pointerId: 1, clientX: 10, clientY: 10 });
    expect(useAudioWidgetStore.getState().collapsed).toBe(false);
  });
});
