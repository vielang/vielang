"use client";

import { create } from "zustand";
import { buildPlaylist, type AutoplayItem } from "@/lib/autoplay";
import { MAX_AUTO_RETRIES, retryDelay } from "@/lib/use-retrying-media";

/**
 * Nghe tự động cả sách: phát lần lượt mọi bài nghe, hết bài thì tự lật tới
 * trang có bài kế tiếp (phần lật trang ở `AutoplayFollower`).
 *
 * Điểm mấu chốt: MỘT thẻ audio duy nhất cho cả phiên nghe, sống ở cấp module
 * chứ không nằm trong component nào.
 *
 * `ReaderView` bị dựng lại mỗi lần lật trang (`key={page}`), kéo theo thẻ
 * <audio> của `AudioWidget`. Thẻ mới dựng không thừa hưởng lần chạm đã mở
 * khoá phát tiếng, nên `play()` trên nó bị Safari iOS chặn — mà đây lại đúng
 * là lúc không có ai chạm, vì bài trước vừa hết. Còn một thẻ đã được mở khoá
 * bằng lần bấm "Nghe tự động" thì đổi `src` bao nhiêu lần cũng phát được.
 *
 * Hệ quả phụ có lợi: tắt màn hình điện thoại vẫn nghe tiếp, và điều khiển
 * được từ màn hình khoá (Media Session).
 *
 * Vì thế `startAutoplay` phải được gọi THẲNG trong hàm xử lý click, không
 * qua effect hay hẹn giờ — ra khỏi lượt xử lý lần chạm là mất quyền phát.
 */

export type AutoplayStatus = "idle" | "playing" | "paused";

interface AutoplayState {
  bookId: string | null;
  queue: AutoplayItem[];
  /** Bài đang phát trong `queue`. */
  index: number;
  status: AutoplayStatus;
  /**
   * Trang trình đọc đang hiện (1 hoặc 2 trang) và có đang ở chế độ 2 trang
   * không — `ReaderView` báo lên, để biết bài kế tiếp có cần lật trang không.
   */
  visiblePages: number[];
  double: boolean;
  setView: (pages: number[], double: boolean) => void;
}

export const useAutoplayStore = create<AutoplayState>((set) => ({
  bookId: null,
  queue: [],
  index: 0,
  status: "idle",
  visiblePages: [],
  double: false,
  setView: (visiblePages, double) => set({ visiblePages, double }),
}));

/** Nghỉ giữa 2 bài — liền tù tì thì chưa kịp định thần đã sang bài khác. */
export const GAP_MS = 1500;

let audio: HTMLAudioElement | null = null;
let gapTimer: ReturnType<typeof setTimeout> | null = null;
let retryTimer: ReturnType<typeof setTimeout> | null = null;
let retries = 0;

function clearTimers() {
  if (gapTimer) clearTimeout(gapTimer);
  if (retryTimer) clearTimeout(retryTimer);
  gapTimer = retryTimer = null;
}

function element(): HTMLAudioElement {
  if (audio) return audio;
  const a = new Audio();
  a.preload = "auto";
  a.addEventListener("ended", onEnded);
  a.addEventListener("error", onError);
  // Hai sự kiện dưới để trạng thái khớp khi người dùng dừng/phát từ ngoài
  // app: màn hình khoá, tai nghe, trung tâm điều khiển.
  a.addEventListener("play", () => {
    if (useAutoplayStore.getState().status === "paused") {
      useAutoplayStore.setState({ status: "playing" });
    }
  });
  a.addEventListener("pause", () => {
    // Hết bài cũng bắn `pause` — đó không phải người dùng bấm dừng.
    if (a.ended) return;
    if (useAutoplayStore.getState().status === "playing") {
      useAutoplayStore.setState({ status: "paused" });
    }
  });
  audio = a;
  return a;
}

function play(a: HTMLAudioElement) {
  a.play().catch((err: unknown) => {
    // Trình duyệt chặn phát (mất quyền do thao tác chạm): chuyển sang tạm
    // dừng để người dùng bấm phát lại một cái — lần bấm đó mở khoá lại.
    // AbortError là do đổi `src` giữa chừng, bài mới sẽ tự phát, bỏ qua.
    if (err instanceof DOMException && err.name === "NotAllowedError") {
      useAutoplayStore.setState({ status: "paused" });
    }
  });
}

function playIndex(i: number) {
  clearTimers();
  const { queue } = useAutoplayStore.getState();
  if (i < 0 || i >= queue.length) {
    stopAutoplay();
    return;
  }
  retries = 0;
  useAutoplayStore.setState({ index: i, status: "playing" });
  const a = element();
  a.src = queue[i].url;
  play(a);
  updateMediaSession();
}

