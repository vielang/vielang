"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Languages, Loader2, Maximize, Minimize, Pause, Play, RotateCw } from "lucide-react";
import { cn } from "@/lib/utils";
import { activeCue, videoPosterUrl, videoUrl, type VideoCue, type VideoLesson } from "@/lib/videos";
import { resumeTime, useVideoProgressStore } from "@/lib/video-progress-store";

/**
 * Player tự dựng control bar (không dùng `controls` gốc của `<video>`) — lý
 * do duy nhất: nút toàn màn hình gốc chỉ fullscreen ĐÚNG thẻ `<video>`, mọi
 * lớp phủ (phụ đề) là phần tử ANH EM cùng cấp nên biến mất khi vào fullscreen
 * (video vẫn chạy, phụ đề thì không ai thấy). Ở đây nút toàn màn hình tự làm
 * gọi `requestFullscreen()` trên CONTAINER bọc ngoài (video + phụ đề + control
 * bar cùng nằm trong), nên phụ đề sống sót qua mọi trạng thái.
 *
 * Phụ đề tiếng Hàn (karaoke, tô sáng dần từng chữ) và tiếng Việt (dịch tự
 * động, bật/tắt được) đều đè lên video làm 2 dòng — không tách dòng Việt ra
 * ngoài nữa, để mắt không phải rời khỏi video lúc xem, và cả hai cùng hiện
 * được ở fullscreen.
 *
 * Video gốc quay tỉ lệ ~1706:1080 (~1.58:1), KHÔNG phải 16:9 — ép khung
 * 16:9 sẽ méo hình hoặc thừa viền đen. Container dùng đúng tỉ lệ này lúc
 * xem bình thường; lúc toàn màn hình dùng `object-contain` để không méo,
 * viền đen còn lại (nếu có) chỉ là phần letterbox tối thiểu do tỉ lệ màn
 * hình khác tỉ lệ video — mọi player (YouTube, Netflix…) đều vậy, không
 * cắt hình để lấp đầy vì sẽ mất nội dung ở rìa khung hình.
 *
 * Video ~100-300MB, mạng di động dễ khựng giữa chừng — 3 việc để người xem
 * không phải xem lại từ đầu mỗi lần: (1) tự lưu vị trí đang xem
 * (`video-progress-store`, localStorage) và tự tua lại đúng chỗ lúc mở lại;
 * (2) hiện vòng xoay lúc đang tải thay vì đứng hình im lặng trông như treo;
 * (3) lỗi tải (`error`) thì hiện nút "Thử lại" gọi lại `.load()` rồi tua về
 * đúng chỗ vừa dừng, không bắt xem lại từ đầu.
 */

const ASPECT = "1706/1080";

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

const CONTROL_BTN = "flex size-8 items-center justify-center rounded-md text-white transition-colors hover:bg-white/15 disabled:opacity-30 disabled:hover:bg-transparent";

