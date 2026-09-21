import { describe, expect, it } from "vitest";
import { getGrammarPageNumbers, isGrammarPage } from "./grammar-pages";
import { getGrammarPages, getPageGrammar } from "./page-grammar";

/** Sách đã soạn nội dung ngữ pháp. Thêm sách mới thì thêm vào đây. */
const BOOKS_WITH_GRAMMAR = ["step1", "step2", "step3"] as const;

/**
 * Mọi điểm ngữ pháp của mọi sách, để các luật dưới đây áp cho tất cả.
 *
 * Chạy qua từng sách chứ không chốt cứng "step1": soạn sách mới mà test chỉ
 * kiểm sách cũ thì mọi luật ở đây thành vô dụng đúng lúc cần nhất.
 */
function everyPoint(): { book: string; page: number }[] {
  return BOOKS_WITH_GRAMMAR.flatMap((book) =>
    getGrammarPages(book).map((page) => ({ book, page }))
  );
}

/**
 * Bố cục bài học Sơ cấp: mỗi bài đúng 2 điểm ngữ pháp, ở offset +3 và +5.
 * Đã xác minh bằng ảnh trang thật ở đầu, giữa và cuối sách (trang 15, 25,
 * 195).
 */
describe("trang ngữ pháp của Sơ cấp 1", () => {
  it("mỗi bài đúng 2 trang, đủ 18 bài", () => {
    expect(getGrammarPageNumbers("step1")).toHaveLength(36);
  });

  it("bắt đầu và kết thúc đúng chỗ đã xác minh bằng ảnh", () => {
    const pages = getGrammarPageNumbers("step1");

    expect(pages.slice(0, 4)).toEqual([15, 17, 25, 27]);
    expect(pages.slice(-2)).toEqual([195, 197]);
  });

  it("không nhận nhầm trang bài đọc hay hội thoại", () => {
    // 18 là hội thoại, 19 là bài đọc — gắn chấm ngữ pháp vào đó là sai hẳn.
    for (const page of [18, 19, 20, 22]) {
      expect(isGrammarPage("step1", page)).toBe(false);
    }
  });

  it("Trung cấp 1: hai trang ngữ pháp LIỀN NHAU, khác Sơ cấp", () => {
    // Sơ cấp là +3/+5, Trung cấp 1 là +3/+4. Suy bừa theo offset của Sơ cấp
    // là gắn chấm vào trang không có ngữ pháp.
    expect(getGrammarPageNumbers("step3").slice(0, 4)).toEqual([15, 16, 27, 28]);
    expect(getGrammarPageNumbers("step3")).toHaveLength(32);
  });

  it("Trung cấp 1: bài 9 trở đi vẫn đúng chỗ số trang nhảy quãng", () => {
    // Bài 8 bắt đầu ở 96 nhưng bài 9 ở 118, không phải 108. Tính offset từ
    // một trang bắt đầu cố định là lệch hết nửa sau quyển sách.
    expect(getGrammarPageNumbers("step3")).toContain(121);
    expect(getGrammarPageNumbers("step3")).toContain(122);
    expect(getGrammarPageNumbers("step3")).not.toContain(111);
  });

  it("Trung cấp 2 chưa khảo sát thì trả về rỗng, không đoán bừa", () => {
    // Dùng chung bảng audio với Trung cấp 1 nhưng chừng đó không đủ để suy
    // ra vị trí trang ngữ pháp.
    expect(getGrammarPageNumbers("step4")).toEqual([]);
  });
});

describe("nội dung ngữ pháp đã soạn", () => {
  it("chỉ nằm trên trang thật sự là trang ngữ pháp", () => {
    // Soạn tay nên rất dễ gõ nhầm số trang; nhầm là cái chấm mọc giữa bài
    // đọc mà không có lỗi nào bắn ra.
    for (const { book, page } of everyPoint()) {
      expect(isGrammarPage(book, page)).toBe(true);
    }
  });

  it("mỗi điểm có mã tra cứu hợp lệ", () => {
    // `slug` là mã của CHÍNH điểm ngữ pháp, không gắn với trang — sau này
    // trang tra cứu gom theo mã này.
    for (const { book, page } of everyPoint()) {
      for (const point of getPageGrammar(book, page)) {
        expect(point.slug).toMatch(/^[a-z0-9-]+$/);
        expect(point.vi.trim()).not.toBe("");
      }
    }
  });

  it("định nghĩa đủ NGẮN để nằm trong một bong bóng nhỏ", () => {
    // Bong bóng nổi trên chính trang sách đang đọc. Định nghĩa dài là che
    // mất thứ người ta đang học — đúng cái đã phải sửa ở bản đầu. Build
    // cũng chặn (xem MAX_GRAMMAR_VI), chốt thêm ở đây cho khỏi trôi dần.
    for (const { book, page } of everyPoint()) {
      for (const point of getPageGrammar(book, page)) {
        expect(point.vi.length).toBeLessThanOrEqual(200);
      }
    }
  });

  it("điểm nào cũng có câu ví dụ kèm bản dịch", () => {
    // Định nghĩa thuần thì đúng nhưng khô. Thiếu ví dụ ở một trang là đúng
    // trang đó người học phải tự đoán.
    for (const { book, page } of everyPoint()) {
      for (const point of getPageGrammar(book, page)) {
        expect(point.exKo.trim()).not.toBe("");
        expect(point.exVi.trim()).not.toBe("");
      }
    }
  });

  it("cả bong bóng vẫn đọc lướt được trong vài giây", () => {
    // Chốt TỔNG chứ không chỉ từng trường: mỗi trường riêng lẻ đều lọt
    // ngưỡng mà cộng lại vẫn có thể thành một khối chữ dày. Đo thật thì
    // trang dài nhất đang 161 — để ngưỡng 240 cho dư chỗ xoay xở nhưng
    // vẫn chặn được việc nó phình dần thành bài giảng.
    for (const { book, page } of everyPoint()) {
      for (const p of getPageGrammar(book, page)) {
        const total = p.title.length + p.vi.length + p.exKo.length + p.exVi.length;
        expect(total).toBeLessThanOrEqual(240);
      }
    }
  });

});

