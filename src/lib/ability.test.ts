import { describe, expect, it, vi } from "vitest";
import { computeAbility, MIN_SAMPLES } from "./ability";
import type { SelfGrade } from "./activity-store";
import type { AnswerKey } from "./page-answers";
import type { GrammarEntry } from "./page-grammar";
import type { QuizSection } from "./quiz";

/**
 * Dữ liệu mẫu tự soạn — cách tính năng lực không phụ thuộc cuốn sách nào,
 * nên không bám vào nội dung sách thật (có thể đổi/xoá bất cứ lúc nào).
 */
const key = (id: string, section: string): AnswerKey => ({
  id,
  section,
  rect: [0, 0, 0, 0],
  source: 100,
  answers: [],
});

const ANSWERS: Record<string, Record<number, AnswerKey[]>> = {
  "wb-mau": {
    // Trang 10–11: bốn bài 어휘.
    10: [key("p10-1", "어휘 1"), key("p10-2", "어휘 2")],
    11: [key("p11-1", "어휘 3"), key("p11-2", "어휘 4")],
    // Trang 12: 명 이에요/예요 (bài 1, 2), trang 13: 명 은/는.
    12: [key("p12-1", "문법 · 명 이에요/예요 1"), key("p12-2", "문법 · 명 이에요/예요 2")],
    13: [key("p13-1", "문법 · 명 은/는 1")],
  },
};

const IEYO: GrammarEntry = {
  id: "g1",
  slug: "ieyo-yeyo",
  rect: [0, 0, 0, 0],
  title: "명 이에요/예요",
  vi: "Là (danh từ)",
  exKo: "",
  exVi: "",
  bookId: "sach-mau",
  page: 15,
};

const QUIZ: Record<string, Record<number, QuizSection[]>> = {
  "sach-mau": {
    19: [
      {
        title: "읽기 1",
        items: [{ id: "q1", kind: "choice", prompt: "?", options: ["가", "나"], answer: 0 }],
      },
    ],
  },
};

vi.mock("./page-answers", () => ({
  getPageAnswers: (bookId: string, page: number) => ANSWERS[bookId]?.[page] ?? [],
}));

vi.mock("./page-grammar", () => ({
  getGrammarPages: () => [],
}));

vi.mock("./quiz", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./quiz")>()),
  getPageQuiz: (bookId: string, page: number) => QUIZ[bookId]?.[page] ?? [],
}));

// Ghép sách bài tập ↔ giáo trình cần cả bộ sách thật — ở đây chỉ giả lập
// kết quả ghép, phần gom nhóm của `computeAbility` mới là thứ cần kiểm.
vi.mock("./skills", async (importOriginal) => {
  const real = await importOriginal<typeof import("./skills")>();
  return {
    ...real,
    textbookGrammarFor: (_bookId: string, _page: number, section: string) =>
      real.grammarHeading(section) === "명 이에요/예요" ? IEYO : null,
  };
});

const g = (grade: 0 | 0.5 | 1): SelfGrade => ({ grade, at: "2025-09-10T00:00:00.000Z" });

/** Khoá tự chấm của mọi chấm đáp án trên một trang. */
function keysOf(bookId: string, page: number) {
  return (ANSWERS[bookId]?.[page] ?? []).map((k) => `${bookId}:${page}:${k.id}`);
}

describe("năng lực theo kỹ năng", () => {
  it("trung bình các lượt tự chấm của đúng kỹ năng đó", () => {
    const vocab = [...keysOf("wb-mau", 10), ...keysOf("wb-mau", 11)];
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
    const ability = computeAbility(
      {},
      { "sach-mau:19": { answers: { q1: 999 }, checked: ["q1"] } }
    );

    expect(ability.skills.find((s) => s.skill === "읽기")).toMatchObject({ count: 1, score: 0 });
    expect(ability.total).toBe(1);
  });

  it("lượt chấm của chấm đáp án không còn tồn tại thì bỏ qua", () => {
    expect(computeAbility({ "wb-mau:10:khong-co": g(0) }, {}).total).toBe(0);
  });

  it(`dưới ${MIN_SAMPLES} lượt vẫn đếm, để My page biết còn thiếu bao nhiêu`, () => {
    const [first] = keysOf("wb-mau", 10);
    expect(computeAbility({ [first]: g(1) }, {}).skills[0].count).toBe(1);
  });
});

describe("điểm ngữ pháp nên ôn", () => {
  it("gom theo điểm ngữ pháp giáo trình, chỉ giữ điểm dưới 75%", () => {
    const ability = computeAbility(
      {
        "wb-mau:12:p12-1": g(0),
        "wb-mau:12:p12-2": g(0.5),
        "wb-mau:13:p13-1": g(1),
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
    const [x, y, z] = [...keysOf("wb-mau", 10), ...keysOf("wb-mau", 11)];
    const { redo } = computeAbility({ [x]: g(0.5), [y]: g(0), [z]: g(1) }, {});

    expect(redo.map((r) => r.grade)).toEqual([0, 0.5]);
  });
});
