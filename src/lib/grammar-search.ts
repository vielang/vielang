import type { GrammarEntry } from "@/lib/page-grammar";

/**
 * Chuẩn hoá chuỗi để so khớp khi tìm kiếm.
 *
 * Bỏ dấu tiếng Việt là chủ ý, không phải cho tiện: người Việt gõ tiếng Việt
 * không dấu rất nhiều, nhất là trên điện thoại. Bắt gõ đúng "điều kiện" mới
 * ra kết quả thì ô tìm kiếm coi như hỏng với phân nửa người dùng.
 *
 * NFD tách dấu thành ký tự tổ hợp riêng rồi xoá dải U+0300–U+036F, sau đó
 * NFC GHÉP LẠI. Bước ghép lại là bắt buộc, không phải cho đẹp: NFD tách cả
 * âm tiết Hàn thành jamo, mà jamo của `도` lại đúng là phần đầu của jamo
 * `동`. Thiếu NFC thì tra `도` khớp luôn mọi tiêu đề mở đầu bằng nhãn từ
 * loại `동` — 50 trên 72 mục, toàn thứ không liên quan. Chữ Latinh đã bị
 * lột dấu ở bước trước nên NFC không dựng lại được dấu nào.
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
    .normalize("NFC")
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
 * Từ khi chỉ tra tiêu đề và phần nghĩa, một âm tiết cho trung bình 7 kết
 * quả và nhiều nhất là 11, nên lằn cắt này gần như không còn chạm tới. Giữ
 * lại làm lưới chắn: nhãn từ loại `동` có mặt ở 49 trên 72 tiêu đề, gõ trúng
 * nó thì danh sách dài vô ích. Đó không phải phép tra thật — không ai tra
 * ngữ pháp bằng cách gõ "động từ" — nhưng cũng không nên để nó đổ cả kho ra
 * màn hình.
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
 * Các trường đem ra so khớp, kèm trọng số.
 *
 * CHỈ tiêu đề và nghĩa tiếng Việt. Trước đây tra cả câu ví dụ và phần giải
 * thích tiếng Hàn, với lý do người ta hay nhớ mang máng một câu trong sách
 * hơn là nhớ tên đuôi câu. Đo trên dữ liệu thật thì lý do đó không đứng
 * vững: gõ một âm tiết như `에` trả về trung bình 36 mục, trong đó điểm ngữ
 * pháp đúng nằm tận hạng 53 — tức là tra xong vẫn không thấy. Bỏ hai trường
 * kia đưa con số đó về 11.
 *
 * Nghĩa tiếng Việt thì PHẢI giữ, dù thu hẹp tới đâu. Tiêu đề toàn chữ Hàn,
 * nên bỏ `vi` là mất sạch đường tra bằng tiếng Việt — mà phần lớn người học
 * chưa thuộc tên tiếng Hàn của điểm ngữ pháp thì mới phải đi tra. Giữ nó
 * cũng gần như không tốn gì khi tra bằng tiếng Hàn: chữ Việt không chứa chữ
 * Hàn nên chỉ thêm khoảng một kết quả cho mỗi lần tra.
 *
 * Cái mất: không còn tra được bằng một câu ví dụ nhớ lõm bõm. Chấp nhận
 * được — mọi mục đều vẫn tra được qua tiêu đề hoặc nghĩa, không mục nào
 * thành mồ côi.
 */
const WEIGHTS = { title: 8, vi: 4 } as const;

/** Ranh giới giữa các thành tố trong tiêu đề: `명 이/가`, `동 형 -는데 (대조)`. */
const BOUNDARY = /[\s/,.?!~()-]/;

/**
 * Từ khoá khớp vào trường này mạnh cỡ nào.
 *
 * Khớp trọn một thành tố đáng giá hơn hẳn khớp lọt vào giữa chữ khác: gõ
 * `이` thì `명 이/가` phải đứng trước `명 이나`, và gõ `도` thì `명 도` phải
 * đứng trước một câu nghĩa tình cờ có `도`. Thiếu bậc này thì danh sách xếp
 * theo thứ tự giáo trình, và thứ người ta tìm nằm lẫn đâu đó giữa chừng.
 */
function fieldScore(text: string, needle: string): number {
  const at = text.indexOf(needle);
  if (at < 0) return 0;
  if (text === needle) return 8;
  if (text.split(BOUNDARY).includes(needle)) return 4;
  return at === 0 || BOUNDARY.test(text[at - 1]) ? 2 : 1;
}

/**
 * Lọc và xếp hạng danh sách ngữ pháp theo từ khoá.
 *
 * Xếp theo độ liên quan chứ không theo thứ tự giáo trình. Với từ khoá dài
 * thì hai cách gần như nhau, nhưng đuôi ngữ pháp tiếng Hàn phần lớn dài một
 * âm tiết và khớp rất nhiều mục, nên thứ tự giáo trình đẩy mục đúng xuống
 * dưới lằn cắt `MAX_VISIBLE_RESULTS` — tra `을` thì `동 -을` nằm hạng 56.
 *
 * Từ khoá rỗng thì trả nguyên danh sách theo thứ tự giáo trình: trang tra
 * cứu lúc mới mở phải thấy được toàn bộ, và lúc đó không có gì để xếp hạng.
 */
export function searchGrammar(
  entries: GrammarEntry[],
  query: string
): GrammarEntry[] {
  const needle = normalize(query);
  if (needle === "") return entries;

  const scored: { entry: GrammarEntry; score: number }[] = [];
  for (const entry of entries) {
    const score = Math.max(
      WEIGHTS.title * fieldScore(normalize(entry.title), needle),
      WEIGHTS.vi * fieldScore(normalize(entry.vi), needle)
    );
    if (score > 0) scored.push({ entry, score });
  }

  // Điểm bằng nhau thì tiêu đề ngắn hơn lên trước, vì nó cụ thể hơn: gõ `에`
  // thì `명 에` đáng đứng trên `명 에 있어요`. Bằng nốt thì giữ nguyên thứ tự
  // giáo trình — `sort` của JS ổn định nên điều đó tự đúng.
  scored.sort((a, b) => b.score - a.score || a.entry.title.length - b.entry.title.length);

  return scored.map((s) => s.entry);
}
