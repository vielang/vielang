import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { ZoomBar } from "./zoom-bar";
import { useZoomStore } from "@/lib/zoom-store";

beforeEach(() => useZoomStore.setState({ scale: 1 }));

describe("thanh phóng to", () => {
  it("hiện mức phóng hiện tại", () => {
    useZoomStore.setState({ scale: 1.5 });
    render(<ZoomBar visible onZoomTo={() => {}} />);

    expect(screen.getByRole("button", { name: /Đang phóng 150%/ })).toBeTruthy();
  });

  it("＋ / － đi từng nấc 25%", () => {
    useZoomStore.setState({ scale: 1.5 });
    const onZoomTo = vi.fn();
    render(<ZoomBar visible onZoomTo={onZoomTo} />);
    fireEvent.click(screen.getByRole("button", { name: "Phóng to" }));
    fireEvent.click(screen.getByRole("button", { name: "Thu nhỏ" }));

    expect(onZoomTo.mock.calls).toEqual([[1.75], [1.25]]);
  });

  it("bấm vào con số phần trăm là về 100%", () => {
    useZoomStore.setState({ scale: 2.25 });
    const onZoomTo = vi.fn();
    render(<ZoomBar visible onZoomTo={onZoomTo} />);
    fireEvent.click(screen.getByRole("button", { name: /Đang phóng 225%/ }));

    expect(onZoomTo).toHaveBeenCalledWith(1);
  });

  it("ở 100% thì không thu nhỏ được nữa, ở 400% thì không phóng thêm được", () => {
    const { rerender } = render(<ZoomBar visible onZoomTo={() => {}} />);
    expect((screen.getByRole("button", { name: "Thu nhỏ" }) as HTMLButtonElement).disabled).toBe(true);

    useZoomStore.setState({ scale: 4 });
    rerender(<ZoomBar visible onZoomTo={() => {}} />);
    expect((screen.getByRole("button", { name: "Phóng to" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("ẩn cùng thanh công cụ, và lúc ẩn thì không chặn chạm lên trang", () => {
    render(<ZoomBar visible={false} onZoomTo={() => {}} />);
    const bar = screen.getByRole("group", { name: "Phóng to, thu nhỏ trang" });

    expect(bar.className).toMatch(/opacity-0/);
    expect(bar.className).toMatch(/pointer-events-none/);
  });
});
