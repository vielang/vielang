"use client";

import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Một thẻ <audio> dùng chung cho cả phần nghe, phát được TỪNG ĐOẠN
 * (một câu, hay lời chỉ dẫn) hoặc cả file liền mạch như thi thật.
 *
 * Không cắt file nghe thành từng câu: một file ~40MB, cắt ra là 30 file mà
 * trình duyệt phải xin riêng từng cái. Giữ một file, nhảy tới giây bắt đầu và
 * tự dừng ở giây kết thúc — thẻ <audio> xin đúng khoảng byte cần (`Range`).
 */
export function useExamAudio(src: string | undefined) {
  const ref = useRef<HTMLAudioElement | null>(null);
  const stopAt = useRef<number | null>(null);
  /** Các đoạn còn chờ phát sau đoạn hiện tại (xem `playSegments`). */
  const queue = useRef<[number, number][]>([]);
  const [playing, setPlaying] = useState(false);
  const [time, setTime] = useState(0);
  const [segment, setSegment] = useState<[number, number] | null>(null);

  useEffect(() => {
    if (!src) return;
    const audio = new Audio(src);
    audio.preload = "metadata";
    ref.current = audio;
    const checkStop = () => {
      if (stopAt.current === null || audio.currentTime < stopAt.current) return;
      const next = queue.current.shift();
      if (next) {
        stopAt.current = next[1];
        audio.currentTime = next[0];
      } else {
        audio.pause();
        stopAt.current = null;
      }
    };
    // `timeupdate` chỉ bắn ~4 lần/giây (có khi thưa hơn): dừng theo nó là lố
    // tới vài trăm ms — đủ nghe lọt tiếng đầu câu sau. Khi đang phát một đoạn
    // thì soát thêm theo từng khung hình.
    let raf = 0;
    const loop = () => {
      checkStop();
      raf = !audio.paused && stopAt.current !== null ? requestAnimationFrame(loop) : 0;
    };
    const onTime = () => {
      setTime(audio.currentTime);
      checkStop();
    };
    const onPlaying = () => {
      if (!raf) raf = requestAnimationFrame(loop);
    };
    const onPlay = () => setPlaying(true);
    const onPause = () => setPlaying(false);
    audio.addEventListener("timeupdate", onTime);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("playing", onPlaying);
    audio.addEventListener("pause", onPause);
    audio.addEventListener("ended", onPause);
    return () => {
      audio.pause();
      cancelAnimationFrame(raf);
      audio.removeEventListener("timeupdate", onTime);
      audio.removeEventListener("playing", onPlaying);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
      audio.removeEventListener("ended", onPause);
      ref.current = null;
    };
  }, [src]);

  /**
   * Phát lần lượt các đoạn [start, end] (giây), tự dừng ở cuối đoạn cuối —
   * vd hội thoại chung của khối rồi tới phần riêng của câu.
   */
  const playSegments = useCallback((segments: [number, number][]) => {
    const audio = ref.current;
    if (!audio || segments.length === 0) return;
    const [first, ...rest] = segments;
    queue.current = rest;
    stopAt.current = first[1];
    setSegment([first[0], segments[segments.length - 1][1]]);
    audio.currentTime = first[0];
    void audio.play();
  }, []);

  /** Phát liền mạch từ `from` tới hết file (chế độ thi thử). */
  const playFrom = useCallback((from = 0) => {
    const audio = ref.current;
    if (!audio) return;
    stopAt.current = null;
    queue.current = [];
    setSegment(null);
    audio.currentTime = from;
    void audio.play();
  }, []);

  const pause = useCallback(() => ref.current?.pause(), []);

  return { playing, time, segment, playSegments, playFrom, pause, audioRef: ref };
}
