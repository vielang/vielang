import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { MyPageView } from "./my-page-view";
import { BOOKS } from "@/lib/books";
import { dayKey, EMPTY_DAY } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { useProgressStore } from "@/lib/progress-store";
import { useQuizStore } from "@/lib/quiz-store";
import { courseCard, courseProgressId, lessonHref, listCourses } from "@/lib/courses";
import type { AnswerKey } from "@/lib/page-answers";
import type { GrammarEntry } from "@/lib/page-grammar";
import type { ChoiceItem } from "@/lib/quiz";

/**
 * Dữ liệu mẫu tự soạn cho phần gợi ý/năng lực: một bài ngữ pháp có đáp án ở
 * trang 12 và một câu quiz ở trang 19 của en-elementary — không bám vào nội
 * dung sách thật.
 */
const QUIZ_ITEM: ChoiceItem = {
  id: "q1",
  kind: "choice",
  prompt: "Choose the correct word",
  options: ["am", "is"],
  answer: 0,
};

const GRAMMAR_ENTRY: GrammarEntry = {
  id: "g1",
  slug: "present-simple",
  rect: [0, 0, 0, 0],
  title: "Present simple",
  vi: "Thì hiện tại đơn",
  exKo: "",
  exVi: "",
  bookId: "en-elementary",
  page: 15,
};

vi.mock("@/lib/page-answers", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/page-answers")>()),
  getPageAnswers: (bookId: string, page: number): AnswerKey[] =>
    bookId === "en-elementary" && page === 12
      ? [{ id: "p12-1", rect: [0, 0, 0, 0], section: "문법 · Present simple 1", source: 120, answers: [] }]
      : [],
}));

// Ghép sách bài tập ↔ giáo trình cần cả bộ sách thật — ở đây chỉ giả lập kết quả ghép.
vi.mock("@/lib/skills", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/skills")>()),
  textbookGrammarFor: () => GRAMMAR_ENTRY,
}));

vi.mock("@/lib/quiz", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/quiz")>()),
  getPageQuiz: (bookId: string, page: number) =>
    bookId === "en-elementary" && page === 19 ? [{ title: "Grammar", items: [QUIZ_ITEM] }] : [],
}));

const studiedToday = (minutes: number) => ({
  [dayKey(new Date())]: { ...EMPTY_DAY, activeMs: minutes * 60_000 },
});

beforeEach(() => {
  localStorage.clear();
  useActivityStore.setState({ days: {}, studied: {}, weeklyGoalMinutes: 60, grades: {} });
  useProgressStore.setState({ books: {}, hasHydrated: true });
  useQuizStore.setState({ pages: {} });
});

describe("lần đầu vào", () => {
  it("mời bắt đầu, không vẽ một trang toàn số 0", () => {
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/Bắt đầu hành trình/)).toBeTruthy();
    expect(screen.getByRole("link", { name: /Chọn khoá học/ })).toBeTruthy();
  });

  it("chưa có gì để lưu thì không mời lưu ra file, chỉ mời nhập", () => {
    render(<MyPageView books={BOOKS} />);

    expect(screen.queryByRole("button", { name: /Lưu ra file/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Nhập từ file/ })).toBeTruthy();
  });
});

describe("lời nhắn đồng hành và việc làm ngay", () => {
  it("ghi nhận hôm nay và mời học tiếp đúng trang đang dở", () => {
    useActivityStore.setState({ days: studiedToday(30), studied: { "en-elementary": [18, 19] } });
    useProgressStore.setState({
      books: {
        "en-elementary": { lastPage: 19, readPages: [17, 18, 19], bookmarks: [], updatedAt: "2025-01-01" },
      },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText("Hôm nay bạn đã học 30 phút")).toBeTruthy();
    const cta = screen.getByRole("link", { name: /Học tiếp English File – Elementary · trang 19/ });
    expect(cta.getAttribute("href")).toBe("/read/en-elementary/19");
  });

  it("tuần này: số phút so với mục tiêu, đổi được mục tiêu", () => {
    useActivityStore.setState({ days: studiedToday(30) });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/\/ 60 phút/).parentElement?.textContent).toContain("30");
    fireEvent.click(screen.getByRole("button", { name: "Đổi mục tiêu" }));
    fireEvent.click(screen.getByRole("radio", { name: "150 phút" }));

    expect(useActivityStore.getState().weeklyGoalMinutes).toBe(150);
  });
});

