import { getBook, type Book } from "@/lib/books";
import { getChapters, type Chapter } from "@/lib/chapters";
import { getPageGrammar, type GrammarPoint } from "@/lib/page-grammar";
import { getPageTranslations, type TranslationRegion } from "@/lib/page-translation";

/**
 * Trang "Bài N" của một cuốn sách: gom bản dịch và ngữ pháp của mọi trang
 * trong bài thành một bài viết đọc được liền mạch.
 *
 * Vì sao cần: trang đọc sách chỉ là ảnh scan, bản dịch nằm trong bong bóng
 * bật lên phía client — Google vào chỉ thấy một bức ảnh. Người Việt lại hay
 * tìm đúng kiểu "KIIP 1 bài 3 dịch", nên mỗi bài cần một trang CHỮ thật, ai
 * cũng thấy (không giấu chữ chỉ cho máy tìm kiếm — Google phạt kiểu đó).
 *
 * Chỉ bài nào có nội dung mới có trang: trang rỗng chỉ tổ bị đánh giá thấp.
 */
export interface LessonPage {
  page: number;
  translations: TranslationRegion[];
  grammar: GrammarPoint[];
}

export interface BookLesson {
  book: Book;
  chapter: Chapter;
  pages: LessonPage[];
}

export function bookLessonHref(bookId: string, lesson: number): string {
  return `/books/${bookId}/bai/${lesson}`;
}

function lessonPages(bookId: string, chapter: Chapter): LessonPage[] {
  const pages: LessonPage[] = [];
  for (let page = chapter.startPage; page <= chapter.endPage; page++) {
    const translations = getPageTranslations(bookId, page);
    const grammar = getPageGrammar(bookId, page);
    if (translations.length > 0 || grammar.length > 0) pages.push({ page, translations, grammar });
  }
  return pages;
}

/** Các bài có nội dung của một cuốn, theo thứ tự trong sách. */
export function bookLessons(bookId: string): BookLesson[] {
  const book = getBook(bookId);
  if (!book) return [];
  return getChapters(book.id, book.totalPages)
    .map((chapter) => ({ book, chapter, pages: lessonPages(book.id, chapter) }))
    .filter((lesson) => lesson.pages.length > 0);
}

export function getBookLesson(bookId: string, lesson: number): BookLesson | undefined {
  return bookLessons(bookId).find((l) => l.chapter.lesson === lesson);
}

/** Số bài chứa trang `page`, hoặc undefined nếu trang nằm ngoài các bài. */
export function lessonOfPage(bookId: string, page: number): number | undefined {
  const book = getBook(bookId);
  if (!book) return undefined;
  return getChapters(book.id, book.totalPages).find((c) => page >= c.startPage && page <= c.endPage)?.lesson;
}

/**
 * Tên ngắn người Việt hay gõ khi tìm: "KIIP 1", "KIIP 3 (Trung cấp 1)"…
 * Sách bài tập thêm chữ "sách bài tập". Sách ngoài KIIP dùng tên gốc.
 */
export function bookSearchName(book: Book): string {
  if (book.series !== "kiip") return book.titleVi;
  return book.id.startsWith("wb-") ? `KIIP ${book.level} sách bài tập` : `KIIP ${book.level}`;
}
