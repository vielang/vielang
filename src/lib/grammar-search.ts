import type { GrammarEntry } from "@/lib/page-grammar";

/**
 * Chuẩn hoá chuỗi để so khớp khi tìm kiếm.
 *
 * Bỏ dấu tiếng Việt là chủ ý, không phải cho tiện: người Việt gõ tiếng Việt
 * không dấu rất nhiều, nhất là trên điện thoại. Bắt gõ đúng "điều kiện" mới
 * ra kết quả thì ô tìm kiếm coi như hỏng với phân nửa người dùng.
 *
 * NFD tách dấu thành ký tự tổ hợp riêng rồi xoá dải U+0300–U+036F. Dải đó
 * chỉ chứa dấu của chữ Latinh nên chữ Hàn không bị đụng tới — jamo nằm ở
 * dải khác hẳn. Quan trọng là CẢ HAI phía (từ khoá và nội dung) đều đi qua
 * đúng hàm này, nên dù chữ Hàn có bị NFD tách ra thì hai bên vẫn tách giống
 * nhau và vẫn khớp.
 *
 * `đ` phải xử lý riêng: nó là một chữ cái độc lập trong bảng mã chứ không
 * phải `d` cộng dấu, nên NFD không tách được.
 *
 * Dải ký tự BẮT BUỘC viết bằng escape `\u0300` chứ không phải ký tự thật:
 * dấu tổ hợp dán luôn vào dấu ngoặc vuông đứng trước và biến cả lớp ký tự
 * thành một vệt vô hình trong trình soạn thảo — không ai đọc ra, không ai
 * sửa nổi, và một lần chuẩn hoá Unicode nhầm là hỏng lặng lẽ.
 */
export function normalize(text: string): string {
  return text
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toLowerCase()
    .trim();
}

/**
 * Số ký tự tối thiểu trước khi hiện danh sách kết quả — cho chữ LATINH.
 *
 * Ô tra cứu nằm ngay đầu trang thư viện, nên nếu gõ một ký tự đã đổ ra hàng
 * chục kết quả thì nó đẩy tụt lưới sách xuống mỗi lần người ta chạm nhầm.
 * Hai ký tự là đủ để chủ ý tra cứu mới hiện.
 */
export const MIN_QUERY_LENGTH = 2;

/**
 * Số kết quả vẽ ra tối đa.
 *
 * Từ khoá ngắn và phổ biến khớp rất nhiều — `이` khớp tới 65 trên 72 mục. Vẽ
 * hết ra thì vừa chậm trên máy yếu vừa vô dụng: không ai đọc sáu chục thẻ để
 * tìm một cái. Cắt bớt rồi mời gõ thêm cho hẹp lại.
 */
export const MAX_VISIBLE_RESULTS = 20;

/**
 * Chuỗi có chứa chữ Hàn không — kể cả jamo rời lúc bộ gõ đang ghép dở.
 *
 * U+1100–11FF là jamo kết hợp, U+3130–318F là jamo tương thích (thứ bàn phím
 * bắn ra giữa chừng), U+AC00–D7AF là âm tiết hoàn chỉnh.
 */
const HANGUL = /[\u1100-\u11ff\u3130-\u318f\uac00-\ud7af]/;

/**
 * Từ khoá đã đủ để tra chưa.
 *
 * Tách riêng khỏi `searchGrammar` vì giao diện cần phân biệt BA trạng thái:
 * chưa gõ đủ (không hiện gì), gõ đủ mà không khớp (báo không tìm thấy), và
 * có kết quả. Gộp "chưa đủ" thành mảng rỗng là mất mất trạng thái giữa, rồi
 * người dùng gõ một chữ lại thấy "không tìm thấy" — sai và gây hoang mang.
 *
 * MỘT âm tiết Hàn là đủ, và đây mới là phần quan trọng. Đuôi ngữ pháp tiếng
 * Hàn phần lớn chỉ dài một âm tiết — `은`, `는`, `이`, `가`, `도`, `에`, `을`
 * — nên bắt gõ hai ký tự là khoá sạch phân nửa kho ngữ pháp: gõ `는` không
 * bao giờ ra gì trong khi nó khớp 37 mục. Ngưỡng hai ký tự chỉ hợp với chữ
 * Latinh, nơi một chữ cái đơn lẻ gần như không mang thông tin gì.
 *
 * Gõ một chữ Hàn trong giao diện tiếng Việt cũng không bao giờ là chạm nhầm,
 * nên cái giá "đẩy tụt lưới sách" mà ngưỡng kia sinh ra để tránh không tồn
 * tại ở đây.
 *
 * Đếm trên chuỗi GỐC chứ KHÔNG trên chuỗi đã chuẩn hoá: NFD tách một âm tiết
 * Hàn thành các jamo rời nên số đếm không còn khớp với thứ người dùng NHÌN
 * THẤY. Trải chuỗi qua `[...]` để đếm theo ký tự chứ không theo đơn vị UTF-16.
 */
export function hasEnoughQuery(query: string): boolean {
  const trimmed = query.trim();
  if (trimmed === "") return false;
  if (HANGUL.test(trimmed)) return true;
  return [...trimmed].length >= MIN_QUERY_LENGTH;
}

/**
 * Lọc danh sách ngữ pháp theo từ khoá.
 *
 * Tìm trong tiêu đề tiếng Hàn, định nghĩa tiếng Việt VÀ câu ví dụ — người ta
 * hay nhớ mang máng một câu trong sách hơn là nhớ tên đuôi câu.
 *
 * Từ khoá rỗng thì trả nguyên danh sách chứ không trả rỗng: trang tra cứu
 * lúc mới mở phải thấy được toàn bộ, ô tìm kiếm chỉ là để thu hẹp.
 */
export function searchGrammar(
  entries: GrammarEntry[],
  query: string
): GrammarEntry[] {
  const needle = normalize(query);
  if (needle === "") return entries;

  return entries.filter((entry) =>
    [entry.title, entry.vi, entry.exKo, entry.exVi, entry.ko ?? ""].some(
      (field) => normalize(field).includes(needle)
    )
  );
}
