import { BOOKS } from "@/lib/books";
import { getLanguage } from "@/lib/languages";
import { LibraryView } from "@/components/library/library-view";

const KOREAN = getLanguage("")!;

export default function LibraryPage() {
  const books = BOOKS.filter((b) => b.lang === KOREAN.code);
  return <LibraryView language={KOREAN} books={books} />;
}
