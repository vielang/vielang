import { describe, expect, it } from "vitest";
import { getGrammarPageNumbers, isGrammarPage } from "./grammar-pages";
import { getGrammarPages, getPageGrammar } from "./page-grammar";

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

  it("Trung cấp chưa khảo sát thì trả về rỗng, không đoán bừa", () => {
    // Trung cấp 12 trang/bài, bố cục khác. Suy bừa theo offset của Sơ cấp là
    // gắn chấm vào trang không có ngữ pháp.
    expect(getGrammarPageNumbers("step3")).toEqual([]);
    expect(getGrammarPageNumbers("step4")).toEqual([]);
  });
});

describe("nội dung ngữ pháp đã soạn", () => {
  it("chỉ nằm trên trang thật sự là trang ngữ pháp", () => {
    // Soạn tay nên rất dễ gõ nhầm số trang; nhầm là cái chấm mọc giữa bài
    // đọc mà không có lỗi nào bắn ra.
    for (const page of getGrammarPages("step1")) {
      expect(isGrammarPage("step1", page)).toBe(true);
    }
  });

  it("mỗi điểm có mã tra cứu hợp lệ", () => {
    // `slug` là mã của CHÍNH điểm ngữ pháp, không gắn với trang — sau này
    // trang tra cứu gom theo mã này.
    for (const page of getGrammarPages("step1")) {
      for (const point of getPageGrammar("step1", page)) {
        expect(point.slug).toMatch(/^[a-z0-9-]+$/);
        expect(point.vi.trim()).not.toBe("");
      }
    }
  });

  it("định nghĩa đủ NGẮN để nằm trong một bong bóng nhỏ", () => {
    // Bong bóng nổi trên chính trang sách đang đọc. Định nghĩa dài là che
    // mất thứ người ta đang học — đúng cái đã phải sửa ở bản đầu. Build
    // cũng chặn (xem MAX_GRAMMAR_VI), chốt thêm ở đây cho khỏi trôi dần.
    for (const page of getGrammarPages("step1")) {
      for (const point of getPageGrammar("step1", page)) {
        expect(point.vi.length).toBeLessThanOrEqual(200);
      }
    }
  });

  it("điểm nào cũng có câu ví dụ kèm bản dịch", () => {
    // Định nghĩa thuần thì đúng nhưng khô. Thiếu ví dụ ở một trang là đúng
    // trang đó người học phải tự đoán.
    for (const page of getGrammarPages("step1")) {
      for (const point of getPageGrammar("step1", page)) {
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
    for (const page of getGrammarPages("step1")) {
      for (const p of getPageGrammar("step1", page)) {
        const total = p.title.length + p.vi.length + p.exKo.length + p.exVi.length;
        expect(total).toBeLessThanOrEqual(240);
      }
    }
  });

  it("không có hai điểm khác nhau dùng chung một mã", () => {
    const bySlug = new Map<string, string>();
    for (const page of getGrammarPages("step1")) {
      for (const point of getPageGrammar("step1", page)) {
        const seen = bySlug.get(point.slug);
        // Cùng mã thì phải cùng tiêu đề — nếu không là gom nhầm hai điểm
        // ngữ pháp khác nhau vào một chỗ.
        if (seen !== undefined) expect(point.title).toBe(seen);
        bySlug.set(point.slug, point.title);
      }
    }
    expect(bySlug.size).toBeGreaterThan(0);
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
    for (const page of getGrammarPages("step1")) {
      for (const p of getPageGrammar("step1", page)) {
        expect(centerY(p.rect)).toBeGreaterThan(0.15);
      }
    }
  });

  it("nằm ở tầm giữa trang, chỗ ngón cái với tới", () => {
    // Quá thấp thì lại chui xuống dưới thanh điều khiển ở mép dưới, cũng
    // `fixed bottom-0 z-20`.
    for (const page of getGrammarPages("step1")) {
      for (const p of getPageGrammar("step1", page)) {
        expect(centerY(p.rect)).toBeLessThan(0.55);
      }
    }
  });

  it("nằm ở cột phải, nơi có hộp ví dụ và hộp chia đuôi", () => {
    // Cột trái là tranh hội thoại — đặt chấm lên đó là che mất hình.
    for (const page of getGrammarPages("step1")) {
      for (const p of getPageGrammar("step1", page)) {
        expect(p.rect[0]).toBeGreaterThan(0.5);
      }
    }
  });
});