/**
 * Chỗ đặt chấm ngữ pháp trên trang.
 *
 * Bản đầu đặt cạnh tiêu đề ở góc trên cùng (y≈0.03). Chỗ đó hỏng thật sự
 * chứ không chỉ khó với tay: thanh công cụ của trang đọc là
 * `fixed top-0 z-20` (xem `reader-controls.tsx`), mà ở chế độ 2 trang ảnh
 * bị giới hạn theo chiều cao nên lấp đầy màn hình — chấm rơi vào khoảng
 * 11px từ mép trên, nằm gọn dưới thanh công cụ cao ~60px. Trên điện thoại
 * nằm ngang thì KHÔNG BẤM ĐƯỢC.
 */
describe("chỗ đặt chấm ngữ pháp", () => {
  /** Tâm chấm theo chiều dọc, tính theo tỉ lệ 0–1 của ảnh trang. */
  function centerY(rect: readonly number[]): number {
    return rect[1] + rect[3] / 2;
  }

  it("thoát hẳn khỏi dải thanh công cụ ở mép trên", () => {
    for (const { book, page } of everyPoint()) {
      for (const p of getPageGrammar(book, page)) {
        expect(centerY(p.rect)).toBeGreaterThan(0.15);
      }
    }
  });

  it("nằm ở tầm giữa trang, chỗ ngón cái với tới", () => {
    // Quá thấp thì lại chui xuống dưới thanh điều khiển ở mép dưới, cũng
    // `fixed bottom-0 z-20`.
    for (const { book, page } of everyPoint()) {
      for (const p of getPageGrammar(book, page)) {
        expect(centerY(p.rect)).toBeLessThan(0.55);
      }
    }
  });

  it("Sơ cấp: nằm ở cột phải, tránh tranh hội thoại bên trái", () => {
    // Ở Sơ cấp chấm nằm NGANG HÀNG với tranh, nên phải né sang phải.
    for (const book of ["step1", "step2"]) {
      for (const page of getGrammarPages(book)) {
        for (const p of getPageGrammar(book, page)) {
          expect(p.rect[0]).toBeGreaterThan(0.5);
        }
      }
    }
  });

  it("Trung cấp 1: nằm hẳn DƯỚI khung giải thích nên không che gì", () => {
    // Trung cấp không né sang phải được: khe giữa hộp ví dụ và hộp chia
    // đuôi bị chữ lấp mất trên 22/32 trang. Bù lại khung giải thích kết
    // thúc ở y≈0.43 trên mọi trang, nên dải trắng ngay dưới nó vừa trống
    // vừa ổn định — đặt giữa trang cũng không đè lên gì.
    for (const page of getGrammarPages("step3")) {
      for (const p of getPageGrammar("step3", page)) {
        expect(p.rect[1]).toBeGreaterThan(0.41);
      }
    }
  });
});

describe("trang ngữ pháp của Sơ cấp 2", () => {
  it("cùng bố cục với Sơ cấp 1 — mỗi bài 2 trang, đủ 18 bài", () => {
    expect(getGrammarPageNumbers("step2")).toHaveLength(36);
    expect(getGrammarPageNumbers("step2")).toEqual(getGrammarPageNumbers("step1"));
  });

  it("đã soạn đủ nội dung cho mọi trang", () => {
    // Sót một trang là người học mở ra không thấy chấm, mà không có lỗi nào
    // báo cho ai biết.
    expect(getGrammarPages("step2")).toEqual(getGrammarPageNumbers("step2"));
  });
});

describe("mã ngữ pháp dùng chung giữa các sách", () => {
  it("cùng mã thì phải cùng tiêu đề, kể cả khác sách", () => {
    // `slug` là mã của CHÍNH điểm ngữ pháp, không gắn với sách — sau này
    // trang tra cứu gom theo mã này. Hai điểm khác nhau mà trùng mã là gom
    // nhầm chúng vào một chỗ.
    const bySlug = new Map<string, string>();
    for (const { book, page } of everyPoint()) {
      for (const point of getPageGrammar(book, page)) {
        const seen = bySlug.get(point.slug);
        if (seen !== undefined) expect(point.title).toBe(seen);
        bySlug.set(point.slug, point.title);
      }
    }
    expect(bySlug.size).toBeGreaterThan(36);
  });
});

describe("nội dung phủ đủ bố cục", () => {
  it("mọi trang ngữ pháp của sách đã soạn đều CÓ nội dung", () => {
    // Test kia chỉ chốt chiều ngược lại: nội dung không nằm nhầm trang.
    // Thiếu chiều này thì soạn 30/32 trang vẫn xanh hết, và hai trang bỏ
    // quên chỉ lộ ra khi có người lật đúng tới đó.
    for (const book of BOOKS_WITH_GRAMMAR) {
      expect(getGrammarPages(book)).toEqual(getGrammarPageNumbers(book));
    }
  });

  it("Trung cấp 1 đủ 32 điểm, mỗi trang đúng một điểm", () => {
    const pages = getGrammarPages("step3");
    expect(pages).toHaveLength(32);
    for (const page of pages) {
      expect(getPageGrammar("step3", page)).toHaveLength(1);
    }
  });
});
