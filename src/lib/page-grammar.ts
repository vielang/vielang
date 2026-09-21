/**
 * Giải thích điểm ngữ pháp, gắn vào đúng trang ngữ pháp trong sách.
 *
 * Vì sao cần: sách giải thích ngữ pháp CHỈ bằng tiếng Hàn. Dòng chú thích
 * kiểu `사람, 사물 이름을 말할 때 사용해요` là một vòng luẩn quẩn — người mới
 * học cần lời giải thích thì lại không đọc nổi chính lời giải thích đó. Bản
 * dịch bài đọc giải quyết phần HIỂU NỘI DUNG, chỗ này giải quyết phần HIỂU
 * QUY TẮC.
 *
 * Cách hoạt động giống hệt `page-translation`: mỗi trang có sẵn danh sách
 * vùng soạn tay, bấm vào chấm là hiện một bong bóng nhỏ.
 *
 * CHỈ một định nghĩa ngắn, cố ý. Bản đầu có cả phần giải thích dài bằng
 * Markdown — bảng chia theo patchim, ví dụ, mục lưu ý — hiện trong một tấm
 * phủ. Đọc giữa lúc đang học thì quá dài và rối, mà còn che mất trang sách
 * đang xem. Muốn học sâu thì đã có tab bài giảng.
 *
 * Khác `page-translation` đúng một chỗ: mỗi mục có `slug` riêng cho điểm
 * ngữ pháp. Trang nào trong sách nào không quan trọng — `-지요?` vẫn là
 * `-지요?`. Nhờ vậy sau này làm được trang tra cứu ngữ pháp, hoặc dùng lại
 * định nghĩa khi cùng điểm ngữ pháp xuất hiện ở sách khác.
 *
 * Mỗi bài đúng 2 điểm ngữ pháp, mỗi trang một điểm. Sơ cấp đặt hai trang đó
 * ở offset +3 và +5, Trung cấp 1 đặt liền nhau ở +3 và +4. Xem
 * `lib/grammar-pages.ts`.
 *
 * Import TĨNH như `lib/notes.ts` và `lib/page-translation.ts`, cùng lý do
 * (Output File Tracing của Next không lần được file đọc qua path dựng động).
 */
import { BOOKS } from "@/lib/books";
import step1 from "../../content/grammar/step1.json";
import step2 from "../../content/grammar/step2.json";
import step3 from "../../content/grammar/step3.json";
import step4 from "../../content/grammar/step4.json";

/** [x, y, rộng, cao] — tất cả theo tỉ lệ 0–1 của ảnh trang. */
export type Rect = [number, number, number, number];

/**
 * Chỗ đặt chấm ngữ pháp — MỘT vị trí cho mọi trang, mọi sách.
 *
 * Tâm nằm ở (0.55, 0.45) của ảnh trang. Đo trên cả 104 trang ngữ pháp của
 * Sơ cấp 1, Sơ cấp 2 và Trung cấp 1: không trang nào có chữ dưới chấm, chỗ
 * dính nhiều nhất cũng chỉ 1.5% diện tích vòng chạm.
 *
 * Trước đây mỗi bộ sách neo vào một mốc riêng trong trang, và chấm nhảy
 * theo bố cục từng trang: Sơ cấp 1 xê dịch tới 212px, Sơ cấp 2 tới 161px.
 * Lật trang là phải đi tìm lại cái chấm, và có hai trang chấm còn cấn vào
 * chữ. Một điểm cố định thì ngón tay quen được chỗ.
 *
 * Cái đánh đổi, nói thẳng: khung giải thích của Sơ cấp cao thấp không đều
 * (đáy dao động 413px), nên điểm cố định này có trang rơi trong khung, có
 * trang ngay dưới khung, có trang nằm cạnh dòng tiêu đề bài tập. Vị trí
 * trên màn hình thì đứng yên, còn quan hệ với nội dung in thì vẫn đổi. Với
 * một nút bấm thì đứng yên đáng giá hơn.
 *
 * Sách mới KHÔNG được mặc nhiên dùng lại con số này — phải đo lại trên ảnh
 * trang thật rồi mới biết nó có rơi vào chữ hay không.
 */
export const GRAMMAR_DOT_RECT: Rect = [0.35, 0.43, 0.4, 0.04];

export interface GrammarPoint {
  id: string;
  /**
   * Mã của CHÍNH điểm ngữ pháp, không gắn với trang hay sách — vd
   * `ieyo-yeyo`, `eun-neun`, `jiyo`. Hai trang cùng dạy một điểm thì dùng
   * chung mã.
   */
  slug: string;
  /**
   * Khung định vị cái chấm. Chỉ TÂM của nó có tác dụng — chấm 44px được
   * canh vào giữa, bề rộng và bề cao không tạo ra vùng bấm.
   *
   * Mọi điểm ngữ pháp đều dùng chung một giá trị: `GRAMMAR_DOT_RECT`. Xem
   * chú thích ở đó để biết vì sao.
   */
  rect: Rect;
  /** Nguyên văn tiêu đề trong sách, vd "명 이에요/예요". */
  title: string;
  /** Dòng chú thích tiếng Hàn in ngay dưới tiêu đề, để đối chiếu với sách. */
  ko?: string;
  /**
   * Định nghĩa tiếng Việt.
   *
   * Giữ trong khoảng một đến hai câu. Đây là câu trả lời cho "cái đuôi này
   * để làm gì?", không phải chỗ dạy hết mọi quy tắc.
   */
  vi: string;
  /**
   * Một câu ví dụ, LẤY TỪ CHÍNH TRANG ĐÓ (hộp 예문 trong sách).
   *
   * Lấy từ trang đang mở chứ không tự đặt câu mới: người học vừa nhìn thấy
   * đúng câu đó ở ngay bên cạnh, nên đối chiếu được lập tức. Câu tự chế thì
   * lại là một thứ nữa phải giải mã.
   *
   * Định nghĩa thuần thì đúng nhưng khô — "trợ từ chủ ngữ, đánh dấu chủ thể"
   * chẳng giúp gì cho người mới. Một câu thật mới làm nó rơi xuống.
   */
  exKo: string;
  /** Bản dịch của đúng câu ví dụ đó. */
  exVi: string;
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

/** Một điểm ngữ pháp kèm chỗ nó xuất hiện trong sách. */
export interface GrammarEntry extends GrammarPoint {
  bookId: string;
  page: number;
}

/**
 * Toàn bộ điểm ngữ pháp của mọi sách, theo ĐÚNG THỨ TỰ HỌC.
 *
 * Xếp theo thứ tự sách rồi tới số trang, chứ không theo bảng chữ cái: người
 * tra cứu hầu hết đang theo giáo trình, nên "bài này ở đâu trong chương
 * trình" hữu ích hơn nhiều so với "chữ này đứng thứ mấy". Muốn tìm nhanh thì
 * đã có ô tìm kiếm.
 *
 * Thứ tự `BOOK_ORDER` lấy từ `BOOKS` để khỏi phải nhớ hai nơi.
 */
export function getAllGrammar(): GrammarEntry[] {
  const order = new Map(BOOKS.map((book, i) => [book.id, i]));

  return Object.entries(GRAMMAR)
    .flatMap(([bookId, pages]) =>
      Object.entries(pages).flatMap(([page, points]) =>
        points.map((point) => ({ ...point, bookId, page: Number(page) }))
      )
    )
    .sort(
      (a, b) =>
        (order.get(a.bookId) ?? 0) - (order.get(b.bookId) ?? 0) ||
        a.page - b.page
    );
}
