import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  formatDuration,
  recordingKey,
  recordingLabel,
  useRecordingStore,
  type Recording,
} from "./recording-store";

/** `removeRecording` gọi `deleteBlob` — theo dõi để chắc tiếng cũng bị dọn. */
const deleteBlob = vi.hoisted(() => vi.fn());
vi.mock("./idb-storage", () => ({
  createIdbStorage: () => ({
    getItem: async () => null,
    setItem: async () => {},
    removeItem: async () => {},
  }),
  deleteBlob,
}));

const BOOK = "step1";
const actions = () => useRecordingStore.getState();

function rec(id: string, label = ""): Recording {
  return {
    id,
    label,
    durationMs: 3000,
    mimeType: "audio/webm",
    createdAt: "2026-09-20T00:00:00.000Z",
  };
}

const listAt = (page: number) =>
  useRecordingStore.getState().recordings[recordingKey(BOOK, page)];

beforeEach(() => {
  deleteBlob.mockClear();
  useRecordingStore.setState({
    recordings: {},
    hasHydrated: true,
    quotaExceeded: false,
    open: false,
    panelPos: null,
  });
});

describe("addRecording", () => {
  it("giữ bản ghi theo từng trang, không lẫn sang trang khác", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    actions().addRecording(BOOK, 11, rec("b"));

    expect(listAt(10)?.map((r) => r.id)).toEqual(["a"]);
    expect(listAt(11)?.map((r) => r.id)).toEqual(["b"]);
  });

  it("giữ đúng thứ tự ghi", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    actions().addRecording(BOOK, 10, rec("b"));

    expect(listAt(10)?.map((r) => r.id)).toEqual(["a", "b"]);
  });
});

describe("removeRecording", () => {
  it("xoá cả tiếng chứ không riêng mục trong danh sách", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    actions().removeRecording(BOOK, 10, "a");

    expect(listAt(10)).toBeUndefined();
    expect(deleteBlob).toHaveBeenCalledWith("a");
  });

  it("chỉ xoá đúng bản được chỉ định", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    actions().addRecording(BOOK, 10, rec("b"));
    actions().removeRecording(BOOK, 10, "a");

    expect(listAt(10)?.map((r) => r.id)).toEqual(["b"]);
  });

  it("không đụng gì khi id không tồn tại", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    const before = useRecordingStore.getState().recordings;
    actions().removeRecording(BOOK, 10, "khong-co");

    expect(useRecordingStore.getState().recordings).toBe(before);
    expect(deleteBlob).not.toHaveBeenCalled();
  });
});

describe("renameRecording", () => {
  it("đổi tên đúng bản, giữ nguyên các bản khác", () => {
    actions().addRecording(BOOK, 10, rec("a"));
    actions().addRecording(BOOK, 10, rec("b"));
    actions().renameRecording(BOOK, 10, "b", "Đọc lần 2");

    expect(listAt(10)?.map((r) => r.label)).toEqual(["", "Đọc lần 2"]);
  });
});

describe("recordingLabel", () => {
  it("chưa đặt tên thì đánh số theo thứ tự trong trang", () => {
    expect(recordingLabel(rec("a"), 0)).toBe("Bản ghi 1");
    expect(recordingLabel(rec("a"), 2)).toBe("Bản ghi 3");
  });

  it("tên chỉ có khoảng trắng cũng coi như chưa đặt", () => {
    expect(recordingLabel(rec("a", "   "), 0)).toBe("Bản ghi 1");
  });

  it("có tên thì dùng tên", () => {
    expect(recordingLabel(rec("a", "Đọc lần 1"), 0)).toBe("Đọc lần 1");
  });
});

describe("formatDuration", () => {
  it("đệm 0 cho phần giây", () => {
    expect(formatDuration(7000)).toBe("0:07");
    expect(formatDuration(83_000)).toBe("1:23");
  });

  it("làm tròn tới giây gần nhất", () => {
    expect(formatDuration(1600)).toBe("0:02");
  });

  it("không ra số âm", () => {
    expect(formatDuration(-5000)).toBe("0:00");
  });
});
