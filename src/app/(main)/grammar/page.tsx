import type { Metadata } from "next";
import { BOOKS, type Book } from "@/lib/books";
import { getAllGrammar } from "@/lib/page-grammar";
import {
  GrammarIndex,
  GrammarIndexHeading,
} from "@/components/grammar/grammar-index";

export const metadata: Metadata = {
  title: "Tra cứu ngữ pháp",
  description:
    "Tra cứu toàn bộ điểm ngữ pháp trong giáo trình KIIP Sơ cấp 1 và Sơ cấp 2, kèm nghĩa tiếng Việt và câu ví dụ.",
};

/**
 * Nhãn ngắn bằng tiếng Việt cho huy hiệu sách, vd "Sơ cấp 1".
 *
 * Suy từ `level` chứ không chép tay danh sách: thêm sách mới là tự có nhãn.
 * `titleVi` thì quá dài cho một huy hiệu, còn `levelLabelKo` (초급1) tuy
 * đúng với bìa sách nhưng đây là trang tra cứu bằng tiếng Việt — bắt người
 * ta đọc chữ Hàn để biết mình đang xem sách nào thì ngược đời.
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

export default function GrammarPage() {
  const entries = getAllGrammar();

  // Dựng bảng tên sách ở phía máy chủ: client chỉ cần đúng vài cái tên, gửi
  // nguyên mảng BOOKS sang là lãng phí.
  const bookTitles = Object.fromEntries(
    BOOKS.map((book) => [book.id, shortLabel(book)])
  );

  return (
    <div className="flex max-w-2xl flex-col gap-6">
      <GrammarIndexHeading total={entries.length} />
      <GrammarIndex entries={entries} bookTitles={bookTitles} />
    </div>
  );
}
