/**
 * Giải thích điểm ngữ pháp, gắn vào đúng trang ngữ pháp trong sách.
 *
 * Vì sao cần: sách giải thích ngữ pháp CHỈ bằng tiếng Hàn. Dòng chú thích
 * kiểu `사람, 사물 이름을 말할 때 사용해요` là một vòng luẩn quẩn — người mới
 * học cần lời giải thích thì lại không đọc nổi chính lời giải thích đó. Bản
 * dịch bài đọc giải quyết phần HIỂU NỘI DUNG, chỗ này giải quyết phần HIỂU
 * QUY TẮC.
 *
 * Cách hoạt động giống `page-translation`: mỗi trang có sẵn danh sách vùng
 * soạn tay, bấm vào chấm là hiện nội dung. Khác ở hai chỗ:
 *
 *  - Nội dung là HTML dựng sẵn từ Markdown (bảng chia theo patchim, ví dụ,
 *    mẹo nhớ) chứ không phải một đoạn chữ trơn, nên phải hiện trong panel
 *    chứ không nhét vừa bong bóng nhỏ.
 *  - Mỗi mục có `slug` riêng cho điểm ngữ pháp. Trang nào trong sách nào
 *    không quan trọng — `-지요?` vẫn là `-지요?`. Nhờ vậy sau này làm được
 *    trang tra cứu ngữ pháp, hoặc dùng lại lời giải thích khi cùng điểm ngữ
 *    pháp xuất hiện ở sách khác, mà không phải chép nội dung sang chỗ mới.
 *
 * Bố cục sách rất đều: mỗi bài đúng 2 điểm ngữ pháp, tiêu đề luôn nằm góc
 * trên bên phải. Xem `lib/grammar-pages.ts`.
 *
 * Import TĨNH như `lib/notes.ts` và `lib/page-translation.ts`, cùng lý do
 * (Output File Tracing của Next không lần được file đọc qua path dựng động).
 */
import step1 from "../../content/grammar/step1.json";
import step2 from "../../content/grammar/step2.json";
import step3 from "../../content/grammar/step3.json";
import step4 from "../../content/grammar/step4.json";

/** [x, y, rộng, cao] — tất cả theo tỉ lệ 0–1 của ảnh trang. */
export type Rect = [number, number, number, number];

export interface GrammarPoint {
  id: string;
  /**
   * Mã của CHÍNH điểm ngữ pháp, không gắn với trang hay sách — vd
   * `ieyo-yeyo`, `eun-neun`, `jiyo`. Hai trang cùng dạy một điểm thì dùng
   * chung mã.
   */
  slug: string;
  /** Vùng tiêu đề trên ảnh, để neo cái chấm vào đúng chỗ. */
  rect: Rect;
  /** Nguyên văn tiêu đề trong sách, vd "명 이에요/예요". */
  title: string;
  /** Dòng chú thích tiếng Hàn in ngay dưới tiêu đề. */
  ko?: string;
  /** Dịch dòng chú thích đó — câu trả lời ngắn nhất cho "cái này để làm gì". */
  vi: string;
  /** Phần giải thích đầy đủ, HTML đã dựng sẵn từ Markdown lúc build. */
  html: string;
}

const GRAMMAR = {
  step1,
  step2,
  step3,
  step4,
} as unknown as Record<string, Record<string, GrammarPoint[]>>;

export function getPageGrammar(bookId: string, page: number): GrammarPoint[] {
  return GRAMMAR[bookId]?.[String(page)] ?? [];
}

/** Danh sách số trang có giải thích ngữ pháp của 1 sách, sắp xếp tăng dần. */
export function getGrammarPages(bookId: string): number[] {
  const book = GRAMMAR[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}
