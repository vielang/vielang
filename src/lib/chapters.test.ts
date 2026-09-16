import { describe, expect, it } from "vitest";
import { getAdjacentSpreadAnchor, getSpreadAnchor, getSpreadPages } from "./chapters";

// en-beginner: mốc Bài 1 = trang 7 (LẺ) -> trang lẻ = trái trong spread.
// en-pre-intermediate: mốc Bài 1 = trang 6 (CHẴN) -> trang chẵn = trái.
describe("getSpreadAnchor", () => {
  it("trang trái (khớp mốc Bài) thì tự làm anchor", () => {
    expect(getSpreadAnchor("en-beginner", 7)).toBe(7);
    expect(getSpreadAnchor("en-pre-intermediate", 6)).toBe(6);
  });

  it("trang phải thì lùi về trang trái liền trước", () => {
    expect(getSpreadAnchor("en-beginner", 8)).toBe(7);
    expect(getSpreadAnchor("en-pre-intermediate", 7)).toBe(6);
  });

  it("trang 1 không có trang trước thì tự đứng riêng dù parity là 'phải'", () => {
    // pre-intermediate: chẵn=trái -> trang 1 (lẻ) lẽ ra là "phải", không có
    // trang 0 để ghép nên phải tự làm anchor riêng.
    expect(getSpreadAnchor("en-pre-intermediate", 1)).toBe(1);
  });
});

describe("getSpreadPages", () => {
  it("ghép đúng 2 trang khi anchor là trang trái và còn trang kế", () => {
    expect(getSpreadPages("en-beginner", 7, 137)).toEqual([7, 8]);
    expect(getSpreadPages("en-pre-intermediate", 6, 167)).toEqual([6, 7]);
  });

  it("đứng riêng 1 trang nếu anchor không phải trang trái (trang 1 sách chẵn=trái)", () => {
    expect(getSpreadPages("en-pre-intermediate", 1, 167)).toEqual([1]);
  });

  it("đứng riêng 1 trang nếu trang kế vượt quá tổng số trang", () => {
    expect(getSpreadPages("en-beginner", 137, 137)).toEqual([137]);
  });
});

describe("getAdjacentSpreadAnchor", () => {
  it("tiến 2 trang khi spread hiện tại có 2 trang", () => {
    expect(getAdjacentSpreadAnchor("en-beginner", 7, 137, 1)).toBe(9);
  });

  it("tiến 1 trang khi spread hiện tại chỉ có 1 trang (đầu sách lệch parity)", () => {
    expect(getAdjacentSpreadAnchor("en-pre-intermediate", 1, 167, 1)).toBe(2);
  });

  it("lùi về đúng spread liền trước, không lùi nửa trang", () => {
    expect(getAdjacentSpreadAnchor("en-beginner", 9, 137, -1)).toBe(7);
    expect(getAdjacentSpreadAnchor("en-pre-intermediate", 2, 167, -1)).toBe(1);
  });

  it("không lùi quá trang 1", () => {
    expect(getAdjacentSpreadAnchor("en-beginner", 1, 137, -1)).toBe(1);
  });

  it("không tiến quá trang cuối", () => {
    expect(getAdjacentSpreadAnchor("en-beginner", 137, 137, 1)).toBe(137);
  });
});
