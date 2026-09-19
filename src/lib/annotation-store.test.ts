import { beforeEach, describe, expect, it } from "vitest";
import {
  HIGHLIGHTER_WIDTHS,
  PEN_COLORS,
  PEN_WIDTHS,
  activeWidth,
  annotationKey,
  paletteFor,
  useAnnotationStore,
  type Stroke,
} from "./annotation-store";

const BOOK = "step1";

function stroke(id: string, tool: Stroke["tool"] = "pen"): Stroke {
  return { id, tool, color: "#ef4444", width: 0.004, points: [0, 0, 1, 1] };
}

function strokesAt(page: number): Stroke[] | undefined {
  return useAnnotationStore.getState().strokes[annotationKey(BOOK, page)];
}

const actions = () => useAnnotationStore.getState();

beforeEach(() => {
  localStorage.clear();
  useAnnotationStore.setState({
    strokes: {},
    hasHydrated: true,
    quotaExceeded: false,
    active: false,
    tool: "pen",
    penColor: PEN_COLORS[0],
    size: 1,
    lastPage: null,
  });
});

describe("addStroke", () => {
  it("giữ nét theo từng trang, không lẫn sang trang khác", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().addStroke(BOOK, 11, stroke("b"));

    expect(strokesAt(10)?.map((s) => s.id)).toEqual(["a"]);
    expect(strokesAt(11)?.map((s) => s.id)).toEqual(["b"]);
  });

  it("giữ đúng thứ tự vẽ — nét sau nằm trên nét trước", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().addStroke(BOOK, 10, stroke("b"));

    expect(strokesAt(10)?.map((s) => s.id)).toEqual(["a", "b"]);
  });
});

describe("eraseStrokes", () => {
  it("xoá một lượt nhiều nét mà đầu tẩy chạm phải", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().addStroke(BOOK, 10, stroke("b"));
    actions().addStroke(BOOK, 10, stroke("c"));
    actions().eraseStrokes(BOOK, 10, ["a", "c"]);

    expect(strokesAt(10)?.map((s) => s.id)).toEqual(["b"]);
  });

  it("xoá hết nét thì bỏ luôn entry của trang", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().eraseStrokes(BOOK, 10, ["a"]);

    expect(strokesAt(10)).toBeUndefined();
  });

  it("giữ nguyên state khi không chạm nét nào", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    const before = useAnnotationStore.getState().strokes;
    actions().eraseStrokes(BOOK, 10, ["khong-co"]);

    expect(useAnnotationStore.getState().strokes).toBe(before);
  });
});

describe("undo", () => {
  it("bỏ đúng nét vẽ sau cùng", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().addStroke(BOOK, 10, stroke("b"));
    actions().undo(BOOK, 10);

    expect(strokesAt(10)?.map((s) => s.id)).toEqual(["a"]);
  });

  it("không làm gì khi trang chưa có nét nào", () => {
    const before = useAnnotationStore.getState().strokes;
    actions().undo(BOOK, 10);

    expect(useAnnotationStore.getState().strokes).toBe(before);
  });
});

describe("clearPage", () => {
  it("chỉ xoá trang được chỉ định", () => {
    actions().addStroke(BOOK, 10, stroke("a"));
    actions().addStroke(BOOK, 11, stroke("b"));
    actions().clearPage(BOOK, 10);

    expect(strokesAt(10)).toBeUndefined();
    expect(strokesAt(11)?.map((s) => s.id)).toEqual(["b"]);
  });
});

describe("setColor", () => {
  it("đổi màu của đúng công cụ đang dùng, không đụng công cụ kia", () => {
    const before = useAnnotationStore.getState().highlighterColor;
    actions().setColor("#123456");

    expect(useAnnotationStore.getState().penColor).toBe("#123456");
    expect(useAnnotationStore.getState().highlighterColor).toBe(before);

    actions().setTool("highlighter");
    actions().setColor("#abcdef");

    expect(useAnnotationStore.getState().highlighterColor).toBe("#abcdef");
    expect(useAnnotationStore.getState().penColor).toBe("#123456");
  });
});

describe("activeWidth / paletteFor", () => {
  it("bút dạ quang dày hơn bút mực ở mọi cỡ", () => {
    for (let i = 0; i < PEN_WIDTHS.length; i++) {
      expect(activeWidth("highlighter", i)).toBeGreaterThan(activeWidth("pen", i));
    }
  });

  it("cỡ ngoài khoảng thì rơi về cỡ vừa", () => {
    expect(activeWidth("pen", 99)).toBe(PEN_WIDTHS[1]);
    expect(activeWidth("highlighter", -1)).toBe(HIGHLIGHTER_WIDTHS[1]);
  });

  it("tẩy dùng chung bảng màu của bút để thanh công cụ không nhảy", () => {
    expect(paletteFor("eraser")).toBe(paletteFor("pen"));
  });
});
