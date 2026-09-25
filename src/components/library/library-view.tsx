import { Languages } from "lucide-react";
import { BookCard } from "@/components/library/book-card";
import { ContinueReading } from "@/components/library/continue-reading";
import type { Book } from "@/lib/books";
import { groupBooksByLevel } from "@/lib/library";
import type { LanguageConfig } from "@/lib/languages";
import type { GrammarEntry } from "@/lib/page-grammar";
import { GrammarIndex } from "@/components/grammar/grammar-index";
import { SegmentedNav } from "@/components/layout/segmented-nav";
import { seriesHref, seriesOfLang, type BookSeries } from "@/lib/series";
import { PageHeader, SectionLabel } from "@/components/layout/page-header";
import { EmptyState } from "@/components/layout/empty-state";
import { VideoPromoCard } from "@/components/video/video-promo-card";

/**
 * Khung trang thư viện dùng chung cho trang gốc (tiếng Hàn) và mọi trang
 * `/[lang]` khác. Không có sách nào khớp `language` -> hiện empty-state
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
  grammar,
  bookTitles,
  showVideoPromo = false,
}: {
  language: LanguageConfig;
  series?: BookSeries;
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
  /** Hiện lối vào "Học tiếng Hàn qua video" — chỉ trang gốc tiếng Hàn (`/`). */
  showVideoPromo?: boolean;
}) {
  const groups = groupBooksByLevel(books);
  const hasGrammar = grammar !== undefined && grammar.length > 0;
  const allSeries = seriesOfLang(language.code);
  const languageHref = language.slug ? `/${language.slug}` : "/";

  return (
    <div className="flex flex-col gap-6">
      {/* Cùng một khung đầu trang với Luyện thi, Cẩm nang, Góc học tập: tiêu
          đề, rồi hàng chọn tầng dưới (nếu có). */}
      <div className="flex flex-col gap-3">
        <PageHeader title={language.heading} subtitle={series?.blurb ?? language.blurb} />
        {allSeries.length > 1 && (
          <SegmentedNav
            label="Bộ sách"
            items={[
              { key: "all", label: "Tất cả", href: languageHref, active: !series },
              ...allSeries.map((s) => ({
                key: s.id,
                label: s.label,
                href: seriesHref(s, languageHref),
                active: s.id === series?.id,
              })),
            ]}
          />
        )}
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

      {showVideoPromo && <VideoPromoCard />}

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
