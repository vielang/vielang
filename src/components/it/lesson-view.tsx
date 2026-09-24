"use client";

import { useEffect, useSyncExternalStore, type MouseEvent } from "react";
import Link from "next/link";
import { ArrowLeft, ArrowRight, Check, ChevronRight, Clock, List } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { QuizBody } from "@/components/quiz/quiz-body";
import {
  courseProgressId,
  lessonHref,
  lessonQuizId,
  type Lesson,
  type LessonOutline,
} from "@/lib/courses";
import { getBookProgress, useProgressStore } from "@/lib/progress-store";
import { useQuizAnswers } from "@/lib/quiz-store";
import { isGradable } from "@/lib/quiz";

const NO_SUBSCRIBE = () => () => {};

/**
 * Nút "Chép" trong khối code (markup dựng sẵn lúc build). Bắt click ở khung
 * nội dung thay vì gắn từng nút, vì HTML bài học là chuỗi dựng sẵn.
 */
function handleCopyClick(event: MouseEvent<HTMLDivElement>) {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-copy-code]");
  const code = button?.parentElement?.querySelector("pre");
  if (!button || !code) return;
  void navigator.clipboard.writeText(code.innerText).then(() => {
    button.textContent = "Đã chép";
    window.setTimeout(() => (button.textContent = "Chép"), 1500);
  });
}

/*
 * Bài học có RẤT nhiều tên hàm, tên kiểu viết trong `code` giữa dòng. Mặc
 * định plugin typography chèn dấu backtick vào trước/sau nên đọc rất rối —
 * bỏ đi và thay bằng nền xám, còn `code` nằm trong khối `pre` thì giữ nguyên
 * (khối code đã được Shiki tô màu, xem globals.css).
 */
const LESSON_PROSE_CLASS =
  "prose prose-base dark:prose-invert max-w-none prose-headings:font-heading prose-headings:scroll-mt-20 " +
  "prose-table:text-sm prose-pre:border prose-pre:border-border prose-pre:leading-relaxed " +
  // Khối trích dẫn trong bài học là hộp "Học xong bạn sẽ / Cần biết trước",
  // không phải lời ai đó nói — bỏ chữ nghiêng, thêm nền cho ra dáng cái hộp.
  "prose-blockquote:not-italic prose-blockquote:font-normal prose-blockquote:rounded-r-lg " +
  "prose-blockquote:bg-muted/40 prose-blockquote:py-2 prose-blockquote:pr-4 " +
  "prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:font-normal " +
  "prose-code:before:content-none prose-code:after:content-none " +
  "[&_pre_code]:bg-transparent [&_pre_code]:p-0";

/**
 * Một bài học dạng chữ: nội dung đã dựng sẵn thành HTML lúc build, kèm mục
 * lục trong bài, nút đánh dấu đã học và điều hướng bài trước/bài sau.
 *
 * Tiến độ dùng chung kho với sách (xem `courseProgressId`): số thứ tự bài
 * đóng vai "số trang".
 */
