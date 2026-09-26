"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Languages,
  Loader2,
  Maximize,
  Minimize,
  Pause,
  Play,
  Repeat,
  RotateCw,
  Settings,
  SkipBack,
  SkipForward,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  buildTranscript,
  cueIndexAt,
  lastStartedCueIndex,
  videoPosterUrl,
  videoUrl,
  type VideoLesson,
} from "@/lib/videos";
import { resumeTime, useVideoProgressStore } from "@/lib/video-progress-store";
import { DEFAULT_VIDEO_PREFS, useVideoPrefsStore } from "@/lib/video-prefs-store";
import { useIsClient } from "@/lib/use-is-client";
import { VideoSettingsPopover } from "@/components/video/video-settings";
import { VideoTranscript } from "@/components/video/video-transcript";

/**
 * Màn xem một tập: player + transcript song ngữ, bố cục như YouTube.
 *
 * - Điện thoại: video tràn hết bề ngang (không chừa lề), transcript chiếm
 *   phần màn hình còn lại bên dưới và tự cuộn bên trong — video luôn nằm
 *   trong tầm mắt khi đọc transcript.
 * - Máy tính: video bên trái chiếm hết phần rộng còn lại, transcript là cột
 *   bên phải cao bằng màn hình. Tắt transcript (cài đặt) thì video rộng hết.
 *
 * Player tự dựng control bar (không dùng `controls` gốc của `<video>`) — lý
 * do: nút toàn màn hình gốc chỉ fullscreen ĐÚNG thẻ `<video>`, mọi lớp phủ
 * (phụ đề, cài đặt) là phần tử anh em nên biến mất khi vào fullscreen. Ở đây
 * `requestFullscreen()` gọi trên CONTAINER bọc ngoài. iPhone không có
 * Fullscreen API cho phần tử thường (chỉ cho chính thẻ video, mất phụ đề) —
 * khi đó giả lập bằng một lớp `fixed` phủ kín màn hình.
 *
 * Luyện nghe: câu trước/câu sau, nghe lại câu, lặp một câu, tự dừng cuối
 * mỗi câu, đổi tốc độ. Vị trí phát được đọc bằng `requestAnimationFrame` lúc
 * đang phát (không chỉ `timeupdate`, vốn chỉ bắn ~4 lần/giây): dừng/lặp đúng
 * cuối câu, không lố sang vài chữ đầu của câu sau.
 *
 * Video ~100-300MB, mạng di động dễ khựng giữa chừng: tự lưu vị trí đang xem
 * và tua lại lúc mở lại, hiện vòng xoay lúc đang tải, lỗi tải thì có nút
 * "Thử lại" tua về đúng chỗ vừa dừng.
 */

const ASPECT = "1706/1080";

/** Cỡ chữ phụ đề (px) theo `subSize`; toàn màn hình thì nhân thêm. */
const KO_SIZE = [14, 17, 20, 24];
const VI_SIZE = [12, 14, 16, 19];
const FULLSCREEN_SCALE = 1.5;

/** Bấm "câu trước" khi câu hiện tại đã chạy quá chừng này giây thì về đầu
 * câu hiện tại (như nút "bài trước" của trình phát nhạc). */
const RESTART_THRESHOLD = 1.5;

interface FullscreenDocument extends Document {
  webkitFullscreenElement?: Element | null;
  webkitExitFullscreen?: () => Promise<void>;
}
interface FullscreenElement extends HTMLElement {
  webkitRequestFullscreen?: () => Promise<void>;
}

function isFullscreenNow(): boolean {
  const doc = document as FullscreenDocument;
  return Boolean(document.fullscreenElement || doc.webkitFullscreenElement);
}

function formatTime(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) return "0:00";
  const total = Math.floor(sec);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false;
  return el.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName);
}

const CONTROL_BTN =
  "flex size-8 shrink-0 items-center justify-center rounded-md text-white transition-colors hover:bg-white/15 disabled:opacity-30 disabled:hover:bg-transparent";

