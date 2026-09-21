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
