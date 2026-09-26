"use client";

import Link from "next/link";
import { getBook } from "@/lib/books";
import { MIN_SAMPLES, type Ability, type RedoItem, type WeakGrammar } from "@/lib/ability";
import { shortBookName } from "@/lib/companion";

const pct = (score: number) => Math.round(score * 100);
const bookName = (id: string) => {
  const book = getBook(id);
  return book ? shortBookName(book) : id;
};

/**
 * Tỉ lệ làm đúng theo kỹ năng — thanh ngang, MỘT màu, thứ tự cố định. Kỹ
 * năng chưa đủ lượt chấm thì không vẽ thanh: 1 bài đúng hết mà hiện "100%"
 * là nói quá, nên chỉ nói còn thiếu bao nhiêu lượt.
 */
export function SkillBars({ skills }: { skills: Ability["skills"] }) {
  return (
    <ul className="flex flex-col gap-3" aria-label="Tỉ lệ làm đúng theo kỹ năng">
      {skills.map((s) => {
        const enough = s.count >= MIN_SAMPLES && s.score !== null;
        return (
          <li
            key={s.skill}
            className="grid grid-cols-[4.5rem_1fr_3rem] items-center gap-3"
            title={`${s.vi} (${s.skill}) — ${s.count} lượt chấm`}
          >
            <span className="text-sm whitespace-nowrap">{s.vi}</span>
            {enough ? (
              <span className="h-1.5 overflow-hidden rounded-full bg-muted">
                <span
                  className="block h-full rounded-full bg-primary"
                  style={{ width: `${Math.max(2, pct(s.score!))}%` }}
                />
              </span>
            ) : (
              <span className="text-xs text-muted-foreground">
                {s.count === 0
                  ? "Chưa có bài nào"
                  : `Chấm thêm ${MIN_SAMPLES - s.count} bài để xem`}
              </span>
            )}
            <span className="text-right text-sm tabular-nums">
              {enough ? `${pct(s.score!)}%` : ""}
            </span>
          </li>
        );
      })}
    </ul>
  );
}

/** Điểm ngữ pháp hay sai, kèm nghĩa và đường tới trang giải thích. */
export function WeakGrammarList({ items }: { items: WeakGrammar[] }) {
  return (
    <ul className="flex flex-col divide-y divide-border">
      {items.map((g) => (
        <li key={`${g.bookId}:${g.page}:${g.heading}`} className="flex flex-col gap-1 py-2.5">
          <p className="font-medium">{g.entry?.title ?? g.heading}</p>
          {g.entry && <p className="text-sm text-muted-foreground">{g.entry.vi}</p>}
          <p className="flex flex-wrap gap-x-4 text-sm">
            {g.entry && (
              <Link
                href={`/read/${g.entry.bookId}/${g.entry.page}`}
                className="text-primary underline underline-offset-4"
              >
                Xem giải thích
              </Link>
            )}
            <Link href={`/read/${g.bookId}/${g.page}`} className="text-primary underline underline-offset-4">
              Làm lại bài
            </Link>
            <span className="text-xs text-muted-foreground self-center tabular-nums">
              {g.count} lượt · đúng {pct(g.score)}%
            </span>
          </p>
        </li>
      ))}
    </ul>
  );
}

/**
 * Bài tự chấm còn sai. Số trang và tên bài đứng TRƯỚC tên sách: trên điện
 * thoại dòng bị cắt ở cuối, và phần bị cắt phải là phần ít cần nhất.
 */
export function RedoList({ items }: { items: RedoItem[] }) {
  return (
    <ul className="flex flex-col divide-y divide-border">
      {items.map((r) => (
        <li key={`${r.bookId}:${r.page}:${r.section}`}>
          <Link
            href={`/read/${r.bookId}/${r.page}`}
            className="flex items-center justify-between gap-3 py-2 text-sm hover:text-primary"
          >
            <span className="min-w-0">
              <span className="block truncate">
                Trang {r.page} · {r.section}
              </span>
              <span className="block truncate text-xs text-muted-foreground">
                {bookName(r.bookId)}
              </span>
            </span>
            <span className="shrink-0 rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
              {r.grade === 0 ? "Sai nhiều" : "Sai vài câu"}
            </span>
          </Link>
        </li>
      ))}
    </ul>
  );
}
