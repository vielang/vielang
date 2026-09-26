"use client";

import { Lightbulb } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  examAssetUrl,
  isImage,
  questionPrompt,
  splitInstruction,
  type Content,
  type Exam,
  type ExamGroup,
  type ExamQuestion,
} from "@/lib/exams";

/**
 * HTML của đề — đề do VieLang tự soạn, nằm trong repo, và chỉ dùng <br> <b>
 * <u> <div class="exam-box"> <img> (exams.test soát từng đề), nên mới dám
 * đưa thẳng vào DOM.
 *
 * Khung `.exam-box` là ô viền quanh từng văn bản (e-mail, thông báo…) như
 * trên đề in. Ảnh luôn có nền trắng: tranh/biểu đồ vẽ trên nền giấy, để nền
 * tối là lộ mép.
 */
function ExamHtml({ html, className }: { html: string; className?: string }) {
  if (!html.trim()) return null;
  return (
    <div
      className={cn(
        "leading-relaxed",
        "[&_.exam-box]:my-2 [&_.exam-box]:rounded-md [&_.exam-box]:border [&_.exam-box]:border-border [&_.exam-box]:px-3 [&_.exam-box]:py-2",
        "[&_img]:mx-auto [&_img]:my-2 [&_img]:h-auto [&_img]:max-w-full [&_img]:rounded-md [&_img]:bg-white",
        className
      )}
      lang="en"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

/** Nội dung một ô của đề: chữ (HTML đã làm sạch) hoặc tranh. */
export function ContentView({
  exam,
  content,
  alt,
  className,
}: {
  exam: Exam;
  content: Content;
  alt: string;
  className?: string;
}) {
  if (isImage(content)) {
    return (
      // eslint-disable-next-line @next/next/no-img-element -- ảnh đề đã tối ưu sẵn (WebP) và đi qua rewrite cùng origin
      <img
        src={examAssetUrl(exam, content.image)}
        alt={content.alt ?? alt}
        loading="lazy"
        className={cn("mx-auto h-auto max-w-full rounded-md bg-white", className)}
      />
    );
  }
  return <ExamHtml html={content.html} className={className} />;
}

/** Đề câu hỏi; không có gì thì không hiện (câu Part 6 — chỗ trống nằm trong văn bản của khối). */
export function PromptView({ exam, question, className }: { exam: Exam; question: ExamQuestion; className?: string }) {
  const content = questionPrompt(question.prompt);
  if (!content) return null;
  return <ContentView exam={exam} content={content} alt={`Câu ${question.no}`} className={className} />;
}

/**
 * Đầu khối: nhãn phần ("Part 6") và khoảng câu, lời chỉ dẫn tiếng Anh như đề
 * in, rồi văn bản dùng chung (Part 6: văn bản có chỗ trống; Part 7: một, hai
 * hoặc ba văn bản — mỗi văn bản một khung `.exam-box`).
 */
export function GroupBlock({
  group,
  showRange = true,
}: {
  exam: Exam;
  group: ExamGroup;
  /** Hiện nhãn "Câu a–b" (tắt khi tiêu đề trang đã ghi khoảng câu). */
  showRange?: boolean;
}) {
  const { part, range, text } = splitInstruction(group.instruction);
  const tag = "mr-2 inline-block rounded bg-muted px-1.5 py-0.5 align-[1px] text-xs font-medium text-muted-foreground tabular-nums";
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[0.95rem] leading-relaxed text-foreground/85">
        {part && <span className={tag}>{part}</span>}
        {range && showRange && <span className={tag}>Câu {range}</span>}
        <span lang="en">{text}</span>
      </p>
      {group.passage && <ExamHtml html={group.passage} className="text-[0.95rem]" />}
    </div>
  );
}

/** Lời giải thích đáp án (tiếng Việt) — hiện sau khi kiểm tra câu và khi xem lại bài thi thử. */
export function Explanation({ question }: { question: ExamQuestion }) {
  if (!question.explanation) return null;
  return (
    <div className="flex items-start gap-2 rounded-lg bg-muted/60 px-3 py-2.5 text-sm leading-relaxed">
      <Lightbulb className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden />
      <p className="min-w-0 flex-1">
        <span className="sr-only">Giải thích: </span>
        {question.explanation}
      </p>
    </div>
  );
}

/**
 * Lưới lựa chọn theo số cột của đề gốc (1, 2 hoặc 4 mỗi hàng) — nhưng trên
 * điện thoại 4 cột chỉ còn 2, không thì chữ bị bóp.
 */
export function OptionsGrid({
  layout,
  count,
  children,
  image,
}: {
  layout: number;
  count: number;
  children: React.ReactNode;
  image?: boolean;
}) {
  const cols =
    image || layout === 2
      ? "grid-cols-2"
      : layout >= 4 && count >= 4
        ? "grid-cols-2 sm:grid-cols-4"
        : "grid-cols-1";
  return <div className={cn("mt-2 grid gap-2", cols)}>{children}</div>;
}
