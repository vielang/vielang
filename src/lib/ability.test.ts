import { describe, expect, it } from "vitest";
import { computeAbility, MIN_SAMPLES } from "./ability";
import type { SelfGrade } from "./activity-store";
import { getPageAnswers } from "./page-answers";
import { getPageQuiz, isGradable } from "./quiz";

const g = (grade: 0 | 0.5 | 1): SelfGrade => ({ grade, at: "2025-09-10T00:00:00.000Z" });

/** Khoá tự chấm của mọi chấm đáp án trên một trang. */
function keysOf(bookId: string, page: number) {
  return getPageAnswers(bookId, page).map((k) => `${bookId}:${page}:${k.id}`);
}

describe("năng lực theo kỹ năng", () => {
  it("trung bình các lượt tự chấm của đúng kỹ năng đó", () => {
    // wb-step1 trang 10–11: bốn bài 어휘.
    const vocab = [...keysOf("wb-step1", 10), ...keysOf("wb-step1", 11)];
    const grades = Object.fromEntries(vocab.map((k, i) => [k, g(i === 0 ? 0 : 1)]));
    const vocabSkill = computeAbility(grades, {}).skills.find((s) => s.skill === "어휘")!;

    expect(vocabSkill.count).toBe(vocab.length);
    expect(vocabSkill.score).toBeCloseTo((vocab.length - 1) / vocab.length);
  });

  it("kỹ năng chưa có lượt nào thì điểm là null, không phải 0%", () => {
    const listening = computeAbility({}, {}).skills.find((s) => s.skill === "듣기")!;
    expect(listening).toMatchObject({ count: 0, score: null });
  });

  it("gộp cả câu quiz đã kiểm tra", () => {
    const section = getPageQuiz("step1", 19).find((s) => s.title.startsWith("읽기"))!;
    const item = section.items.find(isGradable)!;
    const ability = computeAbility(
      {},
      { "step1:19": { answers: { [item.id]: 999 }, checked: [item.id] } }
    );

    expect(ability.skills.find((s) => s.skill === "읽기")).toMatchObject({ count: 1, score: 0 });
    expect(ability.total).toBe(1);
  });

  it("lượt chấm của chấm đáp án không còn tồn tại thì bỏ qua", () => {
    expect(computeAbility({ "wb-step1:10:khong-co": g(0) }, {}).total).toBe(0);
  });

  it(`dưới ${MIN_SAMPLES} lượt vẫn đếm, để My page biết còn thiếu bao nhiêu`, () => {
    const [first] = keysOf("wb-step1", 10);
    expect(computeAbility({ [first]: g(1) }, {}).skills[0].count).toBe(1);
  });
});

describe("điểm ngữ pháp nên ôn", () => {
  it("gom theo điểm ngữ pháp giáo trình, chỉ giữ điểm dưới 75%", () => {
    // wb-step1 trang 12: 명 이에요/예요 (bài 1, 2), trang 13: 명 은/는.
    const ability = computeAbility(
      {
        "wb-step1:12:p12-1": g(0),
        "wb-step1:12:p12-2": g(0.5),
        "wb-step1:13:p13-1": g(1),
      },
      {}
    );

    expect(ability.weakGrammar).toHaveLength(1);
    expect(ability.weakGrammar[0]).toMatchObject({ count: 2, score: 0.25, page: 12 });
    expect(ability.weakGrammar[0].entry?.title).toBe("명 이에요/예요");
  });
});

describe("bài cần làm lại", () => {
  it("sai nhiều đứng trước sai vài câu, bài đúng hết không có mặt", () => {
    const [x, y, z] = [...keysOf("wb-step1", 10), ...keysOf("wb-step1", 11)];
    const { redo } = computeAbility({ [x]: g(0.5), [y]: g(0), [z]: g(1) }, {});

    expect(redo.map((r) => r.grade)).toEqual([0, 0.5]);
  });
});
