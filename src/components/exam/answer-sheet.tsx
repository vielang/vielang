"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { qKey, type Answers, type ExamSection } from "@/lib/exams";

/** Bốn ô tròn của một câu trên phiếu — tô thẳng được. */
function Bubbles({
  no,
  value,
  onAnswer,
  size = "size-6",
}: {
  no: number;
  value: number | undefined;
  onAnswer: (no: number, choice: number) => void;
  size?: string;
}) {
  return (
    <span className="flex gap-1" role="radiogroup" aria-label={`Câu ${no}`}>
      {[1, 2, 3, 4].map((c) => (
        <button
          key={c}
          type="button"
          role="radio"
          aria-checked={value === c}
          aria-label={`Câu ${no}: chọn ${c}`}
          onClick={() => onAnswer(no, c)}
          className={cn(
            "flex items-center justify-center rounded-full text-[0.65rem] tabular-nums transition-colors",
            size,
            value === c
              ? "bg-foreground text-background"
              : "border border-foreground/20 text-muted-foreground hover:border-foreground/50"
          )}
        >
          {c}
        </button>
      ))}
    </span>
  );
}

/**
 * Phiếu trả lời kiểu OMR: mỗi câu một hàng bốn ô tròn — tô thẳng trên phiếu
 * được, bấm số câu để cuộn đề tới câu đó. `current`: câu đang đọc (tô hàng).
 */
export function AnswerSheet({
  section,
  answers,
  onAnswer,
  onJump,
  current,
  columns,
}: {
  section: ExamSection;
  answers: Answers;
  onAnswer: (no: number, choice: number) => void;
  onJump: (no: number) => void;
  current?: number;
  /** Chia hai cột (phiếu trên điện thoại). */
  columns?: boolean;
}) {
  return (
    <ol className={cn("flex flex-col gap-0.5", columns && "grid grid-cols-2 gap-x-4")}>
      {section.questions.map((q) => {
        const a = answers[qKey(section.id, q.no)];
        return (
          <li
            key={q.no}
            data-no={q.no}
            className={cn("flex items-center gap-2 rounded-md py-0.5 pr-1", q.no === current && "bg-muted")}
          >
            <button
              type="button"
              onClick={() => onJump(q.no)}
              aria-label={`Tới câu ${q.no}`}
              className={cn(
                "w-7 shrink-0 text-right text-xs tabular-nums hover:underline",
                q.no === current ? "font-semibold text-foreground" : a ? "text-foreground" : "text-muted-foreground"
              )}
            >
              {q.no}
            </button>
            <Bubbles no={q.no} value={a} onAnswer={onAnswer} />
          </li>
        );
      })}
    </ol>
  );
}

/** Cuộn đề tới câu `no` (chừa chỗ cho thanh dính trên đầu — `scroll-mt` của câu). */
export function jumpToQuestion(no: number) {
  const el = document.getElementById(`q-${no}`);
  if (!el) return;
  const margin = parseFloat(getComputedStyle(el).scrollMarginTop) || 0;
  window.scrollTo({ top: window.scrollY + el.getBoundingClientRect().top - margin, behavior: "instant" });
}

/**
 * Câu đang đọc: câu đầu tiên còn thấy được phía dưới thanh dính trên đầu
 * trang (`topInset` px). Tính lại khi cuộn / đổi kích thước.
 */
function useReadingQuestion(nos: number[], topInset: number): number | undefined {
  const [current, setCurrent] = useState<number | undefined>(nos[0]);
  useEffect(() => {
    let raf = 0;
    const measure = () => {
      raf = 0;
      for (const no of nos) {
        const el = document.getElementById(`q-${no}`);
        if (el && el.getBoundingClientRect().bottom > topInset + 40) {
          setCurrent(no);
          return;
        }
      }
    };
    const onScroll = () => {
      if (!raf) raf = requestAnimationFrame(measure);
    };
    measure();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
    };
  }, [nos, topInset]);
  return current;
}

type Snap = "peek" | "half" | "full";
const PEEK = 92;

function heightOf(snap: Snap, vh: number): number {
  return snap === "peek" ? PEEK : snap === "half" ? Math.round(vh * 0.42) : Math.round(vh * 0.8);
}

/**
 * Phiếu trả lời trên ĐIỆN THOẠI: gắn ở đáy màn hình, kéo tay cầm lên xuống
 * giữa ba nấc — thu gọn (một dòng: câu đang đọc + bốn ô tròn), nửa màn hình
 * (đề cuộn ở trên, phiếu ở dưới — vừa đọc vừa tô), mở hết. Thả tay thì hít
 * về nấc gần nhất; bấm tay cầm thì mở / thu gọn.
 *
 * Vào thi thử thì phiếu THU GỌN — nhường chỗ cho đề; cần thì người học tự
 * kéo lên. Nấc đã chọn giữ nguyên suốt lượt thi (kể cả khi sang phần sau).
 *
 * Phiếu tự theo câu đang đọc (tô hàng, cuộn tới hàng đó); bấm số câu thì đề
 * cuộn tới câu. Trang được chừa khoảng trống dưới cùng bằng chiều cao phiếu
 * để câu cuối không bị che. Máy tính dùng phiếu dính bên phải (`AnswerSheet`).
 */
