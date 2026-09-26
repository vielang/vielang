import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  GAP_MS,
  autoplayNext,
  autoplayPrev,
  continueAutoplay,
  restoreAutoplaySession,
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

/** Sách có bài nghe còn lại — dùng làm dữ liệu mẫu cho danh sách phát. */
const BOOK = "en-elementary";
const queue = buildPlaylist(BOOK);

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
    startAutoplay(BOOK, 2);
    expect(play).toHaveBeenCalledTimes(1);
    expect(el().getAttribute("src")).toBe(queue[2].url);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: BOOK, index: 2, status: "playing" });
  });

  it("hết bài thì nghỉ một nhịp rồi phát bài kế trên CÙNG thẻ audio", () => {
    startAutoplay(BOOK);
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
    startAutoplay(BOOK, queue.length - 1);
    el().dispatchEvent(new Event("ended"));
    vi.advanceTimersByTime(GAP_MS);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: null, status: "idle" });
  });

  it("trình duyệt chặn phát thì chuyển sang tạm dừng, bấm phát là chạy lại", async () => {
    playResult = () => Promise.reject(new DOMException("chặn", "NotAllowedError"));
    startAutoplay(BOOK);
    await vi.waitFor(() => expect(useAutoplayStore.getState().status).toBe("paused"));

    playResult = () => Promise.resolve();
    toggleAutoplayPause();
    expect(useAutoplayStore.getState().status).toBe("playing");
    expect(play).toHaveBeenCalledTimes(2);
  });

  it("tạm dừng lúc đang nghỉ giữa 2 bài thì không tự sang bài kế", () => {
    startAutoplay(BOOK);
    el().dispatchEvent(new Event("ended"));
    toggleAutoplayPause();
    vi.advanceTimersByTime(GAP_MS * 2);
    expect(useAutoplayStore.getState()).toMatchObject({ index: 0, status: "paused" });
  });

  it("lỗi tải thì thử lại, thua hết lượt thì bỏ qua sang bài sau", () => {
    startAutoplay(BOOK);
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
    startAutoplay(BOOK);
    autoplayNext();
    expect(useAutoplayStore.getState().index).toBe(1);
    autoplayPrev();
    autoplayPrev();
    expect(useAutoplayStore.getState().index).toBe(0);
  });
});

describe("nhớ chỗ nghe dở", () => {
  it("ghi lại bài đang nghe, tắt đi rồi vẫn còn", () => {
    startAutoplay(BOOK, 3);
    stopAutoplay();
    expect(useAutoplayResumeStore.getState().positions[BOOK]).toMatchObject({
      url: queue[3].url,
    });
  });

  it("nghe tiếp vào đúng bài và tua tới đúng giây", () => {
    useAutoplayResumeStore.setState({
      positions: { [BOOK]: { url: queue[5].url, time: 42 } },
    });
    continueAutoplay(BOOK);
    expect(useAutoplayStore.getState().index).toBe(5);
    el().dispatchEvent(new Event("loadedmetadata"));
    expect(el().currentTime).toBe(42);
  });

  it("chưa nghe lần nào, hoặc bài đã ghi không còn, thì nghe từ đầu", () => {
    continueAutoplay(BOOK);
    expect(useAutoplayStore.getState().index).toBe(0);
    stopAutoplay();

    useAutoplayResumeStore.setState({
      positions: { [BOOK]: { url: "https://x/khong-con.mp3", time: 10 } },
    });
    continueAutoplay(BOOK);
    expect(useAutoplayStore.getState().index).toBe(0);
  });

  it("nghe hết sách thì xoá chỗ dở, lần sau nghe lại từ đầu", () => {
    startAutoplay(BOOK, queue.length - 1);
    el().dispatchEvent(new Event("ended"));
    vi.advanceTimersByTime(GAP_MS);
    expect(useAutoplayResumeStore.getState().positions[BOOK]).toBeUndefined();
  });
});

