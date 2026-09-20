import { beforeEach, describe, expect, it } from "vitest";
import {
  AUDIO_WIDGET_SIZE,
  audioAnchor,
  audioTop,
  drawToolbarAnchor,
  notePanelAnchor,
  notePanelSize,
  recorderAnchor,
} from "./widget-dock";

/**
 * Bốn widget nổi cùng lúc trong trang đọc. Test ở đây chỉ hỏi đúng một câu:
 * chúng có ĐÈ LÊN NHAU ngay từ lúc mở không — chứ không đi kiểm từng con số,
 * vì con số thì còn chỉnh.
 */
const W = 390; // màn điện thoại
const H = 780;

/** Cỡ thật (xấp xỉ) của từng widget, xem chính component của chúng. */
const DRAW = { w: 45, h: 240 };
const RECORDER = { w: 288, h: 300 };

function box(pos: { x: number; y: number }, w: number, h: number) {
  return { left: pos.x, top: pos.y, right: pos.x + w, bottom: pos.y + h };
}

function overlaps(a: ReturnType<typeof box>, b: ReturnType<typeof box>): boolean {
  return a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
}

beforeEach(() => {
  window.innerWidth = W;
  window.innerHeight = H;
});

describe("bố cục mặc định", () => {
  const audio = () => box(audioAnchor(), AUDIO_WIDGET_SIZE, AUDIO_WIDGET_SIZE);
  const draw = () => box(drawToolbarAnchor(DRAW.w, DRAW.h), DRAW.w, DRAW.h);
  const recorder = () =>
    box(recorderAnchor(RECORDER.w, RECORDER.h), RECORDER.w, RECORDER.h);
  const note = () => {
    const size = notePanelSize();
    return box(notePanelAnchor(), size.width, size.height);
  };

  it("thanh vẽ không đè lên nút audio", () => {
    expect(overlaps(draw(), audio())).toBe(false);
  });

  it("bảng ghi âm không đè lên nút audio", () => {
    // Trước đây bảng canh giữa đáy, rộng 288px trên màn 390px nên với sang
    // tận góc phải và trùm lên nút audio.
    expect(overlaps(recorder(), audio())).toBe(false);
  });

  it("bảng ghi âm không đè lên thanh vẽ", () => {
    expect(overlaps(recorder(), draw())).toBe(false);
  });

  it("panel bài giảng không đè lên nút audio", () => {
    // Trước đây panel chừa 140px dưới đáy, vẫn thò quá mép trên nút audio.
    expect(overlaps(note(), audio())).toBe(false);
  });

  it("panel bài giảng không đè lên thanh vẽ", () => {
    expect(overlaps(note(), draw())).toBe(false);
  });

  it("không widget nào thò ra ngoài màn hình", () => {
    for (const b of [audio(), draw(), recorder(), note()]) {
      expect(b.left).toBeGreaterThanOrEqual(0);
      expect(b.top).toBeGreaterThanOrEqual(0);
      expect(b.right).toBeLessThanOrEqual(W);
      expect(b.bottom).toBeLessThanOrEqual(H);
    }
  });
});

describe("màn hình thấp", () => {
  it("panel bài giảng vẫn cao tối thiểu để dùng được", () => {
    window.innerHeight = 420;
    expect(notePanelSize().height).toBeGreaterThanOrEqual(220);
  });

  it("nút audio vẫn nằm trong màn", () => {
    window.innerHeight = 420;
    expect(audioTop()).toBeGreaterThan(0);
  });
});