export function MobileAnswerSheet({
  section,
  answers,
  onAnswer,
  topInset = 140,
}: {
  section: ExamSection;
  answers: Answers;
  onAnswer: (no: number, choice: number) => void;
  /** Chiều cao thanh dính trên đầu trang (để biết câu nào đang đọc). */
  topInset?: number;
}) {
  const [snap, setSnap] = useState<Snap>("peek");
  const [vh, setVh] = useState(() => window.innerHeight);
  const [drag, setDrag] = useState<number | null>(null);
  const start = useRef<{ y: number; h: number; moved: boolean } | null>(null);
  const list = useRef<HTMLDivElement>(null);
  const nos = useMemo(() => section.questions.map((q) => q.no), [section]);
  const current = useReadingQuestion(nos, topInset);

  useEffect(() => {
    const onResize = () => setVh(window.innerHeight);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  const height = drag ?? heightOf(snap, vh);

  // Giữ hàng của câu đang đọc trong tầm nhìn của phiếu.
  useEffect(() => {
    const box = list.current;
    const row = box?.querySelector<HTMLElement>(`[data-no="${current}"]`);
    if (!box || !row) return;
    const top = row.offsetTop - box.offsetTop;
    if (top < box.scrollTop || top + row.offsetHeight > box.scrollTop + box.clientHeight) {
      box.scrollTo({ top: top - box.clientHeight / 3, behavior: "smooth" });
    }
  }, [current, snap]);

  const onPointerDown = (e: React.PointerEvent) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    start.current = { y: e.clientY, h: height, moved: false };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const s = start.current;
    if (!s) return;
    const dy = s.y - e.clientY;
    if (Math.abs(dy) > 6) s.moved = true;
    if (s.moved) setDrag(Math.max(PEEK, Math.min(heightOf("full", vh), s.h + dy)));
  };
  const onPointerUp = () => {
    const s = start.current;
    start.current = null;
    if (!s) return;
    if (!s.moved) setSnap(snap === "peek" ? "half" : "peek");
    else {
      const h = drag ?? s.h;
      const nearest = (["peek", "half", "full"] as Snap[]).reduce((a, b) =>
        Math.abs(heightOf(b, vh) - h) < Math.abs(heightOf(a, vh) - h) ? b : a
      );
      setSnap(nearest);
    }
    setDrag(null);
  };

  const answered = section.questions.filter((q) => answers[qKey(section.id, q.no)] !== undefined).length;
  const open = height > PEEK + 20;

  return (
    <>
      {/* Chừa chỗ để câu cuối cuộn lên được trên phiếu. */}
      <div
        aria-hidden
        className="lg:hidden"
        style={{ height: `calc(${heightOf(snap, vh) + 16}px + env(safe-area-inset-bottom))` }}
      />
      <section
        aria-label="Phiếu trả lời"
        className={cn(
          "fixed inset-x-0 bottom-0 z-30 flex flex-col rounded-t-2xl border-t border-border bg-background shadow-[0_-8px_24px_-12px_rgb(0_0_0/0.25)] lg:hidden",
          drag === null && "transition-[height] duration-200 ease-out"
        )}
        // Cao thêm đúng vùng an toàn dưới đáy (vạch Home trên iOS) — xem
        // `viewportFit` ở app/layout.
        style={{ height: `calc(${height}px + env(safe-area-inset-bottom))`, paddingBottom: "env(safe-area-inset-bottom)" }}
      >
        <div
          role="button"
          tabIndex={0}
          aria-label={open ? "Thu gọn phiếu trả lời" : "Mở phiếu trả lời"}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              setSnap(snap === "peek" ? "half" : "peek");
            }
          }}
          className="flex shrink-0 cursor-grab touch-none flex-col items-center gap-1 px-4 pt-1.5 pb-2 select-none active:cursor-grabbing"
        >
          <span className="h-1 w-10 rounded-full bg-foreground/20" aria-hidden />
          <span className="flex w-full items-center justify-between gap-3 text-sm">
            <span className="font-medium">
              Phiếu trả lời <span className="font-normal text-muted-foreground tabular-nums">{answered}/{nos.length}</span>
            </span>
            {open ? (
              <ChevronDown className="size-4 text-muted-foreground" aria-hidden />
            ) : (
              <ChevronUp className="size-4 text-muted-foreground" aria-hidden />
            )}
          </span>
        </div>

        {!open && current !== undefined ? (
          // Thu gọn: vẫn tô được câu đang đọc ngay trên thanh.
          <div className="-mt-1 flex items-center gap-2 px-4 pb-2" onPointerDown={(e) => e.stopPropagation()}>
            <span className="text-xs font-semibold tabular-nums">Câu {current}</span>
            <Bubbles no={current} value={answers[qKey(section.id, current)]} onAnswer={onAnswer} size="size-7" />
          </div>
        ) : (
          <div ref={list} className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 pb-[max(0.75rem,env(safe-area-inset-bottom))]">
            <AnswerSheet
              section={section}
              answers={answers}
              onAnswer={onAnswer}
              onJump={(no) => {
                // Phiếu đang mở hết thì hạ về nửa màn hình cho thấy đề.
                if (snap === "full") setSnap("half");
                jumpToQuestion(no);
              }}
              current={current}
              columns
            />
          </div>
        )}
      </section>
    </>
  );
}
