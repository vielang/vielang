"use client";

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  AlertTriangle,
  GripVertical,
  Mic,
  Pause,
  Pencil,
  Play,
  Square,
  Trash2,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { getBlob, putBlob } from "@/lib/idb-storage";
import {
  RECORDER_ERROR_MESSAGE,
  startRecording,
  toRecorderError,
  type ActiveRecording,
} from "@/lib/recorder";
import {
  formatDuration,
  recordingKey,
  recordingLabel,
  useRecordingStore,
  type Recording,
} from "@/lib/recording-store";
import { clampToViewport, useDraggable } from "@/lib/use-draggable";
import { recorderAnchor } from "@/lib/widget-dock";
import { pauseAutoplay } from "@/lib/autoplay-player";
import { cn } from "@/lib/utils";
import { useActivityStore } from "@/lib/activity-store";

/** Nhịp cập nhật đồng hồ lúc đang thu. */
const TICK_MS = 200;
/** Ngắn hơn chừng này thì gần như chắc là bấm nhầm — bỏ, không lưu. */
const MIN_DURATION_MS = 400;

/**
 * Bảng ghi âm của một trang sách: một nút thu, một đồng hồ, và danh sách bản
 * ghi đã có — nghe lại, đổi tên, xoá.
 *
 * Dùng để tự đọc to một đoạn tiếng Hàn rồi nghe lại xem phát âm đã giống
 * chưa, nên mọi thứ xoay quanh đúng một việc: bấm thu, bấm nghe. Không có
 * dạng sóng, không cắt ghép — thêm vào là thành phần mềm thu âm, không còn
 * là chỗ học tiếng.
 *
 * Kéo thả được như các widget nổi khác (xem `use-draggable`).
 */
