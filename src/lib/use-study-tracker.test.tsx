import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { useActivityStore } from "./activity-store";
import { dayKey } from "./activity";
import { DWELL_MS, IDLE_MS, TICK_MS, shouldCountTick, useStudyTracker } from "./use-study-tracker";

function Probe({ pages }: { pages: number[] }) {
  useStudyTracker("step1", pages);
  return null;
}

const today = () => useActivityStore.getState().days[dayKey(new Date())];

beforeEach(() => {
  vi.useFakeTimers();
  useActivityStore.setState({ days: {}, studied: {} });
});
afterEach(() => vi.useRealTimers());

describe("nhịp tính giờ", () => {
  it("chỉ tính khi trang đang hiện và có thao tác gần đây", () => {
    expect(shouldCountTick(IDLE_MS, 0, true)).toBe(true);
    expect(shouldCountTick(IDLE_MS + 1, 0, true)).toBe(false);
    expect(shouldCountTick(1_000, 0, false)).toBe(false);
  });
});

describe("theo dõi trên trang đọc", () => {
  it("cộng thời gian học theo từng nhịp", () => {
    render(<Probe pages={[18]} />);
    act(() => vi.advanceTimersByTime(TICK_MS * 2));

    expect(today().activeMs).toBe(TICK_MS * 2);
  });

  it("bỏ máy đó quá lâu thì thôi tính giờ", () => {
    render(<Probe pages={[18]} />);
    act(() => vi.advanceTimersByTime(IDLE_MS + TICK_MS * 4));

    expect(today().activeMs).toBeLessThanOrEqual(IDLE_MS);
  });

  it("chỉ tính 'đã học' khi ở lại trang đủ lâu", () => {
    render(<Probe pages={[18, 19]} />);
    act(() => vi.advanceTimersByTime(DWELL_MS - 1));
    expect(useActivityStore.getState().studied.step1).toBeUndefined();

    act(() => vi.advanceTimersByTime(1));
    expect(useActivityStore.getState().studied.step1).toEqual([18, 19]);
  });

  it("lật đi trước khi đủ thời gian thì trang đó không tính", () => {
    const { rerender } = render(<Probe pages={[18]} />);
    act(() => vi.advanceTimersByTime(DWELL_MS / 2));
    rerender(<Probe pages={[20]} />);
    act(() => vi.advanceTimersByTime(DWELL_MS));

    expect(useActivityStore.getState().studied.step1).toEqual([20]);
  });
});
