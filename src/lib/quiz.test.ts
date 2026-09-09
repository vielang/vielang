import { describe, expect, it } from "vitest";
import { isCorrect, normalizeAnswer, type FillItem, type ChoiceItem } from "./quiz";

function fill(...answers: string[]): FillItem {
  return { id: "q", kind: "fill", prompt: "", answers };
}

describe("normalizeAnswer", () => {
  it("bỏ qua khoảng trắng thiếu, thừa và đặt sai chỗ", () => {
    expect(normalizeAnswer("베트남 사람이에요")).toBe(
      normalizeAnswer("베트남사람이에요")
    );
    expect(normalizeAnswer("안 해요")).toBe(normalizeAnswer("안해요"));
    expect(normalizeAnswer("  네,  있어요  ")).toBe(normalizeAnswer("네 있어요"));
  });

  it("bỏ qua dấu câu", () => {
    expect(normalizeAnswer("네, 소파가 없어요.")).toBe(
      normalizeAnswer("네 소파가 없어요")
    );
    expect(normalizeAnswer("빠릅니까?")).toBe(normalizeAnswer("빠릅니까"));
    expect(normalizeAnswer("02-2711-5348")).toBe(normalizeAnswer("02)2711 5348"));
  });

  it("chuẩn hoá NFD -> NFC (bàn phím tiếng Hàn trên macOS)", () => {
    const nfc = "한국";
    const nfd = nfc.normalize("NFD");
    expect(nfd).not.toBe(nfc); // khác byte
    expect(normalizeAnswer(nfd)).toBe(normalizeAnswer(nfc));
  });
});

describe("isCorrect — fill", () => {
  it("chấp nhận đáp án đúng dù thiếu/thừa khoảng trắng và dấu chấm", () => {
    const item = fill("베트남 사람이에요");
    expect(isCorrect(item, "베트남 사람이에요")).toBe(true);
    expect(isCorrect(item, "베트남사람이에요")).toBe(true);
    expect(isCorrect(item, " 베트남 사람이에요. ")).toBe(true);
  });

  it("chấp nhận bất kỳ đáp án nào trong danh sách", () => {
    const item = fill("두 시 삼십 분", "두 시 반");
    expect(isCorrect(item, "두시반")).toBe(true);
    expect(isCorrect(item, "두 시 삼십 분")).toBe(true);
  });

  it("vẫn báo sai khi sai chính tả tiếng Hàn", () => {
    const item = fill("안젤라예요");
    expect(isCorrect(item, "안젤라이에요")).toBe(false);
    expect(isCorrect(item, "안젤라")).toBe(false);
  });
});

describe("isCorrect — choice và free", () => {
  it("choice so theo chỉ số", () => {
    const item: ChoiceItem = {
      id: "q",
      kind: "choice",
      prompt: "",
      options: ["네", "아니요"],
      answer: 1,
    };
    expect(isCorrect(item, 1)).toBe(true);
    expect(isCorrect(item, 0)).toBe(false);
  });

  it("free không bao giờ được chấm là đúng", () => {
    expect(isCorrect({ id: "q", kind: "free", prompt: "" }, "bất kỳ")).toBe(false);
  });
});