export function LessonView({
  courseId,
  courseTitle,
  moduleTitle,
  total,
  no,
  lesson,
  prev,
  next,
}: {
  courseId: string;
  courseTitle: string;
  /** Tên chương chứa bài, hiện trong đường dẫn khoá › chương. */
  moduleTitle: string;
  /** Tổng số bài của khoá, để hiện "Bài 3/11". */
  total: number;
  no: number;
  lesson: Lesson;
  prev: LessonOutline | null;
  next: LessonOutline | null;
}) {
  const isClient = useSyncExternalStore(NO_SUBSCRIBE, () => true, () => false);
  const progressByBook = useProgressStore((s) => s.books);
  const hasHydrated = useProgressStore((s) => s.hasHydrated);
  const markPageRead = useProgressStore((s) => s.markPageRead);
  const setLastPage = useProgressStore((s) => s.setLastPage);

  const progressId = courseProgressId(courseId);
  const readPages = isClient ? getBookProgress(progressByBook, progressId).readPages : [];
  const done = readPages.includes(no);

  // Làm xong mọi câu tự kiểm tra thì coi như đã học bài: tự đánh dấu, khỏi
  // bắt người học nhớ bấm nút. Nút bấm tay vẫn giữ cho bài không có quiz.
  const { checked } = useQuizAnswers(lessonQuizId(courseId, lesson.slug));
  const gradable = (lesson.quiz ?? []).flatMap((s) => s.items.filter(isGradable));
  const quizFinished = gradable.length > 0 && gradable.every((i) => checked.includes(i.id));
  useEffect(() => {
    if (hasHydrated && quizFinished && !done) markPageRead(progressId, no);
  }, [hasHydrated, quizFinished, done, markPageRead, progressId, no]);

  // Mở bài nào thì đó là chỗ đang học — ghi lại để "Học tiếp" quay về đúng
  // bài. Chờ hydrate xong, không thì ghi đè lên tiến độ đọc từ storage.
  useEffect(() => {
    if (hasHydrated) setLastPage(progressId, no);
  }, [hasHydrated, progressId, no, setLastPage]);

  const toc = lesson.headings.filter((h) => h.level === 2);

  // Đánh số ở ĐÂY chứ không gõ số vào tiêu đề trong file .md: chèn thêm một
  // mục là phải đánh số lại cả bài, kiểu gì cũng sót.
  const tocList = (
    <ol className="flex flex-col gap-1 text-sm">
      {toc.map((h, i) => (
        <li key={h.id} className="flex gap-2">
          <span className="w-5 shrink-0 text-right text-muted-foreground tabular-nums">{i + 1}.</span>
          <a href={`#${h.id}`} className="text-muted-foreground hover:text-foreground">
            {h.text}
          </a>
        </li>
      ))}
    </ol>
  );

  return (
    // Màn hình rộng: bài ở giữa, mục lục cố định ở cột phải. Màn hình hẹp:
    // một cột, mục lục gập lại thành một dòng bấm để mở.
    <div className="mx-auto w-full max-w-2xl lg:grid lg:max-w-none lg:grid-cols-[minmax(0,42rem)_11rem] lg:justify-center lg:gap-10">
      <article className="flex min-w-0 flex-col gap-6">
        <nav aria-label="Đường dẫn" className="flex min-w-0 items-center gap-1 text-sm text-muted-foreground">
          <Link href={`/it/${courseId}`} className="inline-flex shrink-0 items-center gap-1 hover:text-foreground">
            <ArrowLeft className="size-4" aria-hidden />
            {courseTitle}
          </Link>
          <ChevronRight className="size-3.5 shrink-0" aria-hidden />
          <span className="truncate">{moduleTitle}</span>
        </nav>

        <header className="-mt-2 flex flex-col gap-2">
          <div className="flex items-center gap-3 text-xs text-muted-foreground tabular-nums">
            <span className="shrink-0">
              Bài {no}/{total}
            </span>
            <Progress value={(readPages.length / total) * 100} className="h-1 flex-1" />
            <span className="inline-flex shrink-0 items-center gap-1">
              <Clock className="size-3.5" aria-hidden />
              {lesson.minutes} phút
            </span>
          </div>
          <h1 className="font-heading text-2xl font-semibold tracking-tight">{lesson.title}</h1>
        </header>

        {toc.length > 1 && (
          <details className="group rounded-xl border border-border bg-muted/40 px-4 py-3 lg:hidden">
            <summary className="flex cursor-pointer list-none items-center gap-1.5 text-sm text-muted-foreground">
              <List className="size-4" aria-hidden />
              Nội dung bài · {toc.length} mục
              <ChevronRight className="ml-auto size-4 transition-transform group-open:rotate-90" aria-hidden />
            </summary>
            <div className="mt-2">{tocList}</div>
          </details>
        )}

        {/* Nội dung do mình viết trong content/it, dựng sang HTML lúc build —
            không phải dữ liệu người dùng nhập. */}
        <div
          className={LESSON_PROSE_CLASS}
          onClick={handleCopyClick}
          dangerouslySetInnerHTML={{ __html: lesson.html }}
        />

        {lesson.quiz && (
          <section aria-label="Câu tự kiểm tra" className="border-t border-border pt-6">
            <QuizBody quizId={lessonQuizId(courseId, lesson.slug)} sections={lesson.quiz} numbered />
          </section>
        )}

        <footer className="flex flex-col gap-4 border-t border-border pt-6">
          <Button
            size="lg"
            variant={done ? "outline" : "default"}
            className="self-start"
            onClick={() => markPageRead(progressId, no)}
            disabled={done}
          >
            <Check className={cn("size-4", done && "text-foreground")} aria-hidden />
            {done ? "Đã học bài này" : "Đánh dấu đã học"}
          </Button>

          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            {prev ? (
              <Button asChild variant="outline" className="min-w-0 justify-start">
                <Link href={lessonHref(courseId, prev.slug)}>
                  <ArrowLeft className="size-4 shrink-0" aria-hidden />
                  <span className="min-w-0 truncate">{prev.title}</span>
                </Link>
              </Button>
            ) : (
              <span className="hidden sm:block" />
            )}
            {next && (
              <Button asChild variant="outline" className="min-w-0 justify-end sm:col-start-2">
                <Link href={lessonHref(courseId, next.slug)}>
                  <span className="min-w-0 truncate">{next.title}</span>
                  <ArrowRight className="size-4 shrink-0" aria-hidden />
                </Link>
              </Button>
            )}
          </div>
        </footer>
      </article>

      {toc.length > 1 && (
        <aside className="hidden lg:block">
          <nav aria-label="Nội dung bài" className="sticky top-20 flex flex-col gap-2">
            <p className="inline-flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
              <List className="size-3.5" aria-hidden />
              Nội dung bài
            </p>
            {tocList}
          </nav>
        </aside>
      )}
    </div>
  );
}
