import { beforeEach, describe, expect, it } from "vitest";
import { useQuizStore } from "@/lib/quiz-store";
import { noteKey } from "@/lib/note-store";
import { lessonQuizId } from "@/lib/courses";
import { summarizeQuiz } from "@/components/my/quiz-review";
import { computeAbility } from "@/lib/ability";

const BOOK = noteKey("step1", 19);
const LESSON = lessonQuizId("csharp-core", "collection-va-linq/ienumerable-va-hoan-thuc-thi");

beforeEach(() => {
  useQuizStore.setState({ pages: {} });
  localStorage.clear();
});

describe("quiz-store", () => {
  it("giữ nguyên định dạng khoá của sách", () => {
    // Bài làm của người dùng đã nằm trong máy dưới khoá này — đổi là mất hết.
    expect(BOOK).toBe("step1:19");
  });

  it("lưu và chấm theo từng quizId, không lẫn nhau", () => {
    const { setAnswer, markChecked } = useQuizStore.getState();
    setAnswer(BOOK, "p19-1", 2);
    markChecked(BOOK, "p19-1");
    setAnswer(LESSON, "q1", 1);

    const { pages } = useQuizStore.getState();
    expect(pages[BOOK]).toEqual({ answers: { "p19-1": 2 }, checked: ["p19-1"] });
    expect(pages[LESSON]).toEqual({ answers: { q1: 1 }, checked: [] });
  });

  it("sửa đáp án thì bỏ dấu đã chấm của đúng câu đó", () => {
    const { setAnswer, markChecked } = useQuizStore.getState();
    setAnswer(BOOK, "a", 0);
    setAnswer(BOOK, "b", 0);
    markChecked(BOOK, "a");
    markChecked(BOOK, "b");

    setAnswer(BOOK, "a", 1);
    expect(useQuizStore.getState().pages[BOOK].checked).toEqual(["b"]);
  });

  it("chấm hai lần cùng một câu chỉ ghi một lần", () => {
    const { markChecked } = useQuizStore.getState();
    markChecked(BOOK, "a");
    markChecked(BOOK, "a");
    expect(useQuizStore.getState().pages[BOOK].checked).toEqual(["a"]);
  });

  it("làm lại chỉ xoá bài tập đó", () => {
    const { setAnswer, resetQuiz } = useQuizStore.getState();
    setAnswer(BOOK, "a", 0);
    setAnswer(LESSON, "q1", 0);

    resetQuiz(LESSON);
    expect(useQuizStore.getState().pages[BOOK]).toBeDefined();
    expect(useQuizStore.getState().pages[LESSON]).toBeUndefined();

    // Bài tập chưa từng làm: không tạo rác, không ném lỗi.
    const before = useQuizStore.getState().pages;
    resetQuiz("khong-co-that");
    expect(useQuizStore.getState().pages).toBe(before);
  });
});

describe("khoá bài học không đụng khoá sách", () => {
  it("có tiền tố it: và dấu / nên không trùng dạng bookId:page", () => {
    expect(LESSON).toBe("it:csharp-core/collection-va-linq/ienumerable-va-hoan-thuc-thi");
    expect(LESSON.startsWith("it:")).toBe(true);
  });

  it("màn hình tổng kết của sách bỏ qua bài làm của bài học IT", () => {
    const pages = {
      [LESSON]: { answers: { q1: 0 }, checked: ["q1"] },
    };
    // Không đếm nhầm, không ném lỗi vì "page" của bài học không phải số.
    expect(summarizeQuiz(pages)).toEqual({ checked: 0, correct: 0, wrong: [] });
    expect(computeAbility({}, pages).total).toBe(0);
  });
});
