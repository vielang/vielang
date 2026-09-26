"use client";

import Link from "next/link";
import { getBook } from "@/lib/books";
import { shortBookName } from "@/lib/companion";
import { getPageQuiz, isCorrect, isGradable } from "@/lib/quiz";
import type { PageAnswers } from "@/lib/quiz-store";

export interface WrongItem {
  bookId: string;
  page: number;
  /** "듣기 · 1)" — mục và số câu như in trong sách. */
  where: string;
  prompt: string;
}

export interface QuizSummary {
  checked: number;
  correct: number;
  wrong: WrongItem[];
}

/**
 * Tổng hợp bài làm HIỆN TẠI trong `quiz-store`: mỗi câu tính một lần theo đáp
 * án đang lưu. Khác số lượt chấm theo ngày ở lịch sử học (làm lại một câu
 * sai ba lần thì lịch sử ghi ba lượt, còn ở đây vẫn là một câu).
 */
export function summarizeQuiz(pages: Record<string, PageAnswers>): QuizSummary {
  const summary: QuizSummary = { checked: 0, correct: 0, wrong: [] };
  for (const [key, { answers, checked }] of Object.entries(pages)) {
    const [bookId, pageStr] = key.split(":");
    const page = Number(pageStr);
    for (const section of getPageQuiz(bookId, page)) {
      for (const item of section.items) {
        if (!isGradable(item) || !checked.includes(item.id)) continue;
        summary.checked++;
        if (isCorrect(item, answers[item.id])) {
          summary.correct++;
        } else {
          summary.wrong.push({
            bookId,
            page,
            where: [section.title, item.label].filter(Boolean).join(" · "),
            prompt: item.prompt,
          });
        }
      }
    }
  }
  summary.wrong.sort((a, b) => a.bookId.localeCompare(b.bookId) || a.page - b.page);
  return summary;
}

/** Câu làm sai, bấm vào là tới đúng trang để làm lại. */
export function WrongList({ wrong }: { wrong: WrongItem[] }) {
  if (wrong.length === 0) return null;
  return (
    <ul className="flex flex-col divide-y divide-border">
      {wrong.slice(0, 10).map((w, i) => (
        <li key={i}>
          <Link
            href={`/read/${w.bookId}/${w.page}`}
            className="flex flex-col gap-0.5 py-2 text-sm hover:text-primary"
          >
            <span className="truncate">{w.prompt}</span>
            <span className="truncate text-xs text-muted-foreground">
              Trang {w.page} · {w.where} ·{" "}
              {(() => {
                const book = getBook(w.bookId);
                return book ? shortBookName(book) : w.bookId;
              })()}
            </span>
          </Link>
        </li>
      ))}
      {wrong.length > 10 && (
        <li className="py-2 text-xs text-muted-foreground">
          …và {wrong.length - 10} câu nữa.
        </li>
      )}
    </ul>
  );
}
