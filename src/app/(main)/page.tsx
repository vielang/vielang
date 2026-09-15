import { BOOKS } from "@/lib/books";
import { BookCard } from "@/components/library/book-card";

function groupByLevel() {
  const levels = Array.from(new Set(BOOKS.map((b) => b.level))).sort(
    (a, b) => a - b
  );
  return levels.map((level) => {
    const books = BOOKS.filter((b) => b.level === level);
    const textbook = books.find((b) => b.kind === "textbook");
    return {
      level,
      levelLabelKo: textbook?.levelLabelKo ?? books[0].levelLabelKo,
      books: [...books].sort((a, b) =>
        a.kind === b.kind ? 0 : a.kind === "textbook" ? -1 : 1
      ),
    };
  });
}

export default function LibraryPage() {
  const groups = groupByLevel();

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Thư viện</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Sách văn hóa – xã hội Hàn Quốc (chương trình KIIP), dành cho người
          Việt học tiếng Hàn. Mỗi cấp độ gồm giáo trình chính và sách bài tập
          đi kèm.
        </p>
      </div>

      <div className="flex min-w-0 flex-col gap-6">
        {groups.map((group) => (
          <section key={group.level} className="flex min-w-0 flex-col gap-3">
            <div className="flex items-baseline gap-2 border-b border-border pb-2">
              <h2 className="text-base font-semibold tracking-tight">
                Cấp {group.level}
              </h2>
              <span className="font-korean text-sm text-muted-foreground">
                {group.levelLabelKo}
              </span>
            </div>
            <div className="grid min-w-0 max-w-xl grid-cols-2 gap-4">
              {group.books.map((book) => (
                <BookCard key={book.id} book={book} compact />
              ))}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