export function RecorderWidget({ bookId, page }: { bookId: string; page: number }) {
  const open = useRecordingStore((s) => s.open);
  const setOpen = useRecordingStore((s) => s.setOpen);
  const pos = useRecordingStore((s) => s.panelPos);
  const setPos = useRecordingStore((s) => s.setPanelPos);
  const list = useRecordingStore((s) => s.recordings[recordingKey(bookId, page)]);
  const addRecording = useRecordingStore((s) => s.addRecording);
  const recordRecording = useActivityStore((s) => s.recordRecording);
  const removeRecording = useRecordingStore((s) => s.removeRecording);
  const renameRecording = useRecordingStore((s) => s.renameRecording);
  const quotaExceeded = useRecordingStore((s) => s.quotaExceeded);

  const [elapsed, setElapsed] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [playingId, setPlayingId] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);

  const ref = useRef<HTMLDivElement>(null);
  const session = useRef<ActiveRecording | null>(null);
  const startedAt = useRef(0);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  /** URL tạm của blob đang phát — phải thu hồi, không thì rò bộ nhớ. */
  const objectUrl = useRef<string | null>(null);

  const dragHandlers = useDraggable({ ref, pos, setPos });

  // Đặt vị trí mặc định ở lần mở đầu rồi kẹp lại khi đổi cỡ — cùng cách với
  // thanh công cụ vẽ, xem `annotation-toolbar`.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    function fit() {
      if (!el) return;
      const { offsetWidth: w, offsetHeight: h } = el;
      if (w === 0) return;
      const store = useRecordingStore.getState();
      const next = clampToViewport(store.panelPos ?? recorderAnchor(w, h), w, h);
      if (next.x !== store.panelPos?.x || next.y !== store.panelPos?.y) {
        store.setPanelPos(next);
      }
    }
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, [open]);

  const stopPlayback = useCallback(() => {
    audioRef.current?.pause();
    setPlayingId(null);
  }, []);

  // Đóng bảng hay lật trang giữa chừng: nhả micro và dừng phát. Không nhả thì
  // đèn "đang ghi âm" của trình duyệt sáng mãi.
  useEffect(() => {
    return () => {
      session.current?.cancel();
      session.current = null;
      audioRef.current?.pause();
      if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
    };
  }, []);

  // Đồng hồ lúc đang thu.
  useEffect(() => {
    if (elapsed === null) return;
    const id = setInterval(
      () => setElapsed(Date.now() - startedAt.current),
      TICK_MS
    );
    return () => clearInterval(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ cần biết đang thu hay không, không chạy lại mỗi nhịp
  }, [elapsed === null]);

  const beginRecording = useCallback(async () => {
    setError(null);
    setBusy(true);
    stopPlayback();
    // Đang nghe tự động thì dừng lại: micro sẽ thu lẫn tiếng bài nghe, và hết
    // bài là trang tự lật — bảng này dựng lại theo trang, bản đang ghi đứt.
    pauseAutoplay();
    try {
      session.current = await startRecording();
      startedAt.current = Date.now();
      setElapsed(0);
    } catch (err) {
      setError(RECORDER_ERROR_MESSAGE[toRecorderError(err)]);
    } finally {
      setBusy(false);
    }
  }, [stopPlayback]);

  const finishRecording = useCallback(async () => {
    const active = session.current;
    if (!active) return;
    session.current = null;
    setBusy(true);
    const durationMs = Date.now() - startedAt.current;
    setElapsed(null);
    try {
      const { blob, mimeType } = await active.stop();
      if (durationMs < MIN_DURATION_MS || blob.size === 0) return;

      const id = crypto.randomUUID();
      // Ghi tiếng TRƯỚC rồi mới thêm vào danh sách: ngược lại thì danh sách
      // có một mục bấm vào không ra gì.
      if (!(await putBlob(id, blob))) {
        setError("Không lưu được bản ghi — bộ nhớ trình duyệt bị chặn hoặc đã đầy.");
        return;
      }
      addRecording(bookId, page, {
        id,
        label: "",
        durationMs,
        mimeType,
        createdAt: new Date().toISOString(),
      });
      recordRecording();
    } finally {
      setBusy(false);
    }
  }, [addRecording, recordRecording, bookId, page]);

  const play = useCallback(
    async (rec: Recording) => {
      if (playingId === rec.id) {
        stopPlayback();
        return;
      }
      const blob = await getBlob(rec.id, rec.mimeType);
      if (!blob) {
        setError("Không tìm thấy tiếng của bản ghi này.");
        return;
      }
      if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
      objectUrl.current = URL.createObjectURL(blob);
      const audio = (audioRef.current ??= new Audio());
      audio.onended = () => setPlayingId(null);
      audio.src = objectUrl.current;
      setPlayingId(rec.id);
      // Nghe lại giọng mình mà bài nghe tự động vẫn chạy thì 2 tiếng chồng nhau.
      pauseAutoplay();
      void audio.play();
    },
    [playingId, stopPlayback]
  );

  if (!open) return null;

  const recording = elapsed !== null;
  const items = list ?? [];

  return (
    // z-[55]: trên thanh đọc và thanh vẽ, dưới panel bài giảng (z-[60]).
    <div
      ref={ref}
      {...dragHandlers}
      className="fixed z-[55] w-72 max-w-[calc(100vw-1rem)] touch-none select-none overflow-hidden rounded-2xl bg-neutral-900/85 text-white ring-1 ring-white/15 shadow-2xl backdrop-blur-md"
      style={{ left: pos?.x ?? 0, top: pos?.y ?? 0, opacity: pos ? 1 : 0 }}
    >
      <div className="flex cursor-grab items-center gap-1.5 px-2 py-1.5 active:cursor-grabbing">
        <GripVertical className="size-4 shrink-0 text-white/40" aria-hidden />
        <span className="min-w-0 flex-1 truncate text-xs font-medium">
          Ghi âm — Trang {page}
        </span>
        <Button
          variant="ghost"
          size="icon"
          className="size-7 rounded-full text-white hover:bg-white/15 hover:text-white"
          onClick={() => setOpen(false)}
          aria-label="Đóng bảng ghi âm"
        >
          <X className="size-4" aria-hidden />
        </Button>
      </div>

      <Separator className="bg-white/10" />

      <div className="flex items-center justify-center gap-3 py-3">
        <Button
          size="icon"
          disabled={busy}
          onClick={recording ? finishRecording : beginRecording}
          aria-label={recording ? "Dừng ghi âm" : "Bắt đầu ghi âm"}
          className={cn(
            "size-12 rounded-full transition-colors",
            recording
              ? "bg-red-600 text-white hover:bg-red-600/90"
              : "bg-white text-neutral-900 hover:bg-white/90"
          )}
        >
          {recording ? (
            <Square className="size-5 fill-current" aria-hidden />
          ) : (
            <Mic className="size-5" aria-hidden />
          )}
        </Button>
        <div className="min-w-14">
          <p className="font-mono text-lg tabular-nums" aria-live="off">
            {formatDuration(elapsed ?? 0)}
          </p>
          <p className="text-[11px] text-white/50">
            {recording ? "Đang ghi…" : "Nhấn để ghi"}
          </p>
        </div>
      </div>

      {error && (
        <p className="flex items-start gap-1.5 px-3 pb-2 text-[11px] text-red-300">
          <AlertTriangle className="mt-px size-3.5 shrink-0" aria-hidden />
          {error}
        </p>
      )}
      {quotaExceeded && !error && (
        <p className="flex items-start gap-1.5 px-3 pb-2 text-[11px] text-red-300">
          <AlertTriangle className="mt-px size-3.5 shrink-0" aria-hidden />
          Chưa lưu được danh sách bản ghi — bộ nhớ trình duyệt bị chặn hoặc đã đầy.
        </p>
      )}

      <Separator className="bg-white/10" />

      {items.length === 0 ? (
        <p className="px-3 py-4 text-center text-xs text-white/50">
          Trang này chưa có bản ghi nào.
        </p>
      ) : (
        <ul className="max-h-56 overflow-y-auto py-1">
          {items.map((rec, i) => (
            <li key={rec.id} className="flex items-center gap-1 px-1.5 py-0.5">
              <Button
                variant="ghost"
                size="icon"
                className="size-8 shrink-0 rounded-full text-white hover:bg-white/15 hover:text-white"
                onClick={() => void play(rec)}
                aria-label={
                  playingId === rec.id
                    ? `Tạm dừng ${recordingLabel(rec, i)}`
                    : `Nghe ${recordingLabel(rec, i)}`
                }
              >
                {playingId === rec.id ? (
                  <Pause className="size-4" aria-hidden />
                ) : (
                  <Play className="size-4" aria-hidden />
                )}
              </Button>

              {editingId === rec.id ? (
                <Input
                  autoFocus
                  defaultValue={rec.label}
                  placeholder={recordingLabel(rec, i)}
                  aria-label="Tên bản ghi"
                  // `select-text touch-auto`: cả bảng đặt `select-none` và
                  // `touch-none` để kéo thả, nhưng ô nhập thì phải bôi chọn
                  // và đặt con trỏ được, không thì sửa tên bản ghi rất khó.
                  className="h-8 flex-1 touch-auto border-white/20 bg-white/10 text-sm text-white select-text placeholder:text-white/40"
                  onBlur={(e) => {
                    renameRecording(bookId, page, rec.id, e.target.value);
                    setEditingId(null);
                  }}
                  onKeyDown={(e) => {
                    e.stopPropagation();
                    if (e.key === "Enter") e.currentTarget.blur();
                    if (e.key === "Escape") setEditingId(null);
                  }}
                />
              ) : (
                <button
                  type="button"
                  onClick={() => setEditingId(rec.id)}
                  className="group flex min-w-0 flex-1 items-center gap-1 rounded px-1 py-1 text-left hover:bg-white/10"
                  aria-label={`Đổi tên ${recordingLabel(rec, i)}`}
                >
                  <span className="min-w-0 flex-1 truncate text-sm">
                    {recordingLabel(rec, i)}
                  </span>
                  <Pencil
                    className="size-3 shrink-0 text-white/0 transition-colors group-hover:text-white/50"
                    aria-hidden
                  />
                </button>
              )}

              <span className="shrink-0 text-[11px] tabular-nums text-white/50">
                {formatDuration(rec.durationMs)}
              </span>
              <Button
                variant="ghost"
                size="icon"
                className="size-8 shrink-0 rounded-full text-white/70 hover:bg-white/15 hover:text-white"
                onClick={() => {
                  if (playingId === rec.id) stopPlayback();
                  removeRecording(bookId, page, rec.id);
                }}
                aria-label={`Xoá ${recordingLabel(rec, i)}`}
              >
                <Trash2 className="size-4" aria-hidden />
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