export function VideoPlayer({ lesson, info }: { lesson: VideoLesson; info: React.ReactNode }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSavedRef = useRef(0);
  const resumedRef = useRef(false);
  /** Mốc cuối câu đang phát — tự dừng khi tới (nếu bật). */
  const pauseAtRef = useRef<number | null>(null);
  const lastIdxRef = useRef(-1);

  const [time, setTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [buffering, setBuffering] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [nativeFullscreen, setNativeFullscreen] = useState(false);
  const [pseudoFullscreen, setPseudoFullscreen] = useState(false);
  const [controlsVisible, setControlsVisible] = useState(true);
  const settingsOpenRef = useRef(false);
  /** Khung player dạng state (không chỉ ref) — popover cài đặt cần nó làm
   * nơi gắn lúc toàn màn hình, mà đọc `ref.current` trong lúc render thì không. */
  const [containerEl, setContainerEl] = useState<HTMLDivElement | null>(null);
  const setContainer = useCallback((el: HTMLDivElement | null) => {
    containerRef.current = el;
    setContainerEl(el);
  }, []);
  /** Câu đang lặp lại (chỉ số trong `koCues`), `null` nếu không lặp. */
  const [loopIdx, setLoopIdx] = useState<number | null>(null);
  /** Câu có bản dịch vừa được chạm để hé (khi đang che tiếng Việt) — sang
   * câu khác thì tự che lại. */
  const [viRevealedIdx, setViRevealedIdx] = useState(-1);

  const isFullscreen = nativeFullscreen || pseudoFullscreen;

  // Tuỳ chọn nằm ở localStorage: lượt vẽ đầu (khớp HTML server) dùng mặc
  // định, sang trình duyệt mới đọc tuỳ chọn thật — tránh lỗi hydration.
  const isClient = useIsClient();
  const storedPrefs = useVideoPrefsStore();
  const prefs = isClient ? storedPrefs : { ...storedPrefs, ...DEFAULT_VIDEO_PREFS };
  const setPrefs = storedPrefs.set;
  const hasVi = lesson.hasVi;

  const setProgressTime = useVideoProgressStore((s) => s.setTime);

  const lines = useMemo(() => buildTranscript(lesson), [lesson]);
  const cues = lesson.koCues;
  const activeIdx = cueIndexAt(cues, time);
  // Ở khoảng lặng giữa hai câu thì vẫn tô câu vừa nói xong trong transcript,
  // không để trống — đỡ mất dấu đang ở đâu.
  const currentIdx = lastStartedCueIndex(cues, time);

  const prefsRef = useRef(prefs);
  const loopIdxRef = useRef(loopIdx);
  useEffect(() => {
    prefsRef.current = prefs;
    loopIdxRef.current = loopIdx;
  });

  /** Hiện control bar, tự ẩn lại sau vài giây không chạm tới — chỉ khi đang
   * phát và không mở cài đặt (dừng thì luôn hiện). */
  const wake = useCallback(() => {
    setControlsVisible(true);
    if (hideTimer.current) clearTimeout(hideTimer.current);
    hideTimer.current = setTimeout(() => {
      if (!videoRef.current?.paused && !settingsOpenRef.current) setControlsVisible(false);
    }, 2500);
  }, []);

  const onSettingsOpenChange = useCallback(
    (open: boolean) => {
      settingsOpenRef.current = open;
      if (open) setControlsVisible(true);
      else wake();
    },
    [wake]
  );

  useEffect(() => {
    const video = videoRef.current;
    if (video) video.playbackRate = video.defaultPlaybackRate = prefs.playbackRate;
  }, [prefs.playbackRate]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const saveProgress = (t: number) => {
      lastSavedRef.current = t;
      setProgressTime(lesson.id, t, video.duration || 0);
    };

    const onTime = () => {
      const t = video.currentTime;
      setTime(t);
      // Lưu định kỳ mỗi ~5s lúc đang phát, không phải mỗi lần timeupdate —
      // đỡ ghi localStorage quá dày.
      if (Math.abs(t - lastSavedRef.current) >= 5) saveProgress(t);
    };
    const onMeta = () => {
      setDuration(video.duration || 0);
      video.playbackRate = prefsRef.current.playbackRate;
      if (!resumedRef.current) {
        resumedRef.current = true;
        const saved = useVideoProgressStore.getState().lessons[lesson.id];
        const t = resumeTime(saved);
        if (t > 0) video.currentTime = t;
      }
    };
    const onPlay = () => {
      setPlaying(true);
      wake();
    };
    const onPause = () => {
      setPlaying(false);
      setControlsVisible(true);
      if (hideTimer.current) clearTimeout(hideTimer.current);
      saveProgress(video.currentTime);
    };
    const onSeeked = () => {
      // Tua tay thì mốc tự dừng tính lại theo câu mới tới.
      const idx = cueIndexAt(lesson.koCues, video.currentTime);
      lastIdxRef.current = idx;
      pauseAtRef.current = idx >= 0 ? lesson.koCues[idx].e : null;
      setTime(video.currentTime);
    };
    const onWaiting = () => setBuffering(true);
    const onPlaying = () => {
      setBuffering(false);
      setHasError(false);
    };
    const onError = () => {
      setBuffering(false);
      setHasError(true);
    };

    video.addEventListener("timeupdate", onTime);
    video.addEventListener("loadedmetadata", onMeta);
    video.addEventListener("play", onPlay);
    video.addEventListener("pause", onPause);
    video.addEventListener("seeked", onSeeked);
    video.addEventListener("waiting", onWaiting);
    video.addEventListener("playing", onPlaying);
    video.addEventListener("error", onError);
    // Metadata có thể đã tải xong trước khi effect này chạy (cache).
    if (video.readyState >= 1) onMeta();
    return () => {
      video.removeEventListener("timeupdate", onTime);
      video.removeEventListener("loadedmetadata", onMeta);
      video.removeEventListener("play", onPlay);
      video.removeEventListener("pause", onPause);
      video.removeEventListener("seeked", onSeeked);
      video.removeEventListener("waiting", onWaiting);
      video.removeEventListener("playing", onPlaying);
      video.removeEventListener("error", onError);
      // Rời trang (chuyển tập, quay lại danh sách…) — chốt lại vị trí cuối.
      if (video.currentTime > 0) saveProgress(video.currentTime);
    };
  }, [lesson, wake, setProgressTime]);

  // Vòng đọc vị trí theo khung hình lúc đang phát: karaoke mượt, và tự dừng /
  // lặp câu đúng mốc cuối câu.
  useEffect(() => {
    const video = videoRef.current;
    if (!playing || !video) return;
    let raf = 0;
    let lastShown = -1;
    const tick = () => {
      let t = video.currentTime;
      const loop = loopIdxRef.current;
      if (loop !== null && cues[loop] && t >= cues[loop].e) {
        video.currentTime = cues[loop].s;
        t = cues[loop].s;
      } else if (prefsRef.current.autoPause && pauseAtRef.current !== null && t >= pauseAtRef.current) {
        pauseAtRef.current = null;
        video.pause();
      }
      const idx = cueIndexAt(cues, t);
      if (idx !== lastIdxRef.current) {
        lastIdxRef.current = idx;
        if (idx >= 0) pauseAtRef.current = cues[idx].e;
      }
      if (Math.abs(t - lastShown) >= 0.05) {
        lastShown = t;
        setTime(t);
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [playing, cues]);

  useEffect(() => {
    const onChange = () => setNativeFullscreen(isFullscreenNow());
    document.addEventListener("fullscreenchange", onChange);
    document.addEventListener("webkitfullscreenchange", onChange);
    return () => {
      document.removeEventListener("fullscreenchange", onChange);
      document.removeEventListener("webkitfullscreenchange", onChange);
    };
  }, []);

  // Toàn màn hình giả lập: khoá cuộn trang phía sau.
  useEffect(() => {
    if (!pseudoFullscreen) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, [pseudoFullscreen]);

  const togglePlay = useCallback(() => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) video.play().catch(() => {});
    else video.pause();
  }, []);

  const seekTo = useCallback((t: number, play = false) => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = Math.max(0, t);
    setTime(video.currentTime);
    if (play) video.play().catch(() => {});
  }, []);

  const seekToCue = useCallback(
    (idx: number) => {
      const cue = cues[idx];
      if (!cue) return;
      // Đang lặp một câu mà chọn câu khác thì lặp câu mới đó.
      setLoopIdx((l) => (l === null ? null : idx));
      seekTo(cue.s, true);
    },
    [cues, seekTo]
  );

  const prevSentence = useCallback(() => {
    const t = videoRef.current?.currentTime ?? 0;
    const cur = lastStartedCueIndex(cues, t);
    if (cur < 0) return seekToCue(0);
    seekToCue(t - cues[cur].s > RESTART_THRESHOLD ? cur : Math.max(0, cur - 1));
  }, [cues, seekToCue]);

  const nextSentence = useCallback(() => {
    const t = videoRef.current?.currentTime ?? 0;
    const cur = lastStartedCueIndex(cues, t);
    if (cur + 1 < cues.length) seekToCue(cur + 1);
  }, [cues, seekToCue]);

  const replaySentence = useCallback(() => {
    const cur = lastStartedCueIndex(cues, videoRef.current?.currentTime ?? 0);
    if (cur >= 0) seekToCue(cur);
  }, [cues, seekToCue]);

  const toggleLoop = useCallback(() => {
    setLoopIdx((l) => {
      if (l !== null) return null;
      const cur = lastStartedCueIndex(cues, videoRef.current?.currentTime ?? 0);
      return cur >= 0 ? cur : null;
    });
  }, [cues]);

  /** Lỗi tải (mạng chập chờn) — tải lại rồi tua về đúng chỗ vừa dừng. */
  const retry = () => {
    const video = videoRef.current;
    if (!video) return;
    const resumeAt = video.currentTime || useVideoProgressStore.getState().lessons[lesson.id]?.time || 0;
    setHasError(false);
    const onReady = () => {
      video.currentTime = resumeAt;
      video.play().catch(() => {});
      video.removeEventListener("loadedmetadata", onReady);
    };
    video.addEventListener("loadedmetadata", onReady);
    video.load();
  };

  const toggleFullscreen = useCallback(async () => {
    const container = containerRef.current as FullscreenElement | null;
    const doc = document as FullscreenDocument;
    if (!container) return;
    if (pseudoFullscreen) {
      setPseudoFullscreen(false);
      return;
    }
    try {
      if (isFullscreenNow()) {
        if (document.exitFullscreen) await document.exitFullscreen();
        else await doc.webkitExitFullscreen?.();
      } else if (container.requestFullscreen) {
        await container.requestFullscreen();
      } else if (container.webkitRequestFullscreen) {
        await container.webkitRequestFullscreen();
      } else {
        setPseudoFullscreen(true);
      }
    } catch {
      // Bị chặn (Permissions Policy, khung nhúng…) — vẫn phủ kín màn hình
      // được bằng lớp giả lập.
      setPseudoFullscreen(true);
    }
  }, [pseudoFullscreen]);

  // Phím tắt — kiểu YouTube (Space/K, ←/→, F) cộng phím luyện nghe theo câu.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey || isTypingTarget(e.target)) return;
      const video = videoRef.current;
      if (!video) return;
      switch (e.key.toLowerCase()) {
        case " ":
        case "k":
          // Space trên một nút đang focus thì để nút tự xử lý (tránh bấm 2 lần).
          if (e.key === " " && e.target instanceof HTMLButtonElement) return;
          togglePlay();
          break;
        case "arrowleft":
          seekTo(video.currentTime - 5);
          break;
        case "arrowright":
          seekTo(video.currentTime + 5);
          break;
        case "a":
          prevSentence();
          break;
        case "d":
          nextSentence();
          break;
        case "r":
          replaySentence();
          break;
        case "l":
          toggleLoop();
          break;
        case "v":
          if (hasVi) setPrefs({ subVi: !prefsRef.current.subVi });
          break;
        case "f":
          toggleFullscreen();
          break;
        case "escape":
          if (pseudoFullscreen) setPseudoFullscreen(false);
          return;
        default:
          return;
      }
      e.preventDefault();
      wake();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [togglePlay, seekTo, prevSentence, nextSentence, replaySentence, toggleLoop, toggleFullscreen, setPrefs, hasVi, pseudoFullscreen, wake]);

  const koCue = activeIdx >= 0 ? cues[activeIdx] : undefined;
  const viText = activeIdx >= 0 ? lines[activeIdx].vi : undefined;
  const sizeScale = isFullscreen ? FULLSCREEN_SCALE : 1;
  const showTranscript = prefs.showTranscript;
  // Phụ đề nằm ngay trên control bar lúc control hiện, hạ sát đáy lúc ẩn.
  const subtitleBottom = controlsVisible || !playing ? "bottom-16" : "bottom-3";

  const player = (
    <div
      ref={setContainer}
      className={cn(
        "group relative w-full touch-manipulation overflow-hidden bg-black select-none",
        pseudoFullscreen && "fixed inset-0 z-[100] h-dvh pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)]",
        nativeFullscreen && "h-full",
        !isFullscreen && "lg:rounded-xl lg:max-h-[calc(100dvh-10rem)]"
      )}
      style={isFullscreen ? undefined : { aspectRatio: ASPECT }}
      onMouseMove={wake}
      onTouchStart={wake}
    >
      <video
        ref={videoRef}
        src={videoUrl(lesson.id)}
        poster={videoPosterUrl(lesson.id)}
        playsInline
        preload="metadata"
        onClick={togglePlay}
        className="h-full w-full object-contain"
      />

      {/* Phụ đề: Hàn (karaoke) trên, Việt dưới — cùng đè lên video. */}
      <div
        className={cn(
          "pointer-events-none absolute inset-x-0 flex flex-col items-center gap-1 px-3 transition-[bottom] duration-300",
          subtitleBottom
        )}
      >
        {prefs.subKo && koCue && (
          <p
            className="font-korean max-w-[94%] rounded bg-black/70 px-2.5 py-1 text-center leading-snug font-medium text-white"
            style={{ fontSize: KO_SIZE[prefs.subSize] * sizeScale }}
          >
            {koCue.words.map((w, i) => (
              <span key={i} className={cn("transition-opacity duration-150", w.at <= time ? "opacity-100" : "opacity-40")}>
                {w.t}
                {i < koCue.words.length - 1 && " "}
              </span>
            ))}
          </p>
        )}
        {prefs.subVi && hasVi && viText && (
          <p
            className={cn(
              "max-w-[94%] rounded bg-black/60 px-2 py-0.5 text-center leading-snug text-white/90",
              prefs.blurVi && viRevealedIdx !== activeIdx && "pointer-events-auto cursor-pointer blur-[6px] hover:blur-none"
            )}
            style={{ fontSize: VI_SIZE[prefs.subSize] * sizeScale }}
            onClick={prefs.blurVi ? () => setViRevealedIdx(activeIdx) : undefined}
          >
            {viText}
          </p>
        )}
      </div>

      {!playing && !hasError && !buffering && (
        <button
          type="button"
          onClick={togglePlay}
          aria-label="Phát"
          className="absolute inset-0 flex items-center justify-center bg-black/10"
        >
          <span className="flex size-16 items-center justify-center rounded-full bg-white/90 text-black shadow-lg transition-transform hover:scale-105">
            <Play className="size-7 translate-x-0.5" fill="currentColor" aria-hidden />
          </span>
        </button>
      )}

      {buffering && !hasError && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <Loader2 className="size-10 animate-spin text-white/90" aria-hidden />
        </div>
      )}

      {hasError && (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-black/85 px-4 text-center">
          <p className="text-sm text-white/90">Không tải được video — kiểm tra mạng rồi thử lại.</p>
          <button
            type="button"
            onClick={retry}
            className="inline-flex items-center gap-1.5 rounded-full bg-white px-4 py-1.5 text-sm font-medium text-black transition-transform hover:scale-105"
          >
            <RotateCw className="size-3.5" aria-hidden />
            Thử lại
          </button>
        </div>
      )}

      <div
        className={cn(
          "absolute inset-x-0 bottom-0 flex flex-col gap-1 bg-gradient-to-t from-black/85 to-transparent px-2 pt-8 pb-1.5 transition-opacity duration-300 sm:px-3",
          pseudoFullscreen && "pb-[max(0.375rem,env(safe-area-inset-bottom))]",
          controlsVisible ? "opacity-100" : "pointer-events-none opacity-0"
        )}
      >
        <input
          type="range"
          min={0}
          max={duration || 0}
          step={0.1}
          value={time}
          onChange={(e) => seekTo(Number(e.target.value))}
          className="h-1 w-full cursor-pointer"
          style={{ accentColor: "white" }}
          aria-label="Tua video"
        />
        <div className="flex items-center gap-0.5">
          <button type="button" onClick={togglePlay} aria-label={playing ? "Tạm dừng" : "Phát"} className={CONTROL_BTN}>
            {playing ? <Pause className="size-4" aria-hidden /> : <Play className="size-4" aria-hidden />}
          </button>
          <button type="button" onClick={prevSentence} aria-label="Câu trước (A)" title="Câu trước (A)" className={CONTROL_BTN}>
            <SkipBack className="size-4" aria-hidden />
          </button>
          <button type="button" onClick={nextSentence} aria-label="Câu sau (D)" title="Câu sau (D)" className={CONTROL_BTN}>
            <SkipForward className="size-4" aria-hidden />
          </button>
          <button
            type="button"
            onClick={toggleLoop}
            aria-pressed={loopIdx !== null}
            aria-label="Lặp lại câu này (L)"
            title="Lặp lại câu này (L)"
            className={cn(CONTROL_BTN, loopIdx !== null && "bg-white/25")}
          >
            <Repeat className="size-4" aria-hidden />
          </button>
          <span className="ml-1 truncate text-xs tabular-nums text-white/80">
            {formatTime(time)} / {formatTime(duration)}
          </span>
          <div className="flex-1" />
          {prefs.playbackRate !== 1 && (
            <span className="mr-0.5 text-xs tabular-nums text-white/80">{prefs.playbackRate}×</span>
          )}
          <button
            type="button"
            onClick={() => setPrefs({ subVi: !prefs.subVi })}
            disabled={!hasVi}
            aria-pressed={prefs.subVi && hasVi}
            aria-label="Bật/tắt phụ đề tiếng Việt (V)"
            title="Phụ đề tiếng Việt (V)"
            className={cn(CONTROL_BTN, prefs.subVi && hasVi && "bg-white/25")}
          >
            <Languages className="size-4" aria-hidden />
          </button>
          <PlayerSettingsButton
            container={isFullscreen ? containerEl : undefined}
            hasVi={hasVi}
            onOpenChange={onSettingsOpenChange}
          />
          <button
            type="button"
            onClick={toggleFullscreen}
            aria-label={isFullscreen ? "Thoát toàn màn hình (F)" : "Toàn màn hình (F)"}
            className={CONTROL_BTN}
          >
            {isFullscreen ? <Minimize className="size-4" aria-hidden /> : <Maximize className="size-4" aria-hidden />}
          </button>
        </div>
      </div>
    </div>
  );

  return (
    <div
      className={cn(
        // Điện thoại: cả màn là một cột cao đúng bằng phần dưới header —
        // video trên, transcript cuộn trong phần còn lại.
        "flex h-[calc(100dvh-3.5rem-env(safe-area-inset-top))] flex-col",
        "lg:grid lg:h-auto lg:items-start lg:gap-x-6",
        showTranscript ? "lg:grid-cols-[minmax(0,1fr)_minmax(340px,420px)]" : "lg:grid-cols-1"
      )}
    >
      <div className="shrink-0 lg:col-start-1 lg:row-start-1">{player}</div>
      <div className="shrink-0 lg:col-start-1 lg:row-start-2">{info}</div>
      {showTranscript ? (
        <VideoTranscript
          className="flex-1 border-t border-border/60 lg:col-start-2 lg:row-span-2 lg:row-start-1 lg:h-[calc(100dvh-3.5rem-env(safe-area-inset-top)-3rem)] lg:flex-none lg:border-t"
          lines={lines}
          activeIdx={currentIdx}
          time={time}
          loopIdx={loopIdx}
          blurVi={prefs.blurVi}
          hasVi={hasVi}
          onSeek={seekToCue}
        />
      ) : (
        <div className="flex-1 lg:hidden" />
      )}
    </div>
  );
}

function PlayerSettingsButton({
  container,
  hasVi,
  onOpenChange,
}: {
  container?: HTMLElement | null;
  hasVi: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <VideoSettingsPopover container={container} hasVi={hasVi} onOpenChange={onOpenChange}>
      <button type="button" aria-label="Cài đặt" title="Cài đặt" className={CONTROL_BTN}>
        <Settings className="size-4" aria-hidden />
      </button>
    </VideoSettingsPopover>
  );
}
