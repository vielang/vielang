"use client";

import { ExternalLink } from "lucide-react";
import { cn } from "@/lib/utils";
import { countChars, examAssetUrl, textKey, type Exam, type Texts, type WritingTask } from "@/lib/exams";

const BLANKS = ["㉠", "㉡"];

/**
 * Trang đề / đáp án mẫu phần viết — chỉ có bản in (ảnh), không có chữ. Chữ
 * trên điện thoại khá nhỏ nên kèm liên kết mở ảnh gốc để phóng to.
 */
export function WritingImage({ exam, src, alt }: { exam: Exam; src: string; alt: string }) {
  const url = examAssetUrl(exam, src);
  return (
    <figure className="flex flex-col gap-1">
      {/* eslint-disable-next-line @next/next/no-img-element -- ảnh đề đã tối ưu sẵn (WebP) và đi qua rewrite cùng origin */}
      <img src={url} alt={alt} loading="lazy" className="h-auto w-full rounded-lg border border-border bg-white" />
      <a
        href={url}
        target="_blank"
        rel="noreferrer"
        className="inline-flex items-center gap-1 self-end text-xs text-muted-foreground hover:text-foreground"
      >
        Mở ảnh cỡ lớn
        <ExternalLink className="size-3" aria-hidden />
      </a>
    </figure>
  );
}

/**
 * Ô viết của một câu: 51–52 hai dòng ㉠ ㉡, 53–54 một ô bài viết có đếm chữ
 * theo yêu cầu độ dài của đề.
 */
export function WritingInput({
  task,
  texts,
  onChange,
  readOnly,
}: {
  task: WritingTask;
  texts: Texts;
  onChange: (key: string, text: string) => void;
  readOnly?: boolean;
}) {
  if (task.kind === "blanks") {
    return (
      <div className="flex flex-col gap-2">
        {BLANKS.map((mark, i) => {
          const key = textKey(task.no, i);
          return (
            <label key={key} className="flex items-center gap-2">
              <span className="font-korean text-lg" aria-hidden>
                {mark}
              </span>
              <input
                type="text"
                value={texts[key] ?? ""}
                onChange={(e) => onChange(key, e.target.value)}
                readOnly={readOnly}
                aria-label={`Câu ${task.no}, chỗ trống ${mark}`}
                lang="ko"
                className="h-10 min-w-0 flex-1 rounded-md border border-input bg-background px-3 font-korean text-[0.95rem] outline-none focus-visible:ring-2 focus-visible:ring-ring read-only:bg-muted/50"
              />
            </label>
          );
        })}
      </div>
    );
  }

  const key = textKey(task.no);
  const text = texts[key] ?? "";
  const n = countChars(text);
  const [min, max] = task.chars ?? [0, Infinity];
  return (
    <div className="flex flex-col gap-1.5">
      <textarea
        value={text}
        onChange={(e) => onChange(key, e.target.value)}
        readOnly={readOnly}
        rows={task.no === 54 ? 16 : 8}
        lang="ko"
        spellCheck={false}
        aria-label={`Bài viết câu ${task.no}`}
        className="w-full resize-y rounded-md border border-input bg-background px-3 py-2 font-korean text-[0.95rem] leading-relaxed outline-none focus-visible:ring-2 focus-visible:ring-ring read-only:bg-muted/50"
      />
      {Number.isFinite(max) && <LengthMeter n={n} min={min} max={max} />}
      <p
        className={cn(
          "self-end text-xs tabular-nums",
          n === 0 ? "text-muted-foreground" : n < min || n > max ? "text-amber-700 dark:text-amber-400" : "text-emerald-700 dark:text-emerald-400"
        )}
        aria-live="polite"
      >
        {n} ký tự · yêu cầu {min}–{max} (tính cả dấu cách)
      </p>
    </div>
  );
}

/**
 * Thanh độ dài bài viết: vùng nhạt là khoảng yêu cầu (vd 200–300), phần tô là
 * số chữ đã viết — thấy ngay còn thiếu hay đã quá.
 */
function LengthMeter({ n, min, max }: { n: number; min: number; max: number }) {
  const scale = max * 1.15;
  const pct = (x: number) => `${(Math.min(x, scale) / scale) * 100}%`;
  const ok = n >= min && n <= max;
  return (
    <div className="relative h-1.5 rounded-full bg-muted" aria-hidden>
      <div
        className="absolute inset-y-0 bg-emerald-500/20 dark:bg-emerald-400/20"
        style={{ left: pct(min), width: `calc(${pct(max)} - ${pct(min)})` }}
      />
      <div
        className={cn(
          "absolute inset-y-0 left-0 rounded-full transition-[width] duration-200",
          ok ? "bg-emerald-600 dark:bg-emerald-500" : n > max ? "bg-amber-600" : "bg-foreground/60"
        )}
        style={{ width: pct(n) }}
      />
    </div>
  );
}

/** Câu viết đã có chữ chưa (51–52: có ít nhất một chỗ trống). */
export function hasWritten(task: WritingTask, texts: Texts): boolean {
  const keys = task.kind === "blanks" ? [textKey(task.no, 0), textKey(task.no, 1)] : [textKey(task.no)];
  return keys.some((k) => (texts[k] ?? "").trim() !== "");
}

const RUBRIC: Record<WritingTask["kind"], string> = {
  blanks:
    "Mỗi chỗ trống khoảng 5 điểm: đúng ý và đúng ngữ pháp, văn phong thì trọn điểm; đúng ý nhưng sai chính tả/đuôi câu thì trừ bớt.",
  essay:
    "Chấm theo ba tiêu chí của TOPIK: nội dung & làm đúng yêu cầu đề, bố cục mạch lạc, dùng từ – ngữ pháp đa dạng và chính xác. Viết dưới độ dài yêu cầu hoặc dùng văn nói (-아요/-습니다) sẽ bị trừ điểm.",
};

/** Tự chấm một câu viết sau khi đối chiếu đáp án mẫu. */
export function SelfGrade({
  task,
  value,
  onChange,
}: {
  task: WritingTask;
  value: number | undefined;
  onChange: (points: number) => void;
}) {
  return (
    <div className="flex flex-col gap-2 rounded-lg bg-muted/60 p-3">
      <div className="flex items-center justify-between gap-3">
        <label htmlFor={`grade-${task.no}`} className="text-sm font-medium">
          Tự chấm câu {task.no}
        </label>
        <span className="text-sm tabular-nums">
          {value === undefined ? <span className="text-muted-foreground">chưa chấm</span> : <b>{value}</b>}
          <span className="text-muted-foreground">/{task.points} điểm</span>
        </span>
      </div>
      <input
        id={`grade-${task.no}`}
        type="range"
        min={0}
        max={task.points}
        step={1}
        value={value ?? 0}
        onChange={(e) => onChange(Number(e.target.value))}
        // Bấm vào thanh mà không kéo cũng tính là đã chấm.
        onClick={(e) => onChange(Number((e.target as HTMLInputElement).value))}
        className="w-full accent-primary"
      />
      <p className="text-xs leading-relaxed text-muted-foreground">{RUBRIC[task.kind]}</p>
    </div>
  );
}
