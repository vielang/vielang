"use client";

import { useEffect, useSyncExternalStore } from "react";
import Link from "next/link";
import { ArrowLeft, ArrowRight, Check, Clock, List } from "lucide-react";
import { Button } from "@/components/ui/button";
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

const NO_SUBSCRIBE = () => () => {};

/*
 * Bài học có RẤT nhiều tên hàm, tên kiểu viết trong `code` giữa dòng. Mặc
 * định plugin typography chèn dấu backtick vào trước/sau nên đọc rất rối —
 * bỏ đi và thay bằng nền xám, còn `code` nằm trong khối `pre` thì giữ nguyên
 * (khối code đã được Shiki tô màu, xem globals.css).
 */
const LESSON_PROSE_CLASS =
  "prose prose-sm dark:prose-invert max-w-none prose-headings:font-heading prose-headings:scroll-mt-20 " +
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
  total,
  no,
  lesson,
  prev,
  next,
}: {
  courseId: string;
  courseTitle: string;
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
  const done = isClient && getBookProgress(progressByBook, progressId).readPages.includes(no);

  // Mở bài nào thì đó là chỗ đang học — ghi lại để "Học tiếp" quay về đúng
  // bài. Chờ hydrate xong, không thì ghi đè lên tiến độ đọc từ storage.
  useEffect(() => {
    if (hasHydrated) setLastPage(progressId, no);
  }, [hasHydrated, progressId, no, setLastPage]);

  const toc = lesson.headings.filter((h) => h.level === 2);

  return (
    <article className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <Link
        href={`/it/${courseId}`}
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        {courseTitle}
      </Link>

      <header className="-mt-2 flex flex-col gap-2">
        <p className="text-xs text-muted-foreground tabular-nums">
          Bài {no}/{total}
        </p>
        <h1 className="font-heading text-2xl font-semibold tracking-tight">{lesson.title}</h1>
        <p className="inline-flex items-center gap-1 text-xs text-muted-foreground tabular-nums">
          <Clock className="size-3.5" aria-hidden />
          {lesson.minutes} phút đọc
        </p>
      </header>

      {toc.length > 1 && (
        <nav aria-label="Nội dung bài" className="rounded-xl border border-border bg-muted/40 px-4 py-3">
          <p className="inline-flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <List className="size-3.5" aria-hidden />
            Nội dung bài
          </p>
          <ol className="mt-2 flex flex-col gap-1 text-sm">
            {toc.map((h) => (
              <li key={h.id}>
                <a href={`#${h.id}`} className="text-muted-foreground hover:text-foreground">
                  {h.text}
                </a>
              </li>
            ))}
          </ol>
        </nav>
      )}

      {/* Nội dung do mình viết trong content/it, dựng sang HTML lúc build —
          không phải dữ liệu người dùng nhập. */}
      <div className={LESSON_PROSE_CLASS} dangerouslySetInnerHTML={{ __html: lesson.html }} />

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
  );
}
