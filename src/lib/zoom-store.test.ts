import { describe, expect, it } from "vitest";
import { DOUBLE_TAP_STEP_NARROW, DOUBLE_TAP_STEP_WIDE, nextZoom, isZoomed } from "./zoom-store";

describe("nấc phóng to", () => {
  it("mỗi nấc 25%", () => {
    expect(nextZoom(1, 1)).toBe(1.25);
    expect(nextZoom(1.5, -1)).toBe(1.25);
  });

  it("đang ở mức lẻ (vd sau khi chụm tay) thì bám vào nấc gần nhất theo hướng bấm", () => {
    expect(nextZoom(1.37, 1)).toBe(1.5);
    expect(nextZoom(1.37, -1)).toBe(1.25);
  });

  it("không vượt quá 100%–400%", () => {
    expect(nextZoom(1, -1)).toBe(1);
    expect(nextZoom(4, 1)).toBe(4);
    expect(nextZoom(3.9, 1)).toBe(4);
  });

  it("bấm đúp: máy tính lên 175%, điện thoại lên 250%", () => {
    expect(1 + DOUBLE_TAP_STEP_WIDE).toBe(1.75);
    expect(1 + DOUBLE_TAP_STEP_NARROW).toBe(2.5);
  });
});

describe("isZoomed", () => {
  it("chụm tay nhả ra ở 101% vẫn coi là chưa phóng — chạm mép vẫn lật trang", () => {
    expect(isZoomed(1)).toBe(false);
    expect(isZoomed(1.01)).toBe(false);
    expect(isZoomed(1.25)).toBe(true);
  });
});
