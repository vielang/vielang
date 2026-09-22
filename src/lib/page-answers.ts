/**
 * Đáp án của sách (phần 모범 답안 in ở cuối sách), gắn vào đúng mục bài tập
 * trên trang ảnh.
 *
 * Cách hoạt động giống `page-translation` và `page-grammar`: mỗi trang có sẵn
 * danh sách chấm soạn tay, bấm vào chấm là hiện một bong bóng nhỏ — ở đây là
 * đáp án của mục đó. Người học khỏi phải lật xuống cuối sách rồi lật lại.
 *
 * MỘT chấm cho MỘT MỤC (듣기, 읽기...), không phải cho từng câu: bảng đáp án
 * của sách cũng chia theo mục ("듣기 p.18", "읽기 p.19"), và mỗi câu một chấm
 * thì trang chi chít nút.
 *
 * Sách bài tập (익힘책) thì khác: mỗi trang có vài bài (1., 2.), bài nào
 * cũng có đáp án. Gộp cả trang vào một chấm thì bong bóng dài 15–20 dòng, nên
 * ở đó mỗi BÀI một chấm, đặt cạnh chính bài đó.
 *
 * CHỈ chép nguyên văn đáp án của sách, không tự soạn thêm. Chỗ sách đánh dấu
 * ✓ vào ô trống thì ghi "네 ✓". Bài tự làm có chấm điểm đã có ở `lib/quiz.ts`
 * — hai thứ khác nhau: một bên là "xem đáp án sách", một bên là "làm bài".
 *
 * Import TĨNH như `lib/page-translation.ts`, cùng lý do (Output File Tracing
 * của Next không lần được file đọc qua path dựng động lúc chạy).
 */
import step1 from "../../content/answers/step1.json";
import step2 from "../../content/answers/step2.json";
import step3 from "../../content/answers/step3.json";
import step4 from "../../content/answers/step4.json";
import wbStep1 from "../../content/answers/wb-step1.json";
import wbStep2 from "../../content/answers/wb-step2.json";
import wbStep3 from "../../content/answers/wb-step3.json";
import wbStep4 from "../../content/answers/wb-step4.json";
import type { Rect } from "@/lib/page-translation";

export interface AnswerLine {
  /** Số câu đúng như in trong bảng đáp án: "1)", "2)", "①"... */
  label: string;
  /** Đáp án chép nguyên văn từ bảng đáp án cuối sách. */
  text: string;
}

export interface AnswerKey {
  id: string;
  /**
   * Khung định vị cái chấm, theo tỉ lệ 0–1 của ảnh trang. Chỉ TÂM có tác
   * dụng — chấm 44px được canh vào giữa, bề rộng/cao không tạo vùng bấm.
   *
   * Đặt ngay bên phải dòng câu hỏi ĐẦU TIÊN của mục: đó là chỗ mắt đang
   * nhìn khi làm xong và muốn dò đáp án. Tránh đặt cạnh tiêu đề mục ở đầu
   * trang — sát mép trên thì bị thanh công cụ che (xem `GRAMMAR_DOT_RECT`).
   */
  rect: Rect;
  /** Tên mục như in trong bảng đáp án: "듣기", "읽기". */
  section: string;
  /** Số trang chứa bảng đáp án này trong sách — để người dùng tự đối chiếu. */
  source: number;
  answers: AnswerLine[];
}

/**
 * Ép kiểu qua `unknown`: TypeScript suy `rect` trong JSON thành `number[]`,
 * không khớp tuple `Rect`. Hình dạng thật đã được `scripts/build-content.ts`
 * kiểm lúc build.
 */
const ANSWERS = {
  step1,
  step2,
  step3,
  step4,
  "wb-step1": wbStep1,
  "wb-step2": wbStep2,
  "wb-step3": wbStep3,
  "wb-step4": wbStep4,
} as unknown as Record<string, Record<string, AnswerKey[]>>;

export function getPageAnswers(bookId: string, page: number): AnswerKey[] {
  return ANSWERS[bookId]?.[String(page)] ?? [];
}

/** Danh sách số trang có đáp án sách của 1 sách, sắp xếp tăng dần. */
export function getAnswerPages(bookId: string): number[] {
  const book = ANSWERS[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}
