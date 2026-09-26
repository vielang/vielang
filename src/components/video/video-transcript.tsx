"use client";

import { memo, useEffect, useRef, useState } from "react";
import { ArrowDownToLine, Repeat, Settings } from "lucide-react";
import { cn } from "@/lib/utils";
import type { TranscriptLine } from "@/lib/videos";
import { VideoSettingsPopover } from "@/components/video/video-settings";

/**
 * Transcript song ngữ của cả tập: mỗi câu tiếng Hàn kèm bản dịch ngay dưới
 * (không tách hai cột — mắt đọc từ trên xuống, không phải dò ngang cho khớp
 * câu), câu đang phát được tô và tự cuộn vào tầm mắt.
 *
 * Chạm một câu là tua tới đầu câu đó và phát — cách nhanh nhất để nghe lại
 * một câu chưa nghe rõ, không phải kéo thanh thời gian.
 *
 * Tự cuộn sẽ TẠM NGỪNG ngay khi người học tự cuộn (đang đọc lại câu phía
 * trên mà bị kéo tuột xuống thì rất khó chịu) — hiện nút "Về câu đang phát"
 * để bật lại.
 */
export function VideoTranscript({
  lines,
  activeIdx,
  time,
  loopIdx,
  showVi,
  blurVi,
  karaoke,
  autoScroll,
  hasVi,
  onSeek,
  className,
}: {
  lines: TranscriptLine[];
  activeIdx: number;
  time: number;
  loopIdx: number | null;
  showVi: boolean;
  blurVi: boolean;
  karaoke: boolean;
  autoScroll: boolean;
  hasVi: boolean;
  onSeek: (idx: number) => void;
  className?: string;
}) {
  const listRef = useRef<HTMLOListElement>(null);
  const [userScrolled, setUserScrolled] = useState(false);
  const [revealed, setRevealed] = useState<Set<number>>(() => new Set());

  const follow = autoScroll && !userScrolled;

  useEffect(() => {
    const list = listRef.current;
    if (!follow || !list || activeIdx < 0) return;
    const row = list.children[activeIdx] as HTMLElement | undefined;
    if (!row) return;
    // Đặt câu đang phát ở khoảng 1/3 từ trên xuống: còn thấy được 1-2 câu
    // vừa qua (để đối chiếu) và các câu sắp tới.
    list.scrollTo({ top: row.offsetTop - list.clientHeight / 3, behavior: "smooth" });
  }, [activeIdx, follow]);

  // Chỉ thao tác CỦA NGƯỜI DÙNG (lăn chuột, vuốt, phím) mới tính là tự cuộn —
  // `scroll` thì bắn cả khi chính mình gọi scrollTo ở trên.
  const onManualScroll = () => {
    if (autoScroll) setUserScrolled(true);
  };

  const toggleReveal = (i: number) =>
    setRevealed((prev) => {
      const next = new Set(prev);
      if (next.has(i)) next.delete(i);
      else next.add(i);
      return next;
    });

  return (
    <div className={cn("relative flex min-h-0 flex-col overflow-hidden bg-background lg:rounded-xl lg:border lg:border-border", className)}>
      <div className="flex shrink-0 items-center justify-between gap-2 border-b border-border/60 px-4 py-2 lg:px-3">
        <h2 className="text-sm font-semibold">Transcript</h2>
        <VideoSettingsPopover hasVi={hasVi}>
          <button
            type="button"
            aria-label="Cài đặt"
            className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <Settings className="size-3.5" aria-hidden />
            Cài đặt
          </button>
        </VideoSettingsPopover>
      </div>

      {!hasVi && (
        <p className="shrink-0 border-b border-border/60 px-4 py-2 text-xs text-muted-foreground lg:px-3">
          Tập này chưa có bản dịch tiếng Việt.
        </p>
      )}

      <ol
        ref={listRef}
        className="relative min-h-0 flex-1 overflow-y-auto overscroll-contain py-1"
        onWheel={onManualScroll}
        onTouchMove={onManualScroll}
        onKeyDown={onManualScroll}
      >
        {lines.map((line, i) => (
          <TranscriptRow
            key={i}
            index={i}
            line={line}
            active={i === activeIdx}
            // Chỉ câu đang phát cần biết thời gian (tô từng chữ) — các dòng
            // khác nhận hằng số để `memo` bỏ qua, không vẽ lại cả trăm dòng
            // mỗi khung hình.
            time={i === activeIdx && karaoke ? time : -1}
            looping={i === loopIdx}
            showVi={showVi}
            blurred={blurVi && !revealed.has(i)}
            onSeek={onSeek}
            onReveal={toggleReveal}
          />
        ))}
      </ol>

      {autoScroll && userScrolled && (
        <button
          type="button"
          onClick={() => setUserScrolled(false)}
          className="absolute bottom-3 left-1/2 inline-flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-foreground px-3 py-1.5 text-xs font-medium text-background shadow-lg"
        >
          <ArrowDownToLine className="size-3.5" aria-hidden />
          Về câu đang phát
        </button>
      )}
    </div>
  );
}

function formatStamp(sec: number): string {
  const total = Math.floor(sec);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

const TranscriptRow = memo(function TranscriptRow({
  index,
  line,
  active,
  time,
  looping,
  showVi,
  blurred,
  onSeek,
  onReveal,
}: {
  index: number;
  line: TranscriptLine;
  active: boolean;
  time: number;
  looping: boolean;
  showVi: boolean;
  blurred: boolean;
  onSeek: (idx: number) => void;
  onReveal: (idx: number) => void;
}) {
  // "[음악]", "[박수]"… — không phải lời thoại, làm nhạt đi cho đỡ rối mắt.
  const isSound = /^\[.*\]$/.test(line.ko.t.trim());
  return (
    <li
      className={cn(
        "group flex cursor-pointer gap-3 border-l-2 px-4 py-2 transition-colors lg:px-3",
        active ? "border-primary bg-primary/10" : "border-transparent hover:bg-muted/60"
      )}
      onClick={() => onSeek(index)}
    >
      <span
        className={cn(
          "mt-0.5 w-9 shrink-0 text-xs tabular-nums",
          active ? "font-medium text-primary" : "text-muted-foreground"
        )}
      >
        {formatStamp(line.ko.s)}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <p className={cn("font-korean text-[15px] leading-relaxed", isSound && "text-muted-foreground italic")}>
          {time >= 0
            ? line.ko.words.map((w, i) => (
                <span key={i} className={w.at <= time ? "text-foreground" : "text-foreground/45"}>
                  {w.t}
                  {i < line.ko.words.length - 1 && " "}
                </span>
              ))
            : line.ko.t}
        </p>
        {showVi && line.vi && !isSound && (
          <p
            className={cn(
              "text-sm leading-snug text-muted-foreground transition-[filter]",
              blurred && "blur-[5px] select-none hover:blur-none"
            )}
            onClick={
              blurred
                ? (e) => {
                    // Chạm lần đầu chỉ để hé bản dịch, không tua video.
                    e.stopPropagation();
                    onReveal(index);
                  }
                : undefined
            }
          >
            {line.vi}
          </p>
        )}
      </div>
      {looping && <Repeat className="mt-1 size-3.5 shrink-0 text-primary" aria-label="Đang lặp câu này" />}
    </li>
  );
});
