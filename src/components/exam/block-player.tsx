"use client";

import { Pause, Play, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { useExamAudio } from "@/components/exam/use-exam-audio";

type Segment = [number, number];

function mmss(s: number): string {
  const t = Math.max(0, Math.round(s));
  return `${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}`;
}

/**
 * Trình phát MỘT audio cho cả khối (xem `groupAudio`): một nút phát/tạm
 * dừng, một thanh thời gian kéo được, nút lùi 5 giây. Audio có thể gồm vài
 * đoạn rời nhau trong file nghe của cả phần, người học vẫn thấy một bài liền.
 *
 * "Thời gian khối" là tổng độ dài các đoạn; đổi qua lại với thời gian thật
 * trong file bằng `toReal` / `toBlock`.
 */
export function BlockPlayer({
  audio,
  segments,
  label,
}: {
  audio: ReturnType<typeof useExamAudio>;
  segments: Segment[];
  label: string;
}) {
  const total = segments.reduce((n, [a, b]) => n + (b - a), 0);
  const blockStart = segments[0][0];
  const blockEnd = segments[segments.length - 1][1];

  /** Thời gian thật trong file → thời gian khối (null nếu ngoài khối). */
  const toBlock = (t: number): number | null => {
    let acc = 0;
    for (const [a, b] of segments) {
      if (t >= a && t <= b) return acc + (t - a);
      acc += b - a;
    }
    return null;
  };
  /** Thời gian khối → vị trí thật trong file + các đoạn còn lại để phát tiếp. */
  const fromBlock = (v: number): Segment[] => {
    let acc = 0;
    for (let i = 0; i < segments.length; i++) {
      const [a, b] = segments[i];
      if (v < acc + (b - a)) return [[a + (v - acc), b], ...segments.slice(i + 1)];
      acc += b - a;
    }
    return [];
  };

  // Đang phát khối này khi đoạn đang phát kết thúc đúng ở cuối khối.
  const mine = audio.segment !== null && audio.segment[1] === blockEnd && audio.segment[0] >= blockStart;
  const playing = audio.playing && mine;
  const pos = Math.min(total, toBlock(audio.time) ?? 0);
  const atEnd = total - pos < 0.3;

  const playAt = (v: number) => {
    const rest = fromBlock(Math.max(0, Math.min(v, total - 0.05)));
    if (rest.length) audio.playSegments(rest);
  };
  const toggle = () => {
    if (playing) audio.pause();
    else playAt(atEnd ? 0 : pos);
  };
  const seekTo = (v: number) => {
    if (playing) playAt(v);
    else {
      const rest = fromBlock(Math.max(0, Math.min(v, total - 0.05)));
      if (rest.length) audio.seek(rest[0][0]);
    }
  };

  return (
    <div className="flex items-center gap-2 rounded-xl bg-muted/60 px-2 py-1.5" role="group" aria-label={label}>
      <Button size="icon-lg" className="rounded-full" onClick={toggle} aria-label={playing ? "Tạm dừng" : "Phát"}>
        {playing ? <Pause className="size-4" aria-hidden /> : <Play className="size-4" aria-hidden />}
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={() => seekTo(pos - 5)}
        aria-label="Lùi 5 giây"
        disabled={pos < 0.5}
      >
        <RotateCcw className="size-3.5" aria-hidden />
      </Button>
      <input
        type="range"
        min={0}
        max={total}
        step={0.1}
        value={pos}
        onChange={(e) => seekTo(Number(e.target.value))}
        aria-label="Thời gian"
        aria-valuetext={`${mmss(pos)} / ${mmss(total)}`}
        className="h-1.5 min-w-0 flex-1 cursor-pointer accent-foreground"
      />
      <span className="shrink-0 pr-1 text-xs text-muted-foreground tabular-nums">
        {mmss(pos)} / {mmss(total)}
      </span>
    </div>
  );
}
