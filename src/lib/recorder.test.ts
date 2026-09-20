import { afterEach, describe, expect, it, vi } from "vitest";
import {
  RECORDER_ERROR_MESSAGE,
  pickMimeType,
  startRecording,
  toRecorderError,
} from "./recorder";

/**
 * Dựng tối thiểu `getUserMedia` + `MediaRecorder` — jsdom không có cả hai.
 * Trả về các spy để kiểm chứng điều quan trọng nhất: MICRO CÓ ĐƯỢC NHẢ
 * KHÔNG. Quên nhả là đèn "đang ghi âm" của trình duyệt sáng mãi.
 */
function mockMediaStack() {
  const trackStop = vi.fn();
  const stream = { getTracks: () => [{ stop: trackStop }, { stop: trackStop }] };
  const getUserMedia = vi.fn().mockResolvedValue(stream);
  let options: MediaRecorderOptions | undefined;
  let constraints: MediaStreamConstraints | undefined;

  getUserMedia.mockImplementation((c: MediaStreamConstraints) => {
    constraints = c;
    return Promise.resolve(stream);
  });

  class FakeRecorder {
    /** `pickMimeType` hỏi tới hàm này ngay khi dựng recorder. */
    static isTypeSupported = (t: string) => t === "audio/webm;codecs=opus";
    state = "recording";
    mimeType = "audio/webm;codecs=opus";
    private listeners: Record<string, ((e: unknown) => void)[]> = {};
    constructor(_stream: unknown, opts?: MediaRecorderOptions) {
      options = opts;
    }
    addEventListener(type: string, fn: (e: unknown) => void) {
      (this.listeners[type] ??= []).push(fn);
    }
    start() {}
    stop() {
      this.state = "inactive";
      for (const fn of this.listeners.dataavailable ?? []) {
        fn({ data: new Blob(["âm"], { type: this.mimeType }) });
      }
      for (const fn of this.listeners.stop ?? []) fn({});
    }
  }

  vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });
  vi.stubGlobal("MediaRecorder", FakeRecorder);
  return {
    trackStop,
    getOptions: () => options,
    getConstraints: () => constraints,
  };
}

afterEach(() => vi.unstubAllGlobals());

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

describe("startRecording", () => {
  it("thu 1 kênh", async () => {
    const m = mockMediaStack();
    await startRecording();

    const audio = m.getConstraints()!.audio as MediaTrackConstraints;
    expect(audio.channelCount).toEqual({ ideal: 1 });
  });

  it("TẮT khử ồn để giữ nguyên âm xát tiếng Hàn", async () => {
    // Khác mặc định của Chrome lẫn Safari, nên rất dễ bị ai đó "dọn" lại cho
    // giống thông thường — chốt lại ở đây kèm lý do.
    const m = mockMediaStack();
    await startRecording();

    const audio = m.getConstraints()!.audio as MediaTrackConstraints;
    expect(audio.noiseSuppression).toEqual({ ideal: false });
  });

  it("vẫn giữ khử vọng và tự chỉnh âm lượng", async () => {
    const m = mockMediaStack();
    await startRecording();

    const audio = m.getConstraints()!.audio as MediaTrackConstraints;
    expect(audio.echoCancellation).toEqual({ ideal: true });
    expect(audio.autoGainControl).toEqual({ ideal: true });
  });

  it("đặt bitrate ngay lúc thu, không để mặc định 128 kbps", () => {
    // Chỉnh ở đây thì không phải nén lại về sau — mà nén lại một luồng đã
    // nén là mất mát thêm một lần nữa.
    const m = mockMediaStack();
    return startRecording().then(() => {
      expect(m.getOptions()?.audioBitsPerSecond).toBe(32_000);
    });
  });

  it("dừng thu thì trả về tiếng VÀ nhả micro", async () => {
    const m = mockMediaStack();
    const session = await startRecording();
    const { blob, mimeType } = await session.stop();

    expect(await blob.text()).toBe("âm");
    // Kiểu THẬT trình duyệt cho ra, không phải kiểu mình xin.
    expect(mimeType).toBe("audio/webm;codecs=opus");
    expect(m.trackStop).toHaveBeenCalledTimes(2);
  });

  it("dừng hai lần vẫn ra cùng một bản ghi, không treo", async () => {
    mockMediaStack();
    const session = await startRecording();
    const first = await session.stop();
    const second = await session.stop();

    expect(second).toBe(first);
  });

  it("huỷ giữa chừng cũng nhả micro", async () => {
    const m = mockMediaStack();
    const session = await startRecording();
    session.cancel();

    expect(m.trackStop).toHaveBeenCalledTimes(2);
  });
});
