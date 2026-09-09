/**
 * Bài tập CÓ SẴN TRONG SÁCH của từng trang — chép lại đúng đề in trên trang
 * ảnh (`content/quiz/<bookId>/<page>.json`), gộp thành 1 file/sách qua
 * `npm run build-content` (tự chạy qua predev/prebuild).
 *
 * Đây KHÔNG phải câu hỏi tự nghĩ thêm: người dùng đang nhìn trang sách, phần
 * này chỉ là chỗ để họ nộp đáp án cho chính bài tập in trên trang đó. Vì vậy
 * dữ liệu bám theo cấu trúc của sách — `QuizSection.title` là nhãn mục thật
 * ("연습 1", "읽기", "쓰기"...), `label` là số thứ tự thật ("①", "1)"...).
 *
 * Vì sao tách hẳn khỏi note: note là HTML người dùng SỬA ĐƯỢC và lưu
 * localStorage (xem `note-store`). Nhúng bài tập vào đó thì chỉ cần người
 * dùng sửa bài giảng một lần là cấu trúc bài tập bị editor làm hỏng. Ở đây:
 * đề bài là dữ liệu CHỈ ĐỌC, bài làm nằm ở `quiz-store`, bài giảng vẫn sửa
 * thoải mái — ba thứ vòng đời khác nhau.
 *
 * Import TĨNH như `lib/notes.ts`, cùng lý do (Output File Tracing của Next
 * không lần được file đọc qua path dựng động lúc chạy).
 */
import step1 from "../../content/quiz/step1.json";
import step2 from "../../content/quiz/step2.json";
import step3 from "../../content/quiz/step3.json";
import step4 from "../../content/quiz/step4.json";

interface BaseItem {
  id: string;
  /** Số thứ tự đúng như in trong sách: "①", "1)", "가"... */
  label?: string;
  /** Đề bài, chép nguyên văn từ trang sách. */
  prompt: string;
  /** Giải thích thêm (không có trong sách) — chỉ hiện sau khi đã chấm. */
  explain?: string;
}

/** Bài trắc nghiệm / đúng-sai in sẵn trong sách. */
export interface ChoiceItem extends BaseItem {
  kind: "choice";
  options: string[];
  /** Chỉ số (0-based) của phương án đúng theo đáp án sách. */
  answer: number;
}

/** Bài điền vào chỗ trống có đáp án xác định. */
export interface FillItem extends BaseItem {
  kind: "fill";
  /**
   * Các đáp án chấp nhận được — BẮT BUỘC là mảng: tiếng Hàn biến đổi theo
   * trợ từ ("베트남" / "베트남 사람"), khoá cứng 1 đáp án là bắt sai oan.
   */
  answers: string[];
}

/**
 * Bài mở: 말하기 / 쓰기 / 친구와 이야기해 보세요... — sách không có đáp án
 * đúng duy nhất. CHỈ LƯU, không chấm. `model` là câu mẫu để người học tự đối
 * chiếu, hiện khi họ chủ động bấm xem.
 */
export interface FreeItem extends BaseItem {
  kind: "free";
  model?: string;
  /** Ô nhập nhiều dòng (đoạn văn 쓰기) thay vì 1 dòng. */
  multiline?: boolean;
}

export type QuizItem = ChoiceItem | FillItem | FreeItem;

/** Một mục bài tập của trang, ví dụ "연습 1", "듣기", "읽기". */
export interface QuizSection {
  /** Nhãn mục đúng như in trong sách. */
  title: string;
  /** Câu chỉ dẫn của sách ("보기와 같이 이야기해 보세요."), chép nguyên. */
  instruction?: string;
  items: QuizItem[];
}

const QUIZZES: Record<string, Record<string, QuizSection[]>> = {
  step1,
  step2,
  step3,
  step4,
} as Record<string, Record<string, QuizSection[]>>;

export function getPageQuiz(bookId: string, page: number): QuizSection[] {
  return QUIZZES[bookId]?.[String(page)] ?? [];
}

export function countItems(sections: QuizSection[]): number {
  return sections.reduce((n, s) => n + s.items.length, 0);
}

/** Bài mở không chấm được — tách riêng để đếm tiến độ cho đúng. */
export function isGradable(item: QuizItem): item is ChoiceItem | FillItem {
  return item.kind !== "free";
}

/** Danh sách số trang có bài tập của 1 sách, sắp xếp tăng dần. */
export function getQuizPages(bookId: string): number[] {
  const book = QUIZZES[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}

/**
 * Chuẩn hoá đáp án tự luận trước khi so sánh: gộp khoảng trắng và bỏ dấu câu
 * cuối. Cố ý KHÔNG so khớp mờ (fuzzy) — sai chính tả tiếng Hàn là sai thật,
 * báo đúng mới giúp người học sửa được.
 */
export function normalizeAnswer(text: string): string {
  return text
    .replace(/\s+/g, " ")
    .trim()
    .replace(/[.?!,。？！]+$/u, "")
    .trim();
}

export function isCorrect(item: QuizItem, value: string | number): boolean {
  if (item.kind === "choice") return value === item.answer;
  if (item.kind === "free") return false; // không chấm
  if (typeof value !== "string") return false;
  const given = normalizeAnswer(value);
  return item.answers.some((a) => normalizeAnswer(a) === given);
}
