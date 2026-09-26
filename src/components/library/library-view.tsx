import { Languages } from "lucide-react";
import { BookCard } from "@/components/library/book-card";
import { ContinueReading } from "@/components/library/continue-reading";
import type { Book } from "@/lib/books";
import { groupBooksByLevel } from "@/lib/library";
import type { LanguageConfig } from "@/lib/languages";
import { SegmentedNav } from "@/components/layout/segmented-nav";
import { libraryTabs, seriesOfLang, type BookSeries } from "@/lib/series";
import { PageHeader, SectionLabel } from "@/components/layout/page-header";
import { EmptyState } from "@/components/layout/empty-state";

/**
 * Khung trang thư viện sách dùng chung cho mọi trang `/[lang]` và
 * `/sach/<bộ>`. Không có sách nào khớp `language` -> hiện empty-state
 * "Sắp có nội dung" thay vì lưới trống.
 *
 * Ngôn ngữ đã chọn trên header (điện thoại) / menu (máy tính), nên đầu trang
 * không lặp lại hàng chọn ngôn ngữ mà là tầng dưới: hàng chọn BỘ SÁCH — chỉ
 * hiện khi ngôn ngữ có từ hai bộ. `series` có giá trị = trang một bộ
 * (`/sach/<id>`), bỏ trống = trang ngôn ngữ, gồm mọi bộ.
 */
export function LibraryView({
  language,
  series,
  books,
}: {
  language: LanguageConfig;
  series?: BookSeries;
  books: readonly Book[];
}) {
  const groups = groupBooksByLevel(books);
  const allSeries = seriesOfLang(language.code);
  // Chưa chọn bộ (trang ngôn ngữ) mà chỉ có ĐÚNG 1 bộ: trang đó CHÍNH LÀ bộ
  // đó rồi (xem lib/series.ts `seriesHref`) — tô tab của bộ ấy luôn, không
  // phải tô "Tất cả" (chỉ có ý nghĩa khi có từ 2 bộ trở lên).
  const activeKey = series?.id ?? (allSeries.length > 1 ? "all" : (allSeries[0]?.id ?? "all"));
  const tabs = libraryTabs(language, activeKey);

  return (
    <div className="flex flex-col gap-6">
      {/* Cùng một khung đầu trang với Luyện thi, Cẩm nang, Góc học tập: tiêu
          đề, rồi hàng chọn bộ sách (nếu có từ hai bộ). */}
      <div className="flex flex-col gap-3">
        <PageHeader title={language.heading} subtitle={series?.blurb ?? language.blurb} />
        {tabs.length > 1 && <SegmentedNav label="Bộ sách" items={tabs} />}
      </div>

      <p className="-mt-2 text-sm text-muted-foreground">{language.description}</p>

      {/* Đặt TRƯỚC lưới sách: mở thư viện ra phần lớn là để đọc tiếp cuốn
          đang dở, chứ không phải để chọn cuốn mới. Tự ẩn khi chưa đọc gì. */}
      <ContinueReading books={books} />

      {groups.length === 0 ? (
        <EmptyState
          dashed
          icon={Languages}
          title="Sắp có nội dung"
          description={`Chưa có giáo trình ${language.label.toLowerCase()} nào được thêm vào. Phần này sẽ hiển thị sách theo từng cấp độ, cùng định dạng với các ngôn ngữ khác.`}
        />
      ) : (
        <div className="flex min-w-0 flex-col gap-6">
          {groups.map((group) => (
            <section key={group.level} className="flex min-w-0 flex-col gap-3">
              <SectionLabel
                aside={group.levelLabelKo}
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