describe("các ca dễ hỏng", () => {
  it("đang chờ tua tới chỗ dở thì không ghi đè chỗ dở bằng giây 0", () => {
    useAutoplayResumeStore.setState({
      positions: { [BOOK]: { url: queue[5].url, time: 42 } },
    });
    continueAutoplay(BOOK);
    // Trình duyệt đặt lại về 0 khi đổi `src` và bắn timeupdate/pause.
    el().dispatchEvent(new Event("timeupdate"));
    stopAutoplay();
    expect(useAutoplayResumeStore.getState().positions[BOOK]).toEqual({
      url: queue[5].url,
      time: 42,
    });
  });

  it("lỗi tải ngay lúc đang chờ tua thì thử lại vẫn tua tới đúng chỗ dở", () => {
    useAutoplayResumeStore.setState({
      positions: { [BOOK]: { url: queue[5].url, time: 42 } },
    });
    continueAutoplay(BOOK);
    el().dispatchEvent(new Event("error"));
    vi.advanceTimersByTime(retryDelay(0));
    el().dispatchEvent(new Event("loadedmetadata"));
    expect(el().currentTime).toBe(42);
  });

  it("mất mạng thì tạm dừng; có mạng lại bấm phát là nạp lại bài chứ không kẹt", () => {
    startAutoplay(BOOK);
    const online = vi.spyOn(navigator, "onLine", "get").mockReturnValue(false);
    el().dispatchEvent(new Event("error"));
    expect(useAutoplayStore.getState().status).toBe("paused");

    online.mockReturnValue(true);
    // jsdom không có `error` trên thẻ media — gắn tạm như trình duyệt thật.
    Object.defineProperty(el(), "error", { configurable: true, value: { code: 2 } });
    toggleAutoplayPause();
    expect(load).toHaveBeenCalledTimes(1);
    expect(useAutoplayStore.getState().status).toBe("playing");
    online.mockRestore();
    delete (el() as { error?: unknown }).error;
  });

  it("bấm lùi lúc đang nghỉ giữa 2 bài thì phát lại bài vừa rồi, không nhảy sang bài kế", () => {
    startAutoplay(BOOK, 2);
    Object.defineProperty(el(), "currentTime", { configurable: true, writable: true, value: 120 });
    el().dispatchEvent(new Event("ended"));
    autoplayPrev();
    vi.advanceTimersByTime(GAP_MS * 2);
    expect(useAutoplayStore.getState().index).toBe(2);
    expect(el().currentTime).toBe(0);
    expect(play).toHaveBeenCalledTimes(2);
    // Thẻ audio dùng chung cho mọi test — gỡ giá trị giả ra.
    delete (el() as { currentTime?: number }).currentTime;
  });
});

describe("sống sót qua một lần tải lại cả trang", () => {
  /** Giả lập trang vừa tải lại: bộ nhớ sạch, chỉ còn sessionStorage/localStorage. */
  function simulateReload() {
    // Tải lại thật thì module chạy lại từ đầu, không có lượt "chuyển sang
    // idle" nào để xoá dấu — giữ dấu lại qua bước đưa store về ban đầu.
    const kept = sessionStorage.getItem("kiip-autoplay-session");
    useAutoplayStore.setState({ bookId: null, queue: [], index: 0, status: "idle" });
    if (kept) sessionStorage.setItem("kiip-autoplay-session", kept);
  }

  function session() {
    const raw = sessionStorage.getItem("kiip-autoplay-session");
    return raw ? JSON.parse(raw) : null;
  }

  it("đang nghe thì ghi dấu vào tab, tắt thì xoá dấu", () => {
    startAutoplay(BOOK, 3);
    expect(session()).toMatchObject({ bookId: BOOK, paused: false });
    toggleAutoplayPause();
    expect(session()).toMatchObject({ paused: true });
    stopAutoplay();
    expect(session()).toBeNull();
  });

  it("tải lại giữa chừng thì nghe tiếp đúng bài, đúng giây", () => {
    startAutoplay(BOOK, 3);
    useAutoplayResumeStore.setState({
      positions: { [BOOK]: { url: queue[4].url, time: 17 } },
    });
    simulateReload();
    play.mockClear();

    restoreAutoplaySession(`/read/${BOOK}/${queue[4].page}`);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: BOOK, index: 4, status: "playing" });
    expect(play).toHaveBeenCalledTimes(1);
    el().dispatchEvent(new Event("loadedmetadata"));
    expect(el().currentTime).toBe(17);
  });

  it("trình duyệt chặn tự phát sau tải lại thì vẫn giữ thanh, ở trạng thái chờ bấm", async () => {
    startAutoplay(BOOK, 3);
    simulateReload();
    playResult = () => Promise.reject(new DOMException("chặn", "NotAllowedError"));

    restoreAutoplaySession(`/read/${BOOK}/${queue[3].page}`);
    await vi.waitFor(() => expect(useAutoplayStore.getState().status).toBe("paused"));
    expect(useAutoplayStore.getState().bookId).toBe(BOOK);
  });

  it("đang tạm dừng lúc tải lại thì khôi phục ở trạng thái dừng, không tự phát", () => {
    startAutoplay(BOOK, 3);
    toggleAutoplayPause();
    simulateReload();
    play.mockClear();

    restoreAutoplaySession(`/read/${BOOK}/${queue[3].page}`);
    expect(useAutoplayStore.getState()).toMatchObject({ bookId: BOOK, status: "paused" });
    expect(play).not.toHaveBeenCalled();
  });

  it("quay lại tab sau lâu quá thì không tự phát tiếng", () => {
    startAutoplay(BOOK, 3);
    vi.advanceTimersByTime(10 * 60 * 1000);
    simulateReload();
    play.mockClear();

    restoreAutoplaySession(`/read/${BOOK}/${queue[3].page}`);
    expect(useAutoplayStore.getState().status).toBe("paused");
    expect(play).not.toHaveBeenCalled();
  });

  it("không ở trình đọc của sách đó thì không khôi phục, và bỏ dấu cũ", () => {
    startAutoplay(BOOK, 3);
    simulateReload();

    restoreAutoplaySession(`/books/${BOOK}`);
    expect(useAutoplayStore.getState().status).toBe("idle");
    expect(session()).toBeNull();
  });
});
