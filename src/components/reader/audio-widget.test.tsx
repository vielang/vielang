import { beforeEach, describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import { AudioWidget } from "./audio-widget";
import { useAudioWidgetStore } from "@/lib/audio-widget-store";

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
