"use client";

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

const MARKS = ["①", "②", "③", "④"];
const FILLED = ["❶", "❷", "❸", "❹"];

/**
 * HTML của đề — ĐÃ LÀM SẠCH lúc nhập (`lib/exam-html`: chỉ <br> <b> <u>
 * <div class="exam-box"> <img>), nên mới dám đưa thẳng vào DOM.
 *
 * Khung `.exam-box` là ô viền quanh đoạn văn như trên đề in. Ảnh luôn có nền
 * trắng: tranh/biểu đồ của đề vẽ trên nền giấy, để nền tối là lộ mép.
 */
export function ExamHtml({ html, className }: { html: string; className?: string }) {
  if (!html.trim()) return null;
  return (
    <div
      className={cn(
        "font-korean leading-relaxed break-keep",
        "[&_.exam-box]:my-2 [&_.exam-box]:rounded-md [&_.exam-box]:border [&_.exam-box]:border-border [&_.exam-box]:px-3 [&_.exam-box]:py-2",
        "[&_img]:mx-auto [&_img]:my-2 [&_img]:h-auto [&_img]:max-w-full [&_img]:rounded-md [&_img]:bg-white",
        className
      )}
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

/** Đề câu hỏi (đã bỏ "(N점)" lặp lại — xem `questionPrompt`); không có gì thì không hiện. */
export function PromptView({ exam, question, className }: { exam: Exam; question: ExamQuestion; className?: string }) {
  const content = questionPrompt(question.prompt);
  if (!content) return null;
  return <ContentView exam={exam} content={content} alt={`Câu ${question.no}`} className={className} />;
}

/**
 * Khối "※ [a~b]": lời chỉ dẫn, câu mẫu <보기> (đã có sẵn đáp án, tô đen như
 * đề in) và đoạn văn dùng chung cho các câu trong khối.
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
  const { range, text } = splitInstruction(group.instruction);
  return (
    <div className="flex flex-col gap-3">
      <p className="font-korean text-[0.95rem] leading-relaxed break-keep text-foreground/85">
        {range && showRange && (
          <span className="mr-2 inline-block rounded bg-muted px-1.5 py-0.5 align-[1px] font-sans text-xs font-medium text-muted-foreground tabular-nums">
            Câu {range}
          </span>
        )}
        {text}
      </p>
      {group.example && (
        <div className="relative rounded-lg border border-border px-4 pt-5 pb-3">
          <span className="absolute -top-2.5 left-1/2 -translate-x-1/2 bg-background px-2 text-xs font-korean text-muted-foreground">
            〈보 기〉
          </span>
          <ExamHtml html={group.example.html} className="text-sm" />
          <OptionsGrid layout={group.example.layout} count={group.example.options.length}>
            {group.example.options.map((o, i) => (
              <span key={i} className={cn("font-korean text-sm", i + 1 === group.example!.answer && "font-semibold")}>
                {i + 1 === group.example!.answer ? FILLED[i] : MARKS[i]} {o}
              </span>
            ))}
          </OptionsGrid>
        </div>
      )}
      {group.passage && (
        <div className="rounded-lg border border-border px-4 py-3">
          <ExamHtml html={group.passage} className="text-[0.95rem]" />
        </div>
      )}
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
