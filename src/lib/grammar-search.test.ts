import { describe, expect, it } from "vitest";
import { hasEnoughQuery, normalize, searchGrammar } from "./grammar-search";
import { getAllGrammar, type GrammarEntry } from "./page-grammar";

const entry = (over: Partial<GrammarEntry> = {}): GrammarEntry => ({
  id: "x",
  slug: "x",
  rect: [0, 0, 1, 1],
  title: "동 형 -으면",
  ko: "앞 내용이 뒤 내용의 조건이나 가정이 될 때 사용해요.",
  vi: "Nêu điều kiện hoặc giả định — \"nếu… thì…\".",
  exKo: "시간이 있으면 밥 먹을까요?",
  exVi: "Nếu có thời gian thì mình đi ăn nhé?",
  bookId: "step2",
  page: 55,
  ...over,
});

describe("chuẩn hoá chuỗi", () => {
  it("bỏ dấu tiếng Việt", () => {
    // Người Việt gõ không dấu rất nhiều, nhất là trên điện thoại.
    expect(normalize("điều kiện")).toBe("dieu kien");
    expect(normalize("Nếu… thì…")).toBe("neu… thi…");
  });

  it("xử lý được chữ đ", () => {
    // `đ` là chữ cái riêng trong bảng mã, NFD không tách ra được như dấu.
    expect(normalize("đúng")).toBe("dung");
    expect(normalize("Đi")).toBe("di");
  });

  it("không làm hỏng chữ Hàn", () => {
    // Chuẩn hoá hai lần phải cho cùng kết quả, nếu không thì từ khoá và nội
    // dung tách khác nhau và không bao giờ khớp.
    expect(normalize("이에요")).toBe(normalize("이에요"));
    expect(normalize("이에요")).not.toBe("");
  });
});

describe("tìm kiếm", () => {
  // Hai mục phải khác nhau ở MỌI trường có thể tìm được, kể cả câu ví dụ —
  // nếu không thì một phép tìm tưởng là khớp tiêu đề lại vô tình khớp cả ví
  // dụ mặc định, và test hoá ra kiểm nhầm thứ.
  const list = [
    entry({ id: "a", title: "동 형 -으면", vi: "Nêu điều kiện" }),
    entry({
      id: "b",
      title: "명 이에요/예요",
      vi: "Dùng khi nói tên",
      ko: "사람, 사물 이름을 말할 때 사용해요.",
      exKo: "저는 후엔이에요.",
      exVi: "Tôi là Huyền.",
    }),
  ];

  it("tìm được bằng tiếng Hàn", () => {
    expect(searchGrammar(list, "이에요").map((e) => e.id)).toEqual(["b"]);
  });

  it("tìm được bằng tiếng Việt CÓ dấu", () => {
    expect(searchGrammar(list, "điều kiện").map((e) => e.id)).toEqual(["a"]);
  });

  it("tìm được bằng tiếng Việt KHÔNG dấu", () => {
    // Đây là lý do chính phải chuẩn hoá. Thiếu nó là ô tìm kiếm hỏng với
    // phân nửa người dùng mà chẳng ai báo lỗi.
    expect(searchGrammar(list, "dieu kien").map((e) => e.id)).toEqual(["a"]);
  });

  it("không phân biệt hoa thường", () => {
    expect(searchGrammar(list, "NÊU").map((e) => e.id)).toEqual(["a"]);
  });

  it("tìm được cả trong câu ví dụ", () => {
    // Người ta hay nhớ mang máng một câu trong sách hơn là nhớ tên đuôi câu.
    const found = searchGrammar([entry({ id: "c" })], "밥 먹을까요");
    expect(found.map((e) => e.id)).toEqual(["c"]);
  });

  it("từ khoá rỗng thì trả về TẤT CẢ, không phải rỗng", () => {
    // Trang tra cứu lúc mới mở phải thấy được toàn bộ.
    expect(searchGrammar(list, "")).toHaveLength(2);
    expect(searchGrammar(list, "   ")).toHaveLength(2);
  });

  it("không khớp thì trả về rỗng", () => {
    expect(searchGrammar(list, "xyzzy")).toEqual([]);
  });
});