export function VideoPlayer({ lesson }: { lesson: VideoLesson }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSavedRef = useRef(0);
  const resumedRef = useRef(false);

  const [time, setTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [buffering, setBuffering] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [controlsVisible, setControlsVisible] = useState(true);
  const [koCue, setKoCue] = useState<VideoCue | undefined>();
  const [viCue, setViCue] = useState<VideoCue | undefined>();
  const [showVi, setShowVi] = useState(true);

  const setProgressTime = useVideoProgressStore((s) => s.setTime);

  /** Hiện control bar + phụ đề tạm ẩn lại sau vài giây không chạm tới — chỉ
   * khi đang phát (dừng thì luôn hiện, không có gì để "làm phiền"). */
  const wake = useCallback(() => {
    setControlsVisible(true);
    if (hideTimer.current) clearTimeout(hideTimer.current);
    hideTimer.current = setTimeout(() => {
      if (!videoRef.current?.paused) setControlsVisible(false);
    }, 2500);
  }, []);

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
      setKoCue(activeCue(lesson.koCues, t));
      setViCue(activeCue(lesson.viCues, t));
      // Lưu định kỳ mỗi ~5s lúc đang phát, không phải mỗi lần timeupdate
      // (bắn liên tục) — đỡ ghi localStorage quá dày.
      if (t - lastSavedRef.current >= 5) saveProgress(t);
    };
    const onMeta = () => {
      setDuration(video.duration || 0);
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
    video.addEventListener("waiting", onWaiting);
    video.addEventListener("playing", onPlaying);
    video.addEventListener("error", onError);
    return () => {
      video.removeEventListener("timeupdate", onTime);
      video.removeEventListener("loadedmetadata", onMeta);
      video.removeEventListener("play", onPlay);
      video.removeEventListener("pause", onPause);
      video.removeEventListener("waiting", onWaiting);
      video.removeEventListener("playing", onPlaying);
      video.removeEventListener("error", onError);
      // Rời trang (chuyển tập, quay lại danh sách…) — chốt lại vị trí cuối
      // cùng luôn, không đợi tới mốc lưu định kỳ tiếp theo.
      if (video.currentTime > 0) saveProgress(video.currentTime);
    };
  }, [lesson, wake, setProgressTime]);

  useEffect(() => {
    const onChange = () => setIsFullscreen(isFullscreenNow());
    document.addEventListener("fullscreenchange", onChange);
    document.addEventListener("webkitfullscreenchange", onChange);
    return () => {
      document.removeEventListener("fullscreenchange", onChange);
      document.removeEventListener("webkitfullscreenchange", onChange);
    };
  }, []);

  const togglePlay = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) video.play();
    else video.pause();
  };

  /** Lỗi tải (mạng chập chờn giữa video ~100-300MB) — tải lại rồi tua về
   * đúng chỗ vừa dừng, không bắt xem lại từ đầu. */
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

  const toggleFullscreen = async () => {
    const container = containerRef.current as FullscreenElement | null;
    const doc = document as FullscreenDocument;
    if (!container) return;
    try {
      if (isFullscreenNow()) {
        if (document.exitFullscreen) await document.exitFullscreen();
        else await doc.webkitExitFullscreen?.();
      } else if (container.requestFullscreen) {
        await container.requestFullscreen();
      } else {
        await container.webkitRequestFullscreen?.();
      }
    } catch {
      // Một số trình duyệt/khung nhúng chặn Fullscreen API (Permissions
      // Policy) — coi như không có nút toàn màn hình, không phải lỗi vỡ trang.
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div
        ref={containerRef}
        className={cn(
          "group relative w-full touch-none overflow-hidden bg-black select-none",
          isFullscreen ? "h-full" : "rounded-xl"
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

        {/* Phụ đề: Hàn (karaoke) trên, Việt (tuỳ chọn) dưới — cùng đè lên video. */}
        <div className="pointer-events-none absolute inset-x-0 bottom-12 flex flex-col items-center gap-1 px-3 sm:bottom-14">
          {koCue && (
            <p className="font-korean max-w-[92%] rounded bg-black/70 px-2.5 py-1 text-center text-base leading-snug font-medium sm:text-lg">
              {koCue.words.map((w, i) => (
                <span
                  key={i}
                  className={cn(
                    "transition-opacity duration-150",
                    w.at <= time ? "text-white opacity-100" : "text-white opacity-35"
                  )}
                >
                  {w.t}
                  {i < koCue.words.length - 1 && " "}
                </span>
              ))}
            </p>
          )}
          {showVi && viCue && (
            <p className="max-w-[92%] rounded bg-black/60 px-2 py-0.5 text-center text-sm text-white/90 sm:text-base">
              {viCue.t}
            </p>
          )}
        </div>

        {!playing && !hasError && (
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

        {/* Đang tải lại dữ liệu giữa chừng (mạng chập chờn) — cho biết app
            không treo, chỉ đang chờ, để người xem không tưởng lỗi mà bỏ đi. */}
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
            "absolute inset-x-0 bottom-0 flex flex-col gap-1 bg-gradient-to-t from-black/85 to-transparent px-3 pt-8 pb-2 transition-opacity duration-300",
            controlsVisible ? "opacity-100" : "pointer-events-none opacity-0"
          )}
        >
          <input
            type="range"
            min={0}
            max={duration || 0}
            step={0.1}
            value={time}
            onChange={(e) => {
              const video = videoRef.current;
              if (video) video.currentTime = Number(e.target.value);
            }}
            className="h-1 w-full cursor-pointer"
            style={{ accentColor: "white" }}
            aria-label="Tua video"
          />
          <div className="flex items-center gap-1">
            <button type="button" onClick={togglePlay} aria-label={playing ? "Tạm dừng" : "Phát"} className={CONTROL_BTN}>
              {playing ? <Pause className="size-4" aria-hidden /> : <Play className="size-4" aria-hidden />}
            </button>
            <span className="ml-1 text-xs tabular-nums text-white/80">
              {formatTime(time)} / {formatTime(duration)}
            </span>
            <div className="flex-1" />
            <button
              type="button"
              onClick={() => setShowVi((v) => !v)}
              disabled={!lesson.hasVi}
              aria-pressed={showVi}
              aria-label="Bật/tắt phụ đề tiếng Việt"
              className={cn(CONTROL_BTN, showVi && lesson.hasVi && "bg-white/20")}
            >
              <Languages className="size-4" aria-hidden />
            </button>
            <button
              type="button"
              onClick={toggleFullscreen}
              aria-label={isFullscreen ? "Thoát toàn màn hình" : "Toàn màn hình"}
              className={CONTROL_BTN}
            >
              {isFullscreen ? <Minimize className="size-4" aria-hidden /> : <Maximize className="size-4" aria-hidden />}
            </button>
          </div>
        </div>
      </div>

      {!lesson.hasVi && (
        <p className="text-xs text-muted-foreground">Tập này chưa có phụ đề tiếng Việt.</p>
      )}
    </div>
  );
}
