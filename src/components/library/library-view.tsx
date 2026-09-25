import { Languages } from "lucide-react";
import { BookCard } from "@/components/library/book-card";
import { ContinueReading } from "@/components/library/continue-reading";
import type { Book } from "@/lib/books";
import { groupBooksByLevel } from "@/lib/library";
import type { LanguageConfig } from "@/lib/languages";
import type { GrammarEntry } from "@/lib/page-grammar";
import { GrammarIndex } from "@/components/grammar/grammar-index";
import { TrackNav } from "@/components/layout/track-nav";
import { PageHeader, SectionLabel } from "@/components/layout/page-header";

/**
 * Khung trang thư viện dùng chung cho trang gốc (tiếng Hàn) và mọi trang
 * `/[lang]` khác. Không có sách nào khớp `language` -> hiện empty-state
 * "Sắp có nội dung" thay vì lưới trống.
 */
/** Dòng phụ của trang Thư viện — chung cho mọi mảng (tiếng Hàn, tiếng Anh, IT). */
export const LIBRARY_SUBTITLE = "Giáo trình tiếng Hàn KIIP, tiếng Anh và lộ trình IT.";

export function LibraryView({
  language,
  books,
  grammar,
  bookTitles,
}: {
  language: LanguageConfig;
  books: readonly Book[];
  /**
   * Điểm ngữ pháp để tra cứu. Bỏ trống thì giữ tiêu đề như cũ.
   *
   * Chỉ giáo trình tiếng Hàn mới có nội dung ngữ pháp, mà khung này dùng
   * chung cho cả `/[lang]` — đặt ô tra cứu rỗng lên trang tiếng Anh thì gõ
   * gì cũng không ra, tệ hơn là không có.
   */
  grammar?: GrammarEntry[];
  bookTitles?: Record<string, string>;
}) {
  const groups = groupBooksByLevel(books);
  const hasGrammar = grammar !== undefined && grammar.length > 0;

  return (
    <div className="flex flex-col gap-6">
      {/* Cùng một khung đầu trang với Luyện thi, Cẩm nang, Góc học tập: tiêu
          đề, rồi hàng tab con ngay dưới. Chọn mảng kiến thức nằm ở đây chứ
          không ở header vì nó chỉ áp dụng cho thư viện. */}
      <div className="flex flex-col gap-3">
        <PageHeader title="Thư viện" subtitle={LIBRARY_SUBTITLE} />
        <TrackNav />
      </div>

      {/* Tiếng Hàn: ô tra ngữ pháp (dòng gợi ý bên trong nói có bao nhiêu
          điểm ngữ pháp). Ngôn ngữ khác chưa có ngữ pháp: một dòng giới thiệu. */}
      {hasGrammar ? (
        <GrammarIndex entries={grammar} bookTitles={bookTitles ?? {}} />
      ) : (
        <p className="-mt-2 text-sm text-muted-foreground">{language.description}</p>
      )}

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
              <SectionLabel
                aside={group.levelLabelKo && <span className="font-korean">{group.levelLabelKo}</span>}
              >
                Cấp {group.level}
              </SectionLabel>
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
