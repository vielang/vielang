import { Languages } from "lucide-react";
import { BookCard } from "@/components/library/book-card";
import { ContinueReading } from "@/components/library/continue-reading";
import type { Book } from "@/lib/books";
import { groupBooksByLevel } from "@/lib/library";
import type { LanguageConfig } from "@/lib/languages";

/**
 * Khung trang thư viện dùng chung cho trang gốc (tiếng Hàn) và mọi trang
 * `/[lang]` khác. Không có sách nào khớp `language` -> hiện empty-state
 * "Sắp có nội dung" thay vì lưới trống.
 */
export function LibraryView({
  language,
  books,
}: {
  language: LanguageConfig;
  books: readonly Book[];
}) {
  const groups = groupBooksByLevel(books);

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">
          {language.heading}
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          {language.description}
        </p>
      </div>

      {/* Đặt TRƯỚC lưới sách: mở thư viện ra phần lớn là để đọc tiếp cuốn
          đang dở, chứ không phải để chọn cuốn mới. Tự ẩn khi chưa đọc gì. */}
      <ContinueReading books={books} />

      {groups.length === 0 ? (
        <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed border-border py-24 text-center">
          <Languages className="size-10 text-muted-foreground" aria-hidden />
          <div>
            <h2 className="text-lg font-semibold">Sắp có nội dung</h2>
            <p className="mt-1 max-w-sm text-sm text-muted-foreground">
              Chưa có giáo trình {language.label.toLowerCase()} nào được thêm
              vào. Phần này sẽ hiển thị sách theo từng cấp độ, cùng định dạng
              với các ngôn ngữ khác.
            </p>
          </div>
        </div>
      ) : (
        <div className="flex min-w-0 flex-col gap-6">
          {groups.map((group) => (
            <section key={group.level} className="flex min-w-0 flex-col gap-3">
              <div className="flex items-baseline gap-2 border-b border-border pb-2">
                <h2 className="text-base font-semibold tracking-tight">
                  Cấp {group.level}
                </h2>
                {group.levelLabelKo && (
                  <span className="font-korean text-sm text-muted-foreground">
                    {group.levelLabelKo}
                  </span>
                )}
              </div>
              <div className="grid min-w-0 max-w-xl grid-cols-2 gap-4">
                {group.books.map((book) => (
                  <BookCard key={book.id} book={book} compact />
                ))}
              </div>
            </section>
          ))}
        </div>
      )}
    </div>
  );
}
