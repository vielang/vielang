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
 */
export function normalize(text: string): string {
  return text
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toLowerCase()
    .trim();
}

/**
 * Số ký tự tối thiểu trước khi hiện danh sách kết quả.
 *
 * Ô tra cứu nằm ngay đầu trang thư viện, nên nếu gõ một ký tự đã đổ ra hàng
 * chục kết quả thì nó đẩy tụt lưới sách xuống mỗi lần người ta chạm nhầm.
 * Hai ký tự là đủ để chủ ý tra cứu mới hiện.
 */
export const MIN_QUERY_LENGTH = 2;

/**
 * Số kết quả vẽ ra tối đa.
 *
 * Từ khoá ngắn và phổ biến khớp rất nhiều — "vi" khớp tới 30 mục. Vẽ hết ra
 * thì vừa chậm trên máy yếu vừa vô dụng: không ai đọc ba chục thẻ để tìm một
 * cái. Cắt bớt rồi mời gõ thêm cho hẹp lại.
 */
export const MAX_VISIBLE_RESULTS = 20;

/**
 * Từ khoá đã đủ dài để tra chưa.
 *
 * Tách riêng khỏi `searchGrammar` vì giao diện cần phân biệt BA trạng thái:
 * chưa gõ đủ (không hiện gì), gõ đủ mà không khớp (báo không tìm thấy), và
 * có kết quả. Gộp "chưa đủ" thành mảng rỗng là mất mất trạng thái giữa, rồi
 * người dùng gõ một chữ lại thấy "không tìm thấy" — sai và gây hoang mang.
 *
 * Đếm trên chuỗi GỐC chứ KHÔNG trên chuỗi đã chuẩn hoá: NFD tách một âm
 * tiết Hàn thành các jamo rời, nên `이` đếm ra 2 và lọt chốt ngay khi vừa gõ
 * một chữ. Đếm chuỗi gốc thì ngưỡng khớp với thứ người dùng NHÌN THẤY —
 * hai chữ là hai chữ, dù là Hàn hay Việt.
 *
 * Trải chuỗi qua `[...]` để đếm theo ký tự chứ không theo đơn vị UTF-16.
 */
export function hasEnoughQuery(query: string): boolean {
  return [...query.trim()].length >= MIN_QUERY_LENGTH;
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
