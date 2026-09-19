import { beforeEach, describe, expect, it } from "vitest";
import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import type { BinaryFileData, BinaryFiles } from "@excalidraw/excalidraw/types";
import { drawKey, useDrawStore } from "./draw-store";

const BOOK = "step1";

/** Element thật của Excalidraw có ~25 trường; test chỉ đụng tới mấy trường này. */
function element(id: string, extra: Partial<ExcalidrawElement> = {}): ExcalidrawElement {
  return { id, isDeleted: false, ...extra } as ExcalidrawElement;
}

function image(id: string, fileId: string, isDeleted = false): ExcalidrawElement {
  return { id, isDeleted, type: "image", fileId } as unknown as ExcalidrawElement;
}

function file(id: string): BinaryFileData {
  return { id, dataURL: `data:image/png;base64,${id}`, mimeType: "image/png" } as BinaryFileData;
}

function drawingAt(page: number) {
  return useDrawStore.getState().drawings[drawKey(BOOK, page)];
}

beforeEach(() => {
  localStorage.clear();
  useDrawStore.setState({ drawings: {}, hasHydrated: true, quotaExceeded: false });
});

describe("saveDrawing", () => {
  it("lưu bản vẽ theo từng trang, không lẫn sang trang khác", () => {
    useDrawStore.getState().saveDrawing(BOOK, 10, [element("a")], {}, null);
    useDrawStore.getState().saveDrawing(BOOK, 11, [element("b")], {}, null);

    expect(drawingAt(10).elements.map((e) => e.id)).toEqual(["a"]);
    expect(drawingAt(11).elements.map((e) => e.id)).toEqual(["b"]);
  });

  it("bỏ element đã xoá — undo là việc của phiên đang mở, không cần lưu", () => {
    useDrawStore
      .getState()
      .saveDrawing(BOOK, 10, [element("a"), element("b", { isDeleted: true })], {}, null);

    expect(drawingAt(10).elements.map((e) => e.id)).toEqual(["a"]);
  });

  it("chỉ giữ ảnh còn được element dùng", () => {
    const files: BinaryFiles = { f1: file("f1"), f2: file("f2") };
    useDrawStore
      .getState()
      .saveDrawing(BOOK, 10, [image("i1", "f1"), image("i2", "f2", true)], files, null);

    expect(Object.keys(drawingAt(10).files)).toEqual(["f1"]);
  });

  it("xoá hẳn bản vẽ khi không còn element nào", () => {
    useDrawStore.getState().saveDrawing(BOOK, 10, [element("a")], {}, null);
    useDrawStore.getState().saveDrawing(BOOK, 10, [element("a", { isDeleted: true })], {}, null);

    expect(drawingAt(10)).toBeUndefined();
  });

  it("giữ nguyên state khi xoá bản vẽ vốn không tồn tại", () => {
    const before = useDrawStore.getState().drawings;
    useDrawStore.getState().saveDrawing(BOOK, 10, [], {}, null);

    expect(useDrawStore.getState().drawings).toBe(before);
  });

  it("nhớ màu nền canvas", () => {
    useDrawStore.getState().saveDrawing(BOOK, 10, [element("a")], {}, "#fff8e1");

    expect(drawingAt(10).background).toBe("#fff8e1");
  });
});

describe("clearDrawing", () => {
  it("xoá bản vẽ của đúng trang được chỉ định", () => {
    useDrawStore.getState().saveDrawing(BOOK, 10, [element("a")], {}, null);
    useDrawStore.getState().saveDrawing(BOOK, 11, [element("b")], {}, null);
    useDrawStore.getState().clearDrawing(BOOK, 10);

    expect(drawingAt(10)).toBeUndefined();
    expect(drawingAt(11)).toBeDefined();
  });
});
