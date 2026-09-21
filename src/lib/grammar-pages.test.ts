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
        expect(point.html.trim()).not.toBe("");
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