function onEnded() {
  const { index, status } = useAutoplayStore.getState();
  if (status === "idle") return;
  // Màn hình đang tắt: phát ngay, không nghỉ. Trong khoảng im lặng iOS có
  // thể treo trang, hẹn giờ không bao giờ tới — thế là dừng hẳn giữa sách.
  if (document.hidden) {
    playIndex(index + 1);
    return;
  }
  gapTimer = setTimeout(() => playIndex(index + 1), GAP_MS);
}

function onError() {
  const { index, status } = useAutoplayStore.getState();
  if (status === "idle" || !audio) return;
  // Mất mạng hẳn thì đừng nhảy bài — nhảy tới đâu cũng hỏng, đến lúc có
  // mạng lại thì đã trôi tới cuối sách. Dừng lại chờ người dùng.
  if (!navigator.onLine) {
    useAutoplayStore.setState({ status: "paused" });
    return;
  }
  if (retries < MAX_AUTO_RETRIES) {
    const a = audio;
    retryTimer = setTimeout(() => {
      a.load();
      play(a);
    }, retryDelay(retries));
    retries++;
    return;
  }
  // Bài này hỏng thật: bỏ qua, nghe tiếp bài sau.
  playIndex(index + 1);
}

/** Bật nghe tự động — gọi thẳng trong hàm xử lý click (xem đầu file). */
export function startAutoplay(bookId: string, fromIndex = 0) {
  const state = useAutoplayStore.getState();
  const queue = state.bookId === bookId && state.queue.length > 0 ? state.queue : buildPlaylist(bookId);
  if (queue.length === 0) return;
  useAutoplayStore.setState({ bookId, queue });
  setMediaSessionHandlers();
  playIndex(Math.min(Math.max(fromIndex, 0), queue.length - 1));
}

export function stopAutoplay() {
  clearTimers();
  if (audio) {
    audio.pause();
    audio.removeAttribute("src");
    audio.load();
  }
  useAutoplayStore.setState({ bookId: null, queue: [], index: 0, status: "idle" });
  if (typeof navigator !== "undefined" && "mediaSession" in navigator) {
    navigator.mediaSession.metadata = null;
  }
}

export function pauseAutoplay() {
  if (useAutoplayStore.getState().status !== "playing" || !audio) return;
  clearTimers();
  audio.pause();
  useAutoplayStore.setState({ status: "paused" });
}

export function resumeAutoplay() {
  const { status, index } = useAutoplayStore.getState();
  if (status !== "paused" || !audio) return;
  // Bấm dừng đúng lúc đang nghỉ giữa 2 bài: bài cũ đã hết, phát tiếp là sang bài kế.
  if (audio.ended) {
    playIndex(index + 1);
    return;
  }
  useAutoplayStore.setState({ status: "playing" });
  play(audio);
}

export function toggleAutoplayPause() {
  if (useAutoplayStore.getState().status === "playing") pauseAutoplay();
  else resumeAutoplay();
}

export function autoplayNext() {
  playIndex(useAutoplayStore.getState().index + 1);
}

export function autoplayPrev() {
  const { index } = useAutoplayStore.getState();
  // Như mọi trình phát nhạc: đã nghe được một đoạn thì "lùi" là về đầu bài.
  if (audio && audio.currentTime > 3) {
    audio.currentTime = 0;
    return;
  }
  playIndex(Math.max(index - 1, 0));
}

function updateMediaSession() {
  if (!("mediaSession" in navigator) || typeof MediaMetadata === "undefined") return;
  const { queue, index } = useAutoplayStore.getState();
  const item = queue[index];
  if (!item) return;
  navigator.mediaSession.metadata = new MediaMetadata({
    title: item.label,
    artist: `Trang ${item.page} · ${index + 1}/${queue.length}`,
    album: "VieTopik",
  });
}

function setMediaSessionHandlers() {
  if (!("mediaSession" in navigator)) return;
  const handlers: [MediaSessionAction, () => void][] = [
    ["play", resumeAutoplay],
    ["pause", pauseAutoplay],
    ["nexttrack", autoplayNext],
    ["previoustrack", autoplayPrev],
  ];
  for (const [action, handler] of handlers) {
    try {
      navigator.mediaSession.setActionHandler(action, handler);
    } catch {
      // Trình duyệt cũ không hỗ trợ action này — bỏ qua.
    }
  }
}