describe("sách đang học", () => {
  it("tên ngắn gọn và số trang đã học", () => {
    useActivityStore.setState({ studied: { "en-elementary": [18, 19] } });
    useProgressStore.setState({
      books: { "en-elementary": { lastPage: 19, readPages: [19], bookmarks: [], updatedAt: "" } },
    });
    render(<MyPageView books={BOOKS} />);

    const book = BOOKS.find((b) => b.id === "en-elementary")!;
    expect(screen.getByText(`2/${book.totalPages} trang`)).toBeTruthy();
  });
});

describe("khoá học đang học", () => {
  const course = courseCard(listCourses()[0]);

  it("đã học vài bài thì hiện tiến độ và mời học tiếp bài chưa học đầu tiên", () => {
    useProgressStore.setState({
      books: { [courseProgressId(course.id)]: { lastPage: 1, readPages: [1], bookmarks: [], updatedAt: "" } },
    });
    render(<MyPageView books={BOOKS} courses={[course]} />);

    expect(screen.getByText("Khoá học đang học")).toBeTruthy();
    expect(screen.getByText(`Đã học 1/${course.total} bài`)).toBeTruthy();
    const next = course.lessons[1];
    const link = screen.getByRole("link", { name: `Học tiếp: ${next.title}` });
    expect(link.getAttribute("href")).toBe(lessonHref(course.id, next.slug));
  });

  it("chưa học bài nào thì không hiện khoá đó", () => {
    useActivityStore.setState({ days: studiedToday(5) });
    render(<MyPageView books={BOOKS} courses={[course]} />);

    expect(screen.queryByText("Khoá học đang học")).toBeNull();
  });
});

describe("luyện thi", () => {
  it("chưa có kỳ thi nào thì không hiện mục luyện thi", () => {
    useActivityStore.setState({ days: studiedToday(5) });
    render(<MyPageView books={BOOKS} />);

    expect(screen.queryByRole("heading", { name: "Luyện thi" })).toBeNull();
  });
});

describe("gợi ý và năng lực", () => {
  it("chưa tự chấm gì thì chỉ cách để có gợi ý", () => {
    useActivityStore.setState({ days: studiedToday(5) });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/mình sẽ gợi ý bài nào nên ôn lại/)).toBeTruthy();
    expect(screen.queryByText("Kỹ năng của bạn")).toBeNull();
  });

  it("có lượt tự chấm sai thì gợi ý ôn đúng điểm ngữ pháp, dẫn tới trang giải thích", () => {
    useActivityStore.setState({ grades: { "en-elementary:12:p12-1": { grade: 0, at: "" } } });
    render(<MyPageView books={BOOKS} />);

    const tip = screen.getByRole("link", { name: /Ôn lại ngữ pháp Present simple/ });
    expect(tip.getAttribute("href")).toBe("/read/en-elementary/15");
    expect(screen.getByText("Kỹ năng của bạn")).toBeTruthy();
    expect(screen.getByText(/không phải điểm thi/)).toBeTruthy();
  });

  it("câu quiz còn sai có trong gợi ý và trong danh sách chi tiết", () => {
    useProgressStore.setState({
      books: { "en-elementary": { lastPage: 19, readPages: [19], bookmarks: [], updatedAt: "" } },
    });
    useQuizStore.setState({
      pages: { "en-elementary:19": { answers: { [QUIZ_ITEM.id]: 999 }, checked: [QUIZ_ITEM.id] } },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByRole("link", { name: /Làm lại câu quiz còn sai/ })).toBeTruthy();
    const detail = screen.getByText(QUIZ_ITEM.prompt).closest("a");
    expect(detail?.getAttribute("href")).toBe("/read/en-elementary/19");
  });
});
