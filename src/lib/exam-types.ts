/**
 * Kiểu dữ liệu đề thi — tách riêng khỏi `lib/exams.ts` (nơi import từng file
 * đề) để module nhẹ như `lib/exam-levels.ts` dùng được mà khỏi nạp dữ liệu đề.
 *
 * Nay là đề TOEIC Listening & Reading. Mỗi đề là một file
 * `content/exams/<id>.json` đúng kiểu `Exam` dưới đây.
 */

export type SectionId = "listening" | "reading";

/** Chữ đã làm sạch (HTML an toàn — chỉ <br> <b> <u> <div class="exam-box"> <img>). */
export interface RichText {
  html: string;
}
/** Tranh / biểu đồ — đường dẫn tương đối trong thư mục tài nguyên của đề. */
export interface ImageRef {
  image: string;
  alt?: string;
}
export type Content = RichText | ImageRef;

export const isImage = (c: Content): c is ImageRef => "image" in c;

export interface ExamGroup {
  /** Câu đầu và câu cuối của khối. */
  from: number;
  to: number;
  /** Lời chỉ dẫn, chữ thuần như đề in: "Part 6 — Questions 131-134 refer to the following e-mail." */
  instruction: string;
  /**
   * Văn bản dùng chung cho cả khối (HTML đã làm sạch): bài đọc Part 7, hay
   * văn bản có chỗ trống của Part 6 — chỗ trống nằm ở ĐÂY, đề câu để rỗng.
   */
  passage?: string;
  /** Đoạn âm thanh đọc lời chỉ dẫn của khối (chỉ phần nghe), giây. */
  audio?: [number, number];
  /**
   * Hội thoại / bài nói DÙNG CHUNG cho cả khối (Part 3–4: nghe một đoạn, trả
   * lời ba câu). Đoạn âm thanh của các câu sau chỉ còn tiếng đọc câu hỏi và
   * khoảng dừng — nghe lại riêng câu đó thì phải phát hội thoại này trước.
   */
  dialogue?: [number, number];
}

export interface ExamQuestion {
  /** Số câu như in trong đề (TOEIC: 1–100 nghe, 101–200 đọc). */
  no: number;
  /** Điểm thô của câu — TOEIC mỗi câu 1; điểm quy đổi tính theo số câu đúng (xem `scaledScore`). */
  points: number;
  /** Đáp án đúng 1–4, tức (A)–(D). */
  answer: 1 | 2 | 3 | 4;
  /** Số lựa chọn mỗi hàng: 1 (câu dài), 2 hoặc 4 (từ ngắn). */
  layout: number;
  /**
   * Đề câu hỏi — có thể rỗng (Part 6: chỗ trống nằm trong văn bản của khối;
   * câu nghe chỉ có lựa chọn) hoặc là tranh.
   */
  prompt: Content;
  /** Bốn lựa chọn, KHÔNG kèm nhãn — giao diện tự thêm (A)–(D). */
  options: [Content, Content, Content, Content];
  /** Giải thích đáp án bằng tiếng Việt — hiện sau khi người học kiểm tra câu. */
  explanation?: string;
  /**
   * Đoạn âm thanh của câu trong buổi thi (chỉ phần nghe, nếu đã đo), giây:
   * từ lúc đọc số câu tới HẾT khoảng dừng trả lời — dùng để tô câu đang phát
   * khi thi thử.
   */
  audio?: [number, number];
  /**
   * Đoạn NGHE LẠI riêng câu này khi luyện tập: chỉ phần lời đọc, cắt ở đầu
   * khoảng dừng trả lời — không bắt người học chờ khoảng lặng, và không lọt
   * sang tiếng đọc số câu sau.
   */
  replay?: [number, number];
}

export interface ExamSection {
  id: SectionId;
  /** Tên hiện ở trang đề: "Reading · Part 5–7 (câu 101–200)". */
  title: string;
  /** Thời gian làm bài theo đề thật, phút (TOEIC: nghe 45, đọc 75). */
  minutes: number;
  /** File nghe của cả phần (chỉ phần nghe), tương đối trong thư mục tài nguyên. */
  audio?: string;
  /** Đoạn hướng dẫn chung ở đầu file nghe, giây (nếu đã đo). */
  intro?: [number, number];
  groups: ExamGroup[];
  questions: ExamQuestion[];
}

/**
 * Khoá của một câu trong bài làm: "<phần>:<số câu>". TOEIC đánh số liền
 * 1–200 nên số câu trơn đã đủ phân biệt, nhưng khoá theo phần giữ tương
 * thích với bài làm đã lưu (xem `migrateExamState`).
 */
export type QuestionKey = `${SectionId}:${number}`;

export function qKey(section: SectionId, no: number): QuestionKey {
  return `${section}:${no}`;
}

export type ExamLevel = "TOEIC";

export interface Exam {
  id: string;
  /** Số thứ tự đề luyện: 1, 2, … */
  round: number;
  year: number;
  level: ExamLevel;
  /** Tên hiện ở trang đề: "Đề luyện TOEIC Reading số 1". Không có thì dựng từ kỳ thi + số đề. */
  title?: string;
  /** Thư mục tài nguyên (ảnh, file nghe) của đề. */
  assetDir: string;
  /** Ghi nguồn đề (hiện ở trang đề). */
  source: string;
  sections: ExamSection[];
}

/**
 * Gốc URL tài nguyên của một đề. Ảnh đi qua CÙNG ORIGIN (`/img/exams/…`,
 * xem `imageRewrites` trong next.config) — cùng lý do với ảnh sách: service
 * worker cache được, không phải bật CORS cho R2. Lúc dev, Next phục vụ file
 * có sẵn trong `public/img/exams/` trước khi xét rewrite.
 */
export function examAssetBase(assetDir: string): string {
  return `/img/exams/${assetDir}`;
}
