import { describe, expect, it } from "vitest";
import {
  RECORDER_ERROR_MESSAGE,
  pickMimeType,
  toRecorderError,
} from "./recorder";

describe("pickMimeType", () => {
  it("ưu tiên opus trong webm khi trình duyệt nhận", () => {
    expect(pickMimeType(() => true)).toBe("audio/webm;codecs=opus");
  });

  it("rơi về mp4 trên Safari — nơi webm không được nhận", () => {
    expect(pickMimeType((t) => t === "audio/mp4")).toBe("audio/mp4");
  });

  it("không nhận kiểu nào thì để trình duyệt tự chọn", () => {
    // `undefined` chứ không phải ép một kiểu: ép sai là `new MediaRecorder`
    // ném lỗi ngay, còn để trống thì nó tự lo.
    expect(pickMimeType(() => false)).toBeUndefined();
  });
});

describe("toRecorderError", () => {
  it("người dùng từ chối quyền", () => {
    expect(toRecorderError({ name: "NotAllowedError" })).toBe("denied");
    expect(toRecorderError({ name: "SecurityError" })).toBe("denied");
  });

  it("máy không có micro", () => {
    expect(toRecorderError({ name: "NotFoundError" })).toBe("unsupported");
  });

  it("lỗi lạ thì vẫn ra một lý do nói được", () => {
    expect(toRecorderError(new Error("bể"))).toBe("failed");
    expect(toRecorderError(null)).toBe("failed");
  });

  it("lý do nào cũng có câu nói cho người dùng", () => {
    for (const reason of ["denied", "unsupported", "failed"] as const) {
      expect(RECORDER_ERROR_MESSAGE[reason]).toBeTruthy();
    }
  });
});
