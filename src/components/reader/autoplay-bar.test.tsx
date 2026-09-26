import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { AutoplayBar } from "./autoplay-bar";
import { useAutoplayStore } from "@/lib/autoplay-player";
import { buildPlaylist } from "@/lib/autoplay";
import { TEST_ELEMENT_HEIGHT, TEST_ELEMENT_WIDTH } from "@/test-setup";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn() }) }));

const stop = vi.hoisted(() => vi.fn());
vi.mock("@/lib/autoplay-player", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/autoplay-player")>()),
  stopAutoplay: stop,
}));

function bar(): HTMLElement {
  return screen.getByRole("region", { name: "Nghe tự động" });
}

function drag(el: HTMLElement, dx: number, dy: number) {
  fireEvent.pointerDown(el, { pointerId: 1, clientX: 400, clientY: 600 });
  fireEvent.pointerMove(window, { pointerId: 1, clientX: 400 + dx, clientY: 600 + dy });
  fireEvent.pointerUp(window, { pointerId: 1, clientX: 400 + dx, clientY: 600 + dy });
}

beforeEach(() => {
  useAutoplayStore.setState({
    bookId: "en-elementary",
    queue: buildPlaylist("en-elementary"),
    index: 0,
    status: "playing",
    visiblePages: [1],
    double: false,
    barPos: null,
    barMoved: false,
  });
});

describe("thanh nghe tự động", () => {
  it("chưa kéo thì đứng giữa đáy, ngay trên thanh lật trang", () => {
    render(<AutoplayBar bookId="en-elementary" toolbarVisible />);
    const pos = useAutoplayStore.getState().barPos!;
    expect(pos.x).toBe(Math.round((window.innerWidth - TEST_ELEMENT_WIDTH) / 2));
    expect(pos.y).toBeLessThan(window.innerHeight - TEST_ELEMENT_HEIGHT - 64);
    expect(bar().style.visibility).toBe("");
  });

  it("chưa kéo thì bám theo thanh lật trang: ẩn thanh là tụt xuống sát đáy", () => {
    const { rerender } = render(<AutoplayBar bookId="en-elementary" toolbarVisible />);
    const above = useAutoplayStore.getState().barPos!.y;
    rerender(<AutoplayBar bookId="en-elementary" toolbarVisible={false} />);
    expect(useAutoplayStore.getState().barPos!.y).toBeGreaterThan(above);
  });

  it("kéo được đi chỗ khác, và đứng yên đó dù thanh lật trang ẩn/hiện", () => {
    const { rerender } = render(<AutoplayBar bookId="en-elementary" toolbarVisible />);
    const before = useAutoplayStore.getState().barPos!;
    drag(bar(), -200, -300);

    const after = useAutoplayStore.getState();
    expect(after.barMoved).toBe(true);
    expect(after.barPos).toEqual({ x: before.x - 200, y: before.y - 300 });

    rerender(<AutoplayBar bookId="en-elementary" toolbarVisible={false} />);
    expect(useAutoplayStore.getState().barPos).toEqual(after.barPos);
  });

  it("kéo bắt đầu ngay trên một cái nút thì không vô tình bấm nút đó", () => {
    render(<AutoplayBar bookId="en-elementary" toolbarVisible />);
    const close = screen.getByRole("button", { name: "Tắt nghe tự động" });
    drag(close, 100, 0);
    fireEvent.click(close);
    expect(stop).not.toHaveBeenCalled();

    fireEvent.click(close);
    expect(stop).toHaveBeenCalledTimes(1);
  });

  it("không kéo ra khỏi màn hình được", () => {
    render(<AutoplayBar bookId="en-elementary" toolbarVisible />);
    drag(bar(), 5000, 5000);
    const pos = useAutoplayStore.getState().barPos!;
    expect(pos.x + TEST_ELEMENT_WIDTH).toBeLessThanOrEqual(window.innerWidth);
    expect(pos.y + TEST_ELEMENT_HEIGHT).toBeLessThanOrEqual(window.innerHeight);
  });
});