describe("dữ liệu thật", () => {
  const all = getAllGrammar();

  it("gom đủ điểm ngữ pháp của cả hai sách", () => {
    expect(all).toHaveLength(72);
  });

  it("xếp theo đúng thứ tự học, không phải bảng chữ cái", () => {
    // Người tra cứu hầu hết đang theo giáo trình.
    expect(all[0]).toMatchObject({ bookId: "step1", page: 15 });
    expect(all[35]).toMatchObject({ bookId: "step1", page: 197 });
    expect(all[36]).toMatchObject({ bookId: "step2", page: 15 });
    expect(all.at(-1)).toMatchObject({ bookId: "step2", page: 197 });
  });

  it("mỗi mục biết mình nằm ở sách nào, trang nào", () => {
    // Thiếu là liên kết "mở trang trong sách" trỏ sai chỗ.
    for (const e of all) {
      expect(e.bookId).toMatch(/^step[12]$/);
      expect(e.page).toBeGreaterThan(0);
    }
  });

  it("tra được một đuôi câu có thật trong sách", () => {
    const found = searchGrammar(all, "-지요");
    expect(found.length).toBeGreaterThan(0);
    expect(found[0].bookId).toBe("step1");
  });
});

/**
 * Đây là lỗi người dùng báo: "chỉ search được tiếng Việt, không search được
 * tiếng Hàn". Nguyên nhân không nằm ở chuẩn hoá hay so khớp — cả hai vẫn
 * đúng — mà ở ngưỡng hai ký tự. Đuôi ngữ pháp tiếng Hàn phần lớn dài đúng
 * MỘT âm tiết nên không đuôi nào với tới ngưỡng, trong khi từ tiếng Việt tự
 * nhiên đã dài hai ký tự trở lên và không bao giờ vướng.
 *
 * Kiểm trên dữ liệu THẬT chứ không trên mẫu tự chế: điều cần chốt là mấy
 * đuôi này tra ra được thứ có trong sách, không phải là hàm ngưỡng trả về
 * `true`.
 */
describe("tra bằng một âm tiết tiếng Hàn", () => {
  const all = getAllGrammar();

  for (const ending of ["은", "는", "이", "가", "도", "에", "을", "지", "고"]) {
    it(`"${ending}" tra được và có khớp thật trong sách`, () => {
      expect(hasEnoughQuery(ending)).toBe(true);
      expect(searchGrammar(all, ending).length).toBeGreaterThan(0);
    });
  }
});

describe("ngưỡng bắt đầu tra", () => {
  it("chưa gõ gì thì chưa tra", () => {
    expect(hasEnoughQuery("")).toBe(false);
    expect(hasEnoughQuery("  ")).toBe(false);
  });

  it("chữ Latinh: một ký tự là chưa đủ", () => {
    // Ô nằm ngay đầu trang thư viện; một chữ cái đổ ra hàng chục kết quả là
    // đẩy tụt lưới sách xuống mỗi lần chạm nhầm.
    expect(hasEnoughQuery("d")).toBe(false);
    expect(hasEnoughQuery("di")).toBe(true);
    expect(hasEnoughQuery("điều kiện")).toBe(true);
  });

  it("chữ Hàn: MỘT âm tiết đã đủ", () => {
    // Gõ một chữ Hàn trong giao diện tiếng Việt không bao giờ là chạm nhầm,
    // nên không có cái giá mà ngưỡng kia sinh ra để tránh.
    expect(hasEnoughQuery("는")).toBe(true);
    expect(hasEnoughQuery("이")).toBe(true);
  });

  it("jamo rời lúc bộ gõ đang ghép cũng tính là chữ Hàn", () => {
    // Bàn phím Hàn bắn ra jamo tương thích (U+3130–318F) giữa chừng; nếu dải
    // nhận diện bỏ sót dải này thì ô đứng im đúng lúc người ta đang gõ.
    expect(hasEnoughQuery("ㅇ")).toBe(true);
    expect(hasEnoughQuery("ㄴ")).toBe(true);
  });

  it("lẫn Hàn với ký tự khác thì vẫn tra", () => {
    // Sách viết đuôi câu kèm gạch nối, và người ta chép y như vậy.
    expect(hasEnoughQuery("-지")).toBe(true);
  });

  it("bỏ qua khoảng trắng thừa hai đầu", () => {
    expect(hasEnoughQuery("  d  ")).toBe(false);
    expect(hasEnoughQuery("  di  ")).toBe(true);
  });
});
