"use client";

import Link from "next/link";
import { getBook } from "@/lib/books";
import { MIN_SAMPLES, type Ability } from "@/lib/ability";

const pct = (score: number) => Math.round(score * 100);

/**
 * Năng lực theo kỹ năng + điểm ngữ pháp cần ôn + bài cần làm lại.
 *
 * Thanh ngang, MỘT màu, thứ tự kỹ năng cố định: đại lượng là "nhiều hay ít"
 * giữa vài hạng mục, không có gì để phân biệt bằng màu. Kỹ năng chưa đủ
 * lượt chấm thì không vẽ thanh — 1 bài đúng hết mà hiện "100%" là nói quá.
 */
export function AbilityPanel({ ability }: { ability: Ability }) {
  return (
    <div className="flex flex-col gap-6">
      <ul className="flex flex-col gap-2.5" aria-label="Tỉ lệ làm đúng theo kỹ năng">
        {ability.skills.map((s) => {
          const enough = s.count >= MIN_SAMPLES && s.score !== null;
          return (
            <li key={s.skill} className="grid grid-cols-[5.5rem_1fr_auto] items-center gap-3">
              <span className="text-sm">
                {s.vi} <span className="font-korean text-xs text-muted-foreground">{s.skill}</span>
              </span>
              {enough ? (
                <span
                  className="h-2 overflow-hidden rounded-full bg-muted"
                  title={`${s.vi}: ${pct(s.score!)}% trên ${s.count} lượt chấm`}
                >
                  <span
                    className="block h-full rounded-full bg-primary"
                    style={{ width: `${Math.max(2, pct(s.score!))}%` }}
                  />
                </span>
              ) : (
                <span className="text-xs text-muted-foreground">
                  {s.count === 0 ? "Chưa có lượt chấm" : "Chưa đủ dữ liệu"}
                </span>
              )}
              <span className="text-right text-xs text-muted-foreground tabular-nums">
                {enough ? (
                  <>
                    <span className="text-sm font-medium text-foreground">{pct(s.score!)}%</span> ·{" "}
                    {s.count} lượt
                  </>
                ) : (
                  `${s.count}/${MIN_SAMPLES} lượt`
                )}
              </span>
            </li>
          );
        })}
      </ul>

      {ability.weakGrammar.length > 0 && (
        <div className="flex flex-col gap-2">
          <h3 className="text-sm font-medium">Điểm ngữ pháp nên ôn lại</h3>
          <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
            {ability.weakGrammar.slice(0, 8).map((g) => (
              <li
                key={`${g.bookId}:${g.page}:${g.heading}`}
                className="flex flex-wrap items-start justify-between gap-2 px-3 py-2.5"
              >
                <div className="min-w-0 flex-1">
                  <p className="font-korean font-medium">{g.entry?.title ?? g.heading}</p>
                  {g.entry && <p className="text-sm text-muted-foreground">{g.entry.vi}</p>}
                  <p className="mt-0.5 text-xs text-muted-foreground tabular-nums">
                    Tự chấm {g.count} lượt · trung bình {pct(g.score)}%
                  </p>
                </div>
                <div className="flex shrink-0 gap-3 text-sm">
                  {g.entry && (
                    <Link
                      href={`/read/${g.entry.bookId}/${g.entry.page}`}
                      className="text-primary underline-offset-4 hover:underline"
                    >
                      Xem giải thích
                    </Link>
                  )}
                  <Link
                    href={`/read/${g.bookId}/${g.page}`}
                    className="text-primary underline-offset-4 hover:underline"
                  >
                    Làm lại
                  </Link>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {ability.redo.length > 0 && (
        <div className="flex flex-col gap-2">
          <h3 className="text-sm font-medium">Bài tự chấm còn sai</h3>
          <ul className="flex flex-col divide-y divide-border rounded-lg border border-border">
            {ability.redo.slice(0, 8).map((r) => (
              <li key={`${r.bookId}:${r.page}:${r.section}`}>
                <Link
                  href={`/read/${r.bookId}/${r.page}`}
                  className="flex items-center justify-between gap-3 px-3 py-2 text-sm hover:bg-muted"
                >
                  <span className="min-w-0 truncate">
                    <span className="text-muted-foreground">
                      {getBook(r.bookId)?.titleVi ?? r.bookId} · trang {r.page} ·{" "}
                    </span>
                    <span className="font-korean">{r.section}</span>
                  </span>
                  <span className="shrink-0 text-xs text-muted-foreground">
                    {r.grade === 0 ? "Sai nhiều" : "Sai vài câu"}
                  </span>
                </Link>
              </li>
            ))}
            {ability.redo.length > 8 && (
              <li className="px-3 py-2 text-xs text-muted-foreground">
                …và {ability.redo.length - 8} bài nữa.
              </li>
            )}
          </ul>
        </div>
      )}
    </div>
  );
}
