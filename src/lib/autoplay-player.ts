"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { buildPlaylist, type AutoplayItem } from "@/lib/autoplay";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";
import type { DragPos } from "@/lib/use-draggable";
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
  /**
   * Lật tới trang `dest` bằng hiệu ứng lật của trình đọc — `ReaderView` gắn
   * vào khi đang mở. Trả false khi `dest` không liền kề trang đang xem, lúc
   * đó `AutoplayFollower` tự nhảy trang bằng router.
   */
  turnTo: ((dest: number) => boolean) | null;
  setTurnTo: (fn: ((dest: number) => boolean) | null) => void;
  /**
   * Chỗ đứng của thanh điều khiển. Nằm ở store cấp module để sống qua các
   * lần `ReaderView` dựng lại khi lật trang — cố ý không lưu localStorage,
   * cùng lý do với `audio-widget-store`.
   */
  barPos: DragPos | null;
  /** Người dùng đã tự kéo thanh đi chưa — chưa thì thanh còn bám chỗ mặc định. */
  barMoved: boolean;
  setBarPos: (pos: DragPos, moved: boolean) => void;
}

export const useAutoplayStore = create<AutoplayState>((set) => ({
  bookId: null,
  queue: [],
  index: 0,
  status: "idle",
  visiblePages: [],
  double: false,
  setView: (visiblePages, double) => set({ visiblePages, double }),
  turnTo: null,
  setTurnTo: (turnTo) => set({ turnTo }),
  barPos: null,
  barMoved: false,
  setBarPos: (barPos, barMoved) => set({ barPos, barMoved }),
}));

/** Chỗ đang nghe dở của 1 sách: bài nào (theo URL) và tới giây thứ mấy. */
export interface ResumePosition {
  /**
   * Theo URL chứ không theo số thứ tự: sau này bổ sung bài nghe cho trang
   * nào đó thì số thứ tự dịch đi hết, còn URL của bài thì vẫn thế.
   */
  url: string;
  time: number;
}

interface AutoplayResumeState {
  positions: Record<string, ResumePosition>;
  hasHydrated: boolean;
  setHasHydrated: (v: boolean) => void;
}

/**
 * Chỗ nghe dở của từng sách, để hôm sau bấm "Nghe tiếp" là vào đúng bài,
 * đúng giây. Nghe hết sách thì xoá, lần sau lại nghe từ đầu.
 */
export const useAutoplayResumeStore = create<AutoplayResumeState>()(
  persist(
    (set) => ({
      positions: {},
      hasHydrated: false,
      setHasHydrated: (hasHydrated) => set({ hasHydrated }),
    }),
    {
      name: "kiip-autoplay-v1",
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ positions: s.positions }),
      onRehydrateStorage: () => (state) => {
        state?.setHasHydrated(true);
      },
    }
  )
);

syncAcrossTabs(useAutoplayResumeStore);

function savePosition(bookId: string, position: ResumePosition | null) {
  useAutoplayResumeStore.setState((s) => {
    const positions = { ...s.positions };
    if (position) positions[bookId] = position;
    else delete positions[bookId];
    return { positions };
  });
}

/** Chỗ nghe dở của sách, quy ra vị trí trong danh sách phát hiện tại. */
export function findResumePoint(
  queue: AutoplayItem[],
  position: ResumePosition | undefined
): { index: number; time: number } | null {
  if (!position) return null;
  const index = queue.findIndex((item) => item.url === position.url);
  return index === -1 ? null : { index, time: position.time };
}

/** Nghỉ giữa 2 bài — liền tù tì thì chưa kịp định thần đã sang bài khác. */
export const GAP_MS = 1500;

let audio: HTMLAudioElement | null = null;
let gapTimer: ReturnType<typeof setTimeout> | null = null;
let retryTimer: ReturnType<typeof setTimeout> | null = null;
let retries = 0;
/** Giây cần tua tới khi bài vừa đổi `src` nạp xong (nghe tiếp, thử lại sau lỗi). */
let pendingSeek = 0;
/** Mốc đã ghi gần nhất — ghi xuống localStorage theo nhịp, không phải mỗi `timeupdate`. */
let lastSavedTime = 0;
const SAVE_EVERY_S = 5;

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
  a.addEventListener("loadedmetadata", () => {
    if (pendingSeek > 0) a.currentTime = pendingSeek;
    pendingSeek = 0;
  });
  a.addEventListener("timeupdate", () => {
    if (Math.abs(a.currentTime - lastSavedTime) >= SAVE_EVERY_S) rememberPosition();
  });
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
    rememberPosition();
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

/** Ghi chỗ đang nghe của sách đang phát. */
function rememberPosition() {
  const { bookId, queue, index, status } = useAutoplayStore.getState();
  const item = queue[index];
  if (!bookId || !item || status === "idle" || !audio) return;
  lastSavedTime = audio.currentTime;
  savePosition(bookId, { url: item.url, time: Math.floor(audio.currentTime) });
}

function playIndex(i: number, startAt = 0) {
  clearTimers();
  const { bookId, queue } = useAutoplayStore.getState();
  if (i < 0 || i >= queue.length) {
    stopAutoplay();
    // Nghe hết sách: xoá chỗ nghe dở, lần sau bắt đầu lại từ đầu. Xoá SAU
    // khi dừng — `stopAutoplay` tự ghi lại chỗ đang nghe.
    if (bookId && i >= queue.length) savePosition(bookId, null);
    return;
  }
  retries = 0;
  pendingSeek = startAt;
  lastSavedTime = startAt;
  useAutoplayStore.setState({ index: i, status: "playing" });
  if (bookId) savePosition(bookId, { url: queue[i].url, time: startAt });
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
      // Nạp lại là về giây 0 — đang nghe dở thì tua lại đúng chỗ.
      pendingSeek = a.currentTime;
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
export function startAutoplay(bookId: string, fromIndex = 0, startAt = 0) {
  const state = useAutoplayStore.getState();
  const queue = state.bookId === bookId && state.queue.length > 0 ? state.queue : buildPlaylist(bookId);
  if (queue.length === 0) return;
  useAutoplayStore.setState({ bookId, queue });
  setMediaSessionHandlers();
  playIndex(Math.min(Math.max(fromIndex, 0), queue.length - 1), startAt);
}

/** Nghe tiếp từ chỗ dở của sách; chưa nghe lần nào thì từ bài đầu. */
export function continueAutoplay(bookId: string) {
  const point = findResumePoint(
    buildPlaylist(bookId),
    useAutoplayResumeStore.getState().positions[bookId]
  );
  startAutoplay(bookId, point?.index ?? 0, point?.time ?? 0);
}

export function stopAutoplay() {
  clearTimers();
  rememberPosition();
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
