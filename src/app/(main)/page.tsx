import { BOOKS, type Book } from "@/lib/books";
import { getLanguage } from "@/lib/languages";
import { getAllGrammar } from "@/lib/page-grammar";
import { LibraryView } from "@/components/library/library-view";

const KOREAN = getLanguage("")!;

/**
 * Nhãn ngắn bằng tiếng Việt cho huy hiệu sách trong kết quả tra cứu, vd
 * "Sơ cấp 1".
 *
 * Suy từ `level` chứ không chép tay danh sách: thêm sách mới là tự có nhãn.
 * `titleVi` thì quá dài cho một huy hiệu, còn `levelLabelKo` (초급1) tuy
 * đúng với bìa sách nhưng đây là giao diện tiếng Việt — bắt người ta đọc
 * chữ Hàn để biết mình đang xem sách nào thì ngược đời.
 *
 * Bốn quyển tiếng Hàn chia hai bậc, mỗi bậc hai quyển. Sách tiếng Anh dùng
 * thang cấp độ khác hẳn nên trả về `titleVi` cho an toàn — chúng cũng chưa
 * có nội dung ngữ pháp.
 */
function shortLabel(book: Book): string {
  if (book.lang !== "ko" || book.level > 4) return book.titleVi;
  const tier = book.level <= 2 ? "Sơ cấp" : "Trung cấp";
  const n = book.level <= 2 ? book.level : book.level - 2;
  return `${tier} ${n}`;
}

export default function LibraryPage() {
  const books = BOOKS.filter((b) => b.lang === KOREAN.code);

  return (
    <LibraryView
      language={KOREAN}
      books={books}
      grammar={getAllGrammar()}
      // Dựng bảng tên ở phía máy chủ: client chỉ cần đúng vài cái tên, gửi
      // nguyên mảng BOOKS sang là lãng phí.
      bookTitles={Object.fromEntries(
        BOOKS.map((book) => [book.id, shortLabel(book)])
      )}
    />
  );
}
