import type { Book } from "@/lib/books";

export interface LevelGroup {
  level: number;
  levelLabelKo?: string;
  books: Book[];
}

/**
 * Gom sách theo cấp độ để render trang thư viện — dùng chung cho trang gốc
 * (tiếng Hàn) và mọi trang `/[lang]` khác, mỗi trang chỉ cần lọc BOOKS theo
 * `lang` trước khi gọi hàm này.
 */
export function groupBooksByLevel(books: readonly Book[]): LevelGroup[] {
  const levels = Array.from(new Set(books.map((b) => b.level))).sort(
    (a, b) => a - b
  );
  return levels.map((level) => {
    const levelBooks = books.filter((b) => b.level === level);
    const textbook = levelBooks.find((b) => b.kind === "textbook");
    return {
      level,
      levelLabelKo: textbook?.levelLabelKo ?? levelBooks[0].levelLabelKo,
      books: [...levelBooks].sort((a, b) =>
        a.kind === b.kind ? 0 : a.kind === "textbook" ? -1 : 1
      ),
    };
  });
}
