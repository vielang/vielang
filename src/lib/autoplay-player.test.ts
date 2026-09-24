import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  GAP_MS,
  autoplayNext,
  autoplayPrev,
  continueAutoplay,
  startAutoplay,
  stopAutoplay,
  toggleAutoplayPause,
  useAutoplayResumeStore,
  useAutoplayStore,
} from "./autoplay-player";
import { buildPlaylist } from "./autoplay";
import { MAX_AUTO_RETRIES, retryDelay } from "./use-retrying-media";

/** Thẻ audio nằm kín trong module — bắt nó qua lần gọi `play()` đầu tiên. */
let played: HTMLMediaElement[] = [];
let playResult: () => Promise<void> = () => Promise.resolve();

const play = vi.spyOn(HTMLMediaElement.prototype, "play");
const load = vi.spyOn(HTMLMediaElement.prototype, "load");
vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});

function el(): HTMLMediaElement {
  return played[played.length - 1];
}

const queue = buildPlaylist("step1");

beforeEach(() => {
  vi.useFakeTimers();
  played = [];
  playResult = () => Promise.resolve();
  play.mockImplementation(function (this: HTMLMediaElement) {
    played.push(this);
    return playResult();
  });
  load.mockImplementation(() => {});
  useAutoplayResumeStore.setState({ positions: {} });
});

afterEach(() => {
  stopAutoplay();
  vi.useRealTimers();
});

describe("nghe tự động", () => {
  it("phát ngay bài được chọn, trong đúng lượt gọi", () => {
    startAutoplay("step1", 2);
    expect(play).toHaveBeenCalledTimes(1);
    expect(el().getAttribute("src")).toBe(queue[2].url);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: "step1", index: 2, status: "playing" });
  });

  it("hết bài thì nghỉ một nhịp rồi phát bài kế trên CÙNG thẻ audio", () => {
    startAutoplay("step1");
    const first = el();
    first.dispatchEvent(new Event("ended"));
    expect(play).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(GAP_MS);
    expect(play).toHaveBeenCalledTimes(2);
    // Thẻ mới thì Safari iOS chặn phát — phải là đúng thẻ đã được mở khoá.
    expect(el()).toBe(first);
    expect(el().getAttribute("src")).toBe(queue[1].url);
    expect(useAutoplayStore.getState().index).toBe(1);
  });

  it("hết bài cuối thì tắt hẳn", () => {
    startAutoplay("step1", queue.length - 1);
    el().dispatchEvent(new Event("ended"));
    vi.advanceTimersByTime(GAP_MS);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: null, status: "idle" });
  });

  it("trình duyệt chặn phát thì chuyển sang tạm dừng, bấm phát là chạy lại", async () => {
    playResult = () => Promise.reject(new DOMException("chặn", "NotAllowedError"));
    startAutoplay("step1");
    await vi.waitFor(() => expect(useAutoplayStore.getState().status).toBe("paused"));

    playResult = () => Promise.resolve();
    toggleAutoplayPause();
    expect(useAutoplayStore.getState().status).toBe("playing");
    expect(play).toHaveBeenCalledTimes(2);
  });

  it("tạm dừng lúc đang nghỉ giữa 2 bài thì không tự sang bài kế", () => {
    startAutoplay("step1");
    el().dispatchEvent(new Event("ended"));
    toggleAutoplayPause();
    vi.advanceTimersByTime(GAP_MS * 2);
    expect(useAutoplayStore.getState()).toMatchObject({ index: 0, status: "paused" });
  });

  it("lỗi tải thì thử lại, thua hết lượt thì bỏ qua sang bài sau", () => {
    startAutoplay("step1");
    for (let i = 0; i < MAX_AUTO_RETRIES; i++) {
      el().dispatchEvent(new Event("error"));
      vi.advanceTimersByTime(retryDelay(i));
    }
    expect(load).toHaveBeenCalledTimes(MAX_AUTO_RETRIES);
    expect(useAutoplayStore.getState().index).toBe(0);

    el().dispatchEvent(new Event("error"));
    expect(useAutoplayStore.getState().index).toBe(1);
  });

  it("tới/lùi đổi bài, lùi ở bài đầu thì đứng yên", () => {
    startAutoplay("step1");
    autoplayNext();
    expect(useAutoplayStore.getState().index).toBe(1);
    autoplayPrev();
    autoplayPrev();
    expect(useAutoplayStore.getState().index).toBe(0);
  });
});

describe("nhớ chỗ nghe dở", () => {
  it("ghi lại bài đang nghe, tắt đi rồi vẫn còn", () => {
    startAutoplay("step1", 3);
    stopAutoplay();
    expect(useAutoplayResumeStore.getState().positions.step1).toMatchObject({
      url: queue[3].url,
    });
  });

  it("nghe tiếp vào đúng bài và tua tới đúng giây", () => {
    useAutoplayResumeStore.setState({
      positions: { step1: { url: queue[5].url, time: 42 } },
    });
    continueAutoplay("step1");
    expect(useAutoplayStore.getState().index).toBe(5);
    el().dispatchEvent(new Event("loadedmetadata"));
    expect(el().currentTime).toBe(42);
  });

  it("chưa nghe lần nào, hoặc bài đã ghi không còn, thì nghe từ đầu", () => {
    continueAutoplay("step1");
    expect(useAutoplayStore.getState().index).toBe(0);
    stopAutoplay();

    useAutoplayResumeStore.setState({
      positions: { step1: { url: "https://x/khong-con.mp3", time: 10 } },
    });
    continueAutoplay("step1");
    expect(useAutoplayStore.getState().index).toBe(0);
  });

  it("nghe hết sách thì xoá chỗ dở, lần sau nghe lại từ đầu", () => {
    startAutoplay("step1", queue.length - 1);
    el().dispatchEvent(new Event("ended"));
    vi.advanceTimersByTime(GAP_MS);
    expect(useAutoplayResumeStore.getState().positions.step1).toBeUndefined();
  });
});
