import { BOOKS } from "@/lib/books";
import { BookCard } from "@/components/library/book-card";

export default function LibraryPage() {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Thư viện</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Sách văn hóa – xã hội Hàn Quốc (chương trình KIIP), dành cho người
          Việt học tiếng Hàn.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4">
        {BOOKS.map((book) => (
          <BookCard key={book.id} book={book} />
        ))}
      </div>
    </div>
  );
}
