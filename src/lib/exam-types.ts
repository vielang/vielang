/**
 * Kiểu dữ liệu đề thi TOPIK — tách riêng khỏi `lib/exams.ts` (nơi import từng
 * file đề) để script nhập đề (`scripts/import-topik.ts`) dùng được trước khi
 * các file đề tồn tại.
 */

export type SectionId = "listening" | "reading" | "writing";

/** Chữ đã làm sạch (HTML an toàn — chỉ <br> <b> <u> <div class="exam-box"> <img>). */
export interface RichText {
  html: string;
}
/** Tranh / biểu đồ — đường dẫn tương đối trong thư mục tài nguyên của kỳ thi. */
export interface ImageRef {
  image: string;
  alt?: string;
}
export type Content = RichText | ImageRef;

export const isImage = (c: Content): c is ImageRef => "image" in c;

export interface ExamGroup {
  /** Câu đầu và câu cuối của khối "※ [a~b] …". */
  from: number;
  to: number;
  /** Lời chỉ dẫn, chữ thuần: "※ [1～4] 다음을 듣고 <보기>와 같이 …". */
  instruction: string;
  /** Câu mẫu <보기> (đã có sẵn đáp án). */
  example?: { html: string; options: string[]; answer: number; layout: number };
  /** Đoạn văn / thông báo dùng chung cho cả khối (HTML đã làm sạch). */
  passage?: string;
  /** Đoạn âm thanh đọc chỉ dẫn + ví dụ (chỉ phần nghe), giây. */
  audio?: [number, number];
  /**
   * Hội thoại DÙNG CHUNG cho cả khối (vd [25~26] nghe một đoạn, trả lời hai
   * câu). Đoạn âm thanh của câu thứ hai chỉ còn tiếng đọc số câu và khoảng
   * dừng — nghe lại riêng câu đó thì phải phát hội thoại này trước.
   */
  dialogue?: [number, number];
}

export interface ExamQuestion {
  /** Số câu như in trong đề. */
  no: number;
  points: number;
  /** Đáp án đúng 1–4 (①–④). */
  answer: 1 | 2 | 3 | 4;
  /** Số lựa chọn mỗi hàng như trang gốc: 1, 2 hoặc 4. */
  layout: number;
  /** Đề câu hỏi — có thể rỗng (câu nghe chỉ có lựa chọn) hoặc là tranh. */
  prompt: Content;
  options: [Content, Content, Content, Content];
  /** Đoạn âm thanh của riêng câu này (chỉ phần nghe, nếu đã đo), giây. */
  audio?: [number, number];
}

/**
 * Một câu VIẾT (TOPIK II, câu 51–54). Không chấm tự động được: người học
 * xem đáp án mẫu chính thức rồi tự chấm theo thang điểm của câu.
 */
export interface WritingTask {
  no: number;
  points: number;
  /** 51–52: điền hai chỗ trống ㉠ ㉡ (mỗi câu một dòng). 53–54: viết đoạn/bài. */
  kind: "blanks" | "essay";
  /** Bài viết: độ dài yêu cầu, ký tự (tính cả dấu cách như ô 원고지). */
  chars?: [number, number];
  /** Ảnh trang đề có câu này (đề viết chỉ có bản in, không có chữ). */
  page: string;
}

export interface ExamSection {
  id: SectionId;
  /** Tên như trang gốc: "TOPIKⅠ 듣기 (1번 ～ 30번)". */
  title: string;
  /** Thời gian làm bài theo đề thật, phút. */
  minutes: number;
  /** File nghe của cả phần (chỉ phần nghe), tương đối trong thư mục tài nguyên. */
  audio?: string;
  /** Đoạn hướng dẫn chung ở đầu file nghe, giây (nếu đã đo). */
  intro?: [number, number];
  groups: ExamGroup[];
  /** Câu trắc nghiệm — phần viết thì rỗng, câu nằm ở `writing.tasks`. */
  questions: ExamQuestion[];
  /** Chỉ phần viết. */
  writing?: {
    tasks: WritingTask[];
    /** Ảnh trang đáp án mẫu + tiêu chí chấm chính thức. */
    modelAnswers: string[];
  };
}

/**
 * Khoá của một câu trong bài làm: "<phần>:<số câu>". Không dùng số câu trơn
 * vì TOPIK II đánh số LẠI từ 1 ở phần đọc — câu 1 nghe và câu 1 đọc trùng số.
 */
export type QuestionKey = `${SectionId}:${number}`;

export function qKey(section: SectionId, no: number): QuestionKey {
  return `${section}:${no}`;
}

/** Khoá ô chữ của câu viết: câu 51–52 có hai ô (㉠ = 0, ㉡ = 1), bài viết một ô. */
export function textKey(no: number, blank = 0): string {
  return `${no}:${blank}`;
}

export type ExamLevel = "TOPIK I" | "TOPIK II";

export interface Exam {
  id: string;
  /** Số kỳ thi: 102. */
  round: number;
  year: number;
  level: ExamLevel;
  /** Thư mục tài nguyên (ảnh, file nghe) — dùng chung cho TOPIK I và II cùng kỳ. */
  assetDir: string;
  /** Ghi nguồn đề (hiện ở trang đề). */
  source: string;
  sections: ExamSection[];
}

/**
 * Gốc URL tài nguyên của một kỳ thi. Ảnh đi qua CÙNG ORIGIN (`/img/exams/…`,
 * xem `imageRewrites` trong next.config) — cùng lý do với ảnh sách: service
 * worker cache được, không phải bật CORS cho R2. Lúc dev, Next phục vụ file
 * có sẵn trong `public/img/exams/` trước khi xét rewrite.
 */
export function examAssetBase(assetDir: string): string {
  return `/img/exams/${assetDir}`;
}
