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

  it("KHÔNG tra trong câu ví dụ", () => {
    // Thu hẹp có chủ ý, không phải bỏ sót. Tra cả câu ví dụ nghe thì rộng
    // rãi nhưng đo ra thì hỏng: gõ một âm tiết như 에 trả về trung bình 36
    // mục và điểm ngữ pháp đúng tụt xuống hạng 53, tức là tra xong vẫn
    // không thấy. Bỏ đi thì con số đó về 11.
    const found = searchGrammar([entry({ id: "c" })], "밥 먹을까요");
    expect(found).toEqual([]);
  });

  it("KHÔNG tra trong phần giải thích tiếng Hàn", () => {
    // Đây mới là nguồn nhiễu lớn nhất: riêng trường này đã thêm trung bình
    // 17 kết quả cho mỗi lần tra một âm tiết.
    const found = searchGrammar([entry({ id: "c" })], "조건이나 가정");
    expect(found).toEqual([]);
  });

  it("nhưng nghĩa tiếng Việt thì PHẢI tra được", () => {
    // Tiêu đề toàn chữ Hàn. Bỏ nốt trường này là mất sạch đường tra bằng
    // tiếng Việt — mà chưa thuộc tên tiếng Hàn thì mới phải đi tra.
    const found = searchGrammar([entry({ id: "c" })], "gia dinh");
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

  it("gom đủ điểm ngữ pháp của cả ba sách đã soạn", () => {
    // 36 + 36 ở Sơ cấp, 32 ở Trung cấp 1 (16 bài x 2, ít hơn vì Trung cấp
    // chỉ có 16 bài chứ không phải 18).
    expect(all).toHaveLength(104);
  });

  it("xếp theo đúng thứ tự học, không phải bảng chữ cái", () => {
    // Người tra cứu hầu hết đang theo giáo trình.
    expect(all[0]).toMatchObject({ bookId: "step1", page: 15 });
    expect(all[35]).toMatchObject({ bookId: "step1", page: 197 });
    expect(all[36]).toMatchObject({ bookId: "step2", page: 15 });
    expect(all[71]).toMatchObject({ bookId: "step2", page: 197 });
    expect(all[72]).toMatchObject({ bookId: "step3", page: 15 });
    expect(all.at(-1)).toMatchObject({ bookId: "step3", page: 206 });
  });

  it("mỗi mục biết mình nằm ở sách nào, trang nào", () => {
    // Thiếu là liên kết "mở trang trong sách" trỏ sai chỗ.
    for (const e of all) {
      expect(e.bookId).toMatch(/^step[123]$/);
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

/**
 * Lỗi âm thầm nhất trong cả phần này: NFD tách âm tiết Hàn thành jamo, mà
 * jamo của 도 đúng là phần đầu của jamo 동. Tra 도 khớp luôn mọi tiêu đề mở
 * đầu bằng nhãn từ loại 동 — 50 trên 72 mục. Không có gì báo lỗi, chỉ là kết
 * quả trông như rác.
 */
describe("không lẫn âm tiết Hàn", () => {
  it("chuẩn hoá xong vẫn là âm tiết trọn vẹn", () => {
    expect(normalize("동 형 -아요").includes(normalize("도"))).toBe(false);
    expect(normalize("명 이에요/예요").includes(normalize("이"))).toBe(true);
  });

  it("mọi kết quả đều THẬT SỰ chứa từ khoá", () => {
    // Bất biến quan trọng nhất của ô tra cứu. Đây là thứ đáng ra phải bắt
    // được lỗi trên ngay từ đầu: nó không chốt một con số cụ thể nào mà chốt
    // đúng cái điều người dùng tin là đang xảy ra.
    const all = getAllGrammar();
    for (const q of ["도", "는", "이", "을", "에", "면", "고", "지"]) {
      for (const e of searchGrammar(all, q)) {
        // So trên dạng NFC — thứ người dùng NHÌN THẤY — chứ không qua
        // `normalize()`. Dùng chính hàm đang kiểm để kiểm nó thì hai bên tự
        // xác nhận lẫn nhau: lỗi jamo ở trên lọt qua test này y như thường.
        expect(`${e.title} ${e.vi}`.normalize("NFC")).toContain(q.normalize("NFC"));
      }
    }
  });

  it("tra 도 ra vài mục chứ không ra nửa quyển sách", () => {
    const found = searchGrammar(getAllGrammar(), "도");
    expect(found.length).toBeLessThan(5);
    expect(found[0].title).toBe("명 도");
  });
});

/**
 * Xếp theo độ liên quan chứ không theo thứ tự giáo trình. Đuôi ngữ pháp
 * tiếng Hàn dài một âm tiết khớp rất nhiều mục, nên thứ tự giáo trình đẩy
 * mục đúng xuống dưới lằn cắt `MAX_VISIBLE_RESULTS`: tra 을 thì 동 -을 từng
 * nằm hạng 56, tức tra xong vẫn không thấy.
 */
describe("xếp hạng theo độ liên quan", () => {
  it("khớp tiêu đề đứng trên khớp phần nghĩa", () => {
    const list = [
      entry({ id: "trong-nghia", title: "동 -으면", vi: "So sánh, giống 보다." }),
      entry({ id: "trong-tieu-de", title: "명 보다", vi: "Dùng khi so sánh." }),
    ];
    expect(searchGrammar(list, "보다").map((e) => e.id)).toEqual([
      "trong-tieu-de",
      "trong-nghia",
    ]);
  });

  it("khớp trọn một thành tố đứng trên khớp lọt giữa chữ", () => {
    // Gõ 이 thì 명 이/가 phải đứng trước 명 이나.
    const list = [
      entry({ id: "lot-giua", title: "명 이나" }),
      entry({ id: "tron-ven", title: "명 이/가" }),
    ];
    expect(searchGrammar(list, "이")[0].id).toBe("tron-ven");
  });

  it("hoà điểm thì tiêu đề ngắn hơn lên trước", () => {
    // Ngắn hơn nghĩa là cụ thể hơn: gõ 에 thì 명 에 đáng đứng trên 명 에 있어요.
    const list = [
      entry({ id: "dai", title: "명 에 있어요" }),
      entry({ id: "ngan", title: "명 에" }),
    ];
    expect(searchGrammar(list, "에")[0].id).toBe("ngan");
  });

  it("trên dữ liệu thật, mục đúng đứng ngay hạng 1", () => {
    // Ba ca này trước đây nằm hạng 53, 56 và 27 — đều ngoài 20 mục hiển thị.
    const all = getAllGrammar();
    expect(searchGrammar(all, "에")[0].title).toBe("명 에");
    expect(searchGrammar(all, "을")[0].title).toBe("동 -을");
    expect(searchGrammar(all, "는")[0].title).toBe("동 -는");
  });
});
