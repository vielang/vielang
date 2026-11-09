import { beforeEach, describe, expect, it } from "vitest";
import { useQuizStore } from "@/lib/quiz-store";
import { lessonQuizId } from "@/lib/courses";

const OTHER = lessonQuizId("csharp-core", "khac/bai-khac");
const LESSON = lessonQuizId("csharp-core", "collection-va-linq/ienumerable-va-hoan-thuc-thi");

beforeEach(() => {
  useQuizStore.setState({ pages: {} });
  localStorage.clear();
});

describe("quiz-store", () => {
  it("lưu và chấm theo từng quizId, không lẫn nhau", () => {
    const { setAnswer, markChecked } = useQuizStore.getState();
    setAnswer(OTHER, "p19-1", 2);
    markChecked(OTHER, "p19-1");
    setAnswer(LESSON, "q1", 1);

    const { pages } = useQuizStore.getState();
    expect(pages[OTHER]).toEqual({ answers: { "p19-1": 2 }, checked: ["p19-1"] });
    expect(pages[LESSON]).toEqual({ answers: { q1: 1 }, checked: [] });
  });

  it("sửa đáp án thì bỏ dấu đã chấm của đúng câu đó", () => {
    const { setAnswer, markChecked } = useQuizStore.getState();
    setAnswer(OTHER, "a", 0);
    setAnswer(OTHER, "b", 0);
    markChecked(OTHER, "a");
    markChecked(OTHER, "b");

    setAnswer(OTHER, "a", 1);
    expect(useQuizStore.getState().pages[OTHER].checked).toEqual(["b"]);
  });

  it("chấm hai lần cùng một câu chỉ ghi một lần", () => {
    const { markChecked } = useQuizStore.getState();
    markChecked(OTHER, "a");
    markChecked(OTHER, "a");
    expect(useQuizStore.getState().pages[OTHER].checked).toEqual(["a"]);
  });

  it("làm lại chỉ xoá bài tập đó", () => {
    const { setAnswer, resetQuiz } = useQuizStore.getState();
    setAnswer(OTHER, "a", 0);
    setAnswer(LESSON, "q1", 0);

    resetQuiz(LESSON);
    expect(useQuizStore.getState().pages[OTHER]).toBeDefined();
    expect(useQuizStore.getState().pages[LESSON]).toBeUndefined();

    // Bài tập chưa từng làm: không tạo rác, không ném lỗi.
    const before = useQuizStore.getState().pages;
    resetQuiz("khong-co-that");
    expect(useQuizStore.getState().pages).toBe(before);
  });
});

describe("khoá bài học", () => {
  it("có tiền tố it: và dấu / — định dạng đã nằm trong máy người dùng", () => {
    expect(LESSON).toBe("it:csharp-core/collection-va-linq/ienumerable-va-hoan-thuc-thi");
  });
});
