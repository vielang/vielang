"use client";

import { useState } from "react";
import Link from "next/link";
import { Check, CheckCircle2, ChevronRight, Grid3x3, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { cn } from "@/lib/utils";
import { examTitle, sectionVi, type Exam, type ExamSection } from "@/lib/exams";
import { BackLink } from "@/components/layout/back-link";

/**
 * Khung dùng chung của các màn luyện thi: đầu trang (về đề + chuyển phần +
 * tiến độ), bảng số câu, thanh thao tác, lời chấm. Mục tiêu: phẳng, ít viền,
 * chỉ tô màu cho đúng/sai.
 */

/** Thanh tiến độ mảnh. */
export function ProgressBar({ value, max, className }: { value: number; max: number; className?: string }) {
  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={value}
      className={cn("h-1 overflow-hidden rounded-full bg-muted", className)}
    >
      <div
        className="h-full rounded-full bg-foreground/70 transition-[width] duration-300"
        style={{ width: `${max ? (value / max) * 100 : 0}%` }}
      />
    </div>
  );
}

/** Vòng tiến độ nhỏ (vd đã luyện bao nhiêu câu của một đề). */
export function ProgressRing({ value, max, size = 32 }: { value: number; max: number; size?: number }) {
  const r = (size - 4) / 2;
  const c = 2 * Math.PI * r;
  const p = max ? value / max : 0;
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="-rotate-90" aria-hidden>
      <circle cx={size / 2} cy={size / 2} r={r} fill="none" strokeWidth={3} className="stroke-muted" />
      {p > 0 && (
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          strokeWidth={3}
          strokeLinecap="round"
          strokeDasharray={`${c * p} ${c}`}
          className="stroke-foreground/70"
        />
      )}
    </svg>
  );
}

/** Huy hiệu cấp đạt được ("Cấp 2"), hoặc "Chưa đạt". */
export function LevelBadge({
  level,
  emptyLabel = "Chưa đạt",
  className,
}: {
  level: string | null | undefined;
  emptyLabel?: string;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium whitespace-nowrap",
        level ? "bg-foreground text-background" : "bg-muted text-muted-foreground",
        className
      )}
    >
      {level ?? emptyLabel}
    </span>
  );
}

