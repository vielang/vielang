/**
 * Bản dịch từng đoạn trên ảnh trang sách.
 *
 * Cách hoạt động: mỗi trang có sẵn danh sách vùng chữ nhật (soạn tay) kèm
 * bản dịch đã dịch sẵn. Người dùng bấm vào vùng nào thì hiện bản dịch của
 * vùng đó — KHÔNG có OCR hay dịch máy lúc chạy, mọi thứ tĩnh.
 *
 * PHẠM VI: CHỈ đặt vùng cho phần VĂN BẢN của bài — bài đọc (읽기), hội thoại
 * (말하기), và 문화와 정보. Ở Trung cấp, 문화와 정보 là một bài văn xuôi viết
 * bằng 한다체, thuộc loại khó nhất cả bài. Ở Sơ cấp nó ngắn hơn nhưng KHÔNG
 * chỉ là ảnh: gần như trang nào cũng có một đoạn giới thiệu 5–8 dòng cộng ô
 * câu hỏi thảo luận — đúng phần người mới học khó đọc nhất (bản đầu bỏ qua vì
 * nhận định nhầm là "chỉ có ảnh và vài dòng chú thích"). Câu hỏi thảo luận ở
 * đây mỗi trang một khác, không phải câu đề bài lặp lại, nên cũng được dịch.
 * Kèm theo là mấy trang in nguyên văn bản đời thật — đơn từ, thông báo tuyển
 * dụng, quy định đổi trả, áp phích, tin nhắn cảnh báo — vì đó đúng là thứ
 * người học phải đọc được ngoài đời.
 *
 * Câu đề bài, câu hỏi, dòng thay từ, gợi ý, bảng từ vựng có hình minh hoạ...
 * thì KHÔNG: đều là câu ngắn lặp đi lặp lại, người học quen rất nhanh — dịch
 * hết thì trang nào cũng chi chít vùng bấm mà chẳng giúp được gì thêm.
 *
 * Toạ độ lưu theo TỈ LỆ 0–1 của ảnh gốc (1200x1562), không phải pixel: ảnh
 * hiển thị co giãn theo màn hình và theo mức zoom, chỉ tỉ lệ mới bất biến.
 *
 * Import TĨNH như `lib/notes.ts` và `lib/quiz.ts`, cùng lý do (Output File
 * Tracing của Next không lần được file đọc qua path dựng động lúc chạy).
 */
import step1 from "../../content/translate/step1.json";
import step2 from "../../content/translate/step2.json";
import step3 from "../../content/translate/step3.json";
import step4 from "../../content/translate/step4.json";

/** [x, y, rộng, cao] — tất cả theo tỉ lệ 0–1 của ảnh trang. */
export type Rect = [number, number, number, number];

export interface TranslationRegion {
  id: string;
  rect: Rect;
  /** Nhãn ngắn hiện trên vùng, ví dụ "Hội thoại", "Bài đọc". */
  label?: string;
  /** Nguyên văn tiếng Hàn trong vùng (để đối chiếu khi đọc bản dịch). */
  ko?: string;
  /** Bản dịch tiếng Việt — đã dịch sẵn. */
  vi: string;
  /**
   * Chỗ đặt chấm dịch [x, y] (tâm chấm, tỉ lệ 0–1), khi chỗ mặc định —
   * ngay dưới mép dưới `rect`, canh giữa — bị chữ khác chiếm.
   *
   * Hay gặp ở trang 문화와 정보 Sơ cấp: ô câu hỏi nằm sát ngay dưới đoạn văn.
   * Trước khi có trường này, cách duy nhất là kéo giãn `rect` cho tâm của nó
   * dời sang chỗ trống — chấm rơi xuống dưới một bức ảnh bên cạnh, xa hẳn
   * đoạn nó dịch. Giờ `rect` luôn khớp đúng khối chữ, còn chấm đặt ở chỗ tự
   * nhiên nhất (thường là cuối dòng cuối của đoạn).
   */
  dot?: [number, number];
}

/**
 * Ép kiểu qua `unknown`: TypeScript suy `rect` trong JSON thành `number[]`,
 * không khớp tuple 4 phần tử `Rect` nên không ép thẳng được. Hình dạng thật
 * đã được `scripts/build-content.ts` kiểm lúc build (đủ 4 số, nằm trong ảnh).
 */
const TRANSLATIONS = {
  step1,
  step2,
  step3,
  step4,
} as unknown as Record<string, Record<string, TranslationRegion[]>>;

export function getPageTranslations(
  bookId: string,
  page: number
): TranslationRegion[] {
  return TRANSLATIONS[bookId]?.[String(page)] ?? [];
}

/** Danh sách số trang đã có bản dịch của 1 sách, sắp xếp tăng dần. */
export function getTranslatedPages(bookId: string): number[] {
  const book = TRANSLATIONS[bookId];
  if (!book) return [];
  return Object.keys(book)
    .map(Number)
    .sort((a, b) => a - b);
}
