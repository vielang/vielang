import { ChevronRight, List } from "lucide-react";
import type { LessonHeading } from "@/lib/courses";

/**
 * Bộ class `prose` gốc cho bài đọc dạng chữ (bài học IT, bài cẩm nang). Mỗi
 * loại bài nối thêm phần riêng của nó; phần chung để một chỗ cho hai loại bài
 * không trôi lệch nhau (trước đây lệch cả khoảng cách cuộn tới tiêu đề).
 *
 * `scroll-mt-24`: bấm mục lục thì tiêu đề dừng dưới header dính, không bị
 * header che mất.
 */
export const ARTICLE_PROSE_BASE =
  "prose prose-base dark:prose-invert max-w-none prose-headings:font-heading prose-headings:scroll-mt-24 prose-table:text-sm";

/**
 * Danh sách mục của bài. Đánh số ở ĐÂY chứ không gõ số vào tiêu đề trong
 * file .md: chèn thêm một mục là phải đánh số lại cả bài, kiểu gì cũng sót.
 */
function TocList({ headings }: { headings: LessonHeading[] }) {
  return (
    <ol className="flex flex-col gap-1 text-sm">
      {headings.map((h, i) => (
        <li key={h.id} className="flex gap-2">
          <span className="w-5 shrink-0 text-right text-muted-foreground tabular-nums">{i + 1}.</span>
          <a href={`#${h.id}`} className="text-muted-foreground hover:text-foreground">
            {h.text}
          </a>
        </li>
      ))}
    </ol>
  );
}

/** Màn hẹp: mục lục gập thành một dòng, bấm mới mở. Ẩn trên màn rộng (có cột bên). */
export function ArticleTocMobile({ headings }: { headings: LessonHeading[] }) {
  return (
    <details className="group rounded-lg bg-muted/40 px-3 py-2.5 lg:hidden">
      <summary className="flex cursor-pointer list-none items-center gap-1.5 text-sm text-muted-foreground">
        <List className="size-4" aria-hidden />
        Nội dung bài · {headings.length} mục
        <ChevronRight className="ml-auto size-4 transition-transform group-open:rotate-90" aria-hidden />
      </summary>
      <div className="mt-2">
        <TocList headings={headings} />
      </div>
    </details>
  );
}

/**
 * Màn rộng: mục lục dính ở cột phải. Đặt làm cột thứ hai của lưới
 * `lg:grid-cols-[minmax(0,42rem)_12rem]` ở trang bài.
 */
export function ArticleTocAside({ headings }: { headings: LessonHeading[] }) {
  return (
    <aside className="hidden lg:block">
      <nav aria-label="Nội dung bài" className="sticky top-24 flex flex-col gap-2">
        <p className="inline-flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <List className="size-3.5" aria-hidden />
          Nội dung bài
        </p>
        <TocList headings={headings} />
      </nav>
    </aside>
  );
}