/** Đầu trang luyện tập: về trang đề, chuyển phần Nghe / Viết / Đọc, tiến độ phần đang luyện. */
export function PracticeHeader({
  exam,
  section,
  done,
  total,
  summary,
}: {
  exam: Exam;
  section: ExamSection;
  done: number;
  total: number;
  summary: string;
}) {
  return (
    <header className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <BackLink href={`/exam/${exam.id}`}>{examTitle(exam)}</BackLink>
        <nav aria-label="Phần thi" className="flex gap-0.5 rounded-lg bg-muted p-0.5 text-sm">
          {exam.sections.map((s) => (
            <Link
              key={s.id}
              href={`/exam/${exam.id}/practice?section=${s.id}`}
              aria-current={s.id === section.id ? "page" : undefined}
              className={cn(
                "rounded-md px-3 py-1 transition-colors",
                s.id === section.id
                  ? "bg-background font-medium shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {sectionVi(s.id)}
            </Link>
          ))}
        </nav>
      </div>
      <div className="flex items-center gap-3">
        <ProgressBar value={done} max={total} className="flex-1" />
        <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{summary}</span>
      </div>
    </header>
  );
}

/** Các phần của lượt thi thử: phần xong có dấu ✓, phần đang làm đậm. */
export function SectionStepper({ exam, current }: { exam: Exam; current: number }) {
  return (
    <ol className="flex items-center gap-1.5 text-xs" aria-label="Các phần thi">
      {exam.sections.map((s, i) => (
        <li key={s.id} className="flex items-center gap-1.5">
          {i > 0 && <span className="h-px w-3 bg-border" aria-hidden />}
          <span
            aria-current={i === current ? "step" : undefined}
            className={cn(
              "inline-flex items-center gap-0.5",
              i === current ? "font-semibold text-foreground" : "text-muted-foreground"
            )}
          >
            {i < current && <Check className="size-3" aria-label="đã xong" />}
            {sectionVi(s.id)}
          </span>
        </li>
      ))}
    </ol>
  );
}

// ---------------------------------------------------------------------------
// Bảng số câu

export type CellState = "idle" | "answered" | "right" | "wrong";

export interface GridItem {
  no: number;
  state: CellState;
  /** Chữ thêm sau số câu (vd điểm tự chấm câu viết). */
  suffix?: string;
  label?: string;
}

const CELL: Record<CellState, string> = {
  idle: "text-muted-foreground hover:bg-muted",
  answered: "bg-muted font-medium text-foreground",
  right: "bg-emerald-100 font-medium text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300",
  wrong: "bg-red-100 font-medium text-red-800 dark:bg-red-900/40 dark:text-red-300",
};

/** Lưới số câu: nền xám = đã chọn, xanh = đúng, đỏ = sai; câu đang xem có vòng viền. */
export function QuestionGrid({
  items,
  current,
  onPick,
  className,
}: {
  items: GridItem[];
  current: number;
  onPick: (index: number) => void;
  className?: string;
}) {
  const wide = items.some((x) => x.suffix);
  return (
    <div className={cn("flex flex-wrap gap-1", className)}>
      {items.map((x, i) => (
        <button
          key={x.no}
          type="button"
          onClick={() => onPick(i)}
          aria-current={i === current ? "step" : undefined}
          aria-label={x.label ?? `Câu ${x.no}`}
          className={cn(
            "flex h-8 items-center justify-center rounded-md text-xs tabular-nums transition-colors",
            wide ? "min-w-12 px-2" : "w-8",
            CELL[x.state],
            i === current && "ring-2 ring-foreground/70 ring-offset-1 ring-offset-background"
          )}
        >
          {x.no}
          {x.suffix}
        </button>
      ))}
    </div>
  );
}

/**
 * Bảng số câu trên điện thoại: gập vào một nút, bấm thì trượt lên từ đáy —
 * không chiếm nửa màn hình như lưới mở sẵn.
 */
export function QuestionGridSheet({
  items,
  current,
  onPick,
  count,
}: {
  items: GridItem[];
  current: number;
  onPick: (index: number) => void;
  count: string;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="ghost" size="lg" className="sm:hidden" onClick={() => setOpen(true)}>
        <Grid3x3 className="size-4" aria-hidden />
        <span className="tabular-nums">{count}</span>
      </Button>
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent side="bottom" className="max-h-[70vh] overflow-y-auto rounded-t-2xl px-4 pb-6">
          <SheetHeader className="px-0">
            <SheetTitle>Bảng câu · {count}</SheetTitle>
          </SheetHeader>
          <QuestionGrid
            items={items}
            current={current}
            onPick={(i) => {
              setOpen(false);
              onPick(i);
            }}
          />
          <GridLegend />
        </SheetContent>
      </Sheet>
    </>
  );
}

export function GridLegend({ className }: { className?: string }) {
  const dot = (c: string) => <span className={cn("inline-block size-2.5 rounded-sm", c)} aria-hidden />;
  return (
    <p className={cn("flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground", className)}>
      <span className="inline-flex items-center gap-1">{dot("bg-muted-foreground/30")} Đã chọn</span>
      <span className="inline-flex items-center gap-1">{dot("bg-emerald-500/60")} Đúng</span>
      <span className="inline-flex items-center gap-1">{dot("bg-red-500/60")} Sai</span>
    </p>
  );
}

/**
 * Thanh thao tác của màn luyện: dính đáy màn hình trên điện thoại (khỏi cuộn
 * tìm nút), nằm yên trong trang trên máy tính.
 */
export function ActionBar({ children }: { children: React.ReactNode }) {
  return (
    <div
      className={cn(
        "fixed inset-x-0 bottom-0 z-30 border-t border-border bg-background/95 px-4 pt-2 pb-[max(0.5rem,env(safe-area-inset-bottom))] backdrop-blur",
        "sm:static sm:z-auto sm:border-0 sm:bg-transparent sm:p-0 sm:backdrop-blur-none"
      )}
    >
      <div className="mx-auto flex max-w-2xl items-center justify-between gap-2">{children}</div>
    </div>
  );
}

/** Lời chấm một câu. */
export function Verdict({ right, children }: { right: boolean; children: React.ReactNode }) {
  const Icon = right ? CheckCircle2 : XCircle;
  return (
    <div
      role="status"
      className={cn(
        "flex items-start gap-2 rounded-lg px-3 py-2.5 text-sm",
        right
          ? "bg-emerald-50 text-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200"
          : "bg-red-50 text-red-900 dark:bg-red-950/40 dark:text-red-200"
      )}
    >
      <Icon className="mt-0.5 size-4 shrink-0" aria-hidden />
      <div className="min-w-0 flex-1">{children}</div>
    </div>
  );
}

/** Gợi ý phím tắt — chỉ hiện khi có chuột/bàn phím (màn cảm ứng thì vô nghĩa). */
export function KeyHint({ children }: { children: React.ReactNode }) {
  return <p className="hidden text-xs text-muted-foreground [@media(pointer:fine)]:block">{children}</p>;
}

/** "Câu 5–6" / "Câu 53" — nhãn khoảng câu của một khối. */
export function rangeLabel(from: number, to: number): string {
  return from === to ? `Câu ${from}` : `Câu ${from}–${to}`;
}

/**
 * Nút đi tiếp sau khi làm xong một khối: "Tiếp tục · Câu 5–6" (ghi rõ sẽ
 * sang đâu — mỗi lần bấm là cả một nhóm câu, không phải một câu). Ở khối
 * cuối thì sang phần kế tiếp ("Sang phần Đọc"), hết đề thì về trang đề.
 */
export function NextButton({
  exam,
  section,
  next,
  onNext,
}: {
  exam: Exam;
  section: ExamSection;
  /** Khoảng câu của khối kế tiếp; không có = đang ở khối cuối của phần. */
  next?: string;
  onNext: () => void;
}) {
  if (next) {
    return (
      <Button size="lg" onClick={onNext} className="min-w-32">
        Tiếp tục
        <span className="font-normal tabular-nums opacity-70">{next}</span>
        <ChevronRight className="size-4" aria-hidden />
      </Button>
    );
  }
  const i = exam.sections.findIndex((s) => s.id === section.id);
  const after = exam.sections[i + 1];
  return (
    <Button asChild size="lg" className="min-w-32">
      <Link href={after ? `/exam/${exam.id}/practice?section=${after.id}` : `/exam/${exam.id}`}>
        {after ? `Sang phần ${sectionVi(after.id)}` : "Về trang đề"}
        <ChevronRight className="size-4" aria-hidden />
      </Link>
    </Button>
  );
}
