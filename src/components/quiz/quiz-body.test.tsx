import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { QuizBody } from "./quiz-body";
import { QuizPanel } from "@/components/reader/quiz-panel";
import { useQuizStore } from "@/lib/quiz-store";
import { useActivityStore } from "@/lib/activity-store";
import { dayKey } from "@/lib/activity";
import { getPageQuiz } from "@/lib/quiz";
import { getLesson, lessonQuizId, listCourses } from "@/lib/courses";
import type { QuizSection } from "@/lib/quiz";

const SECTIONS: QuizSection[] = [
  {
    title: "연습 1",
    items: [
      {
        id: "c1",
        kind: "choice",
        prompt: "1 + 1 = ?",
        options: ["1", "2", "3"],
        answer: 1,
        explain: "Cộng hai số.",
      },
      { id: "f1", kind: "fill", prompt: "Thủ đô Hàn Quốc?", answers: ["서울"] },
      { id: "o1", kind: "free", prompt: "Bạn nghĩ sao?" },
    ],
  },
];

beforeEach(() => {
  localStorage.clear();
  useQuizStore.setState({ pages: {} });
  useActivityStore.setState({ days: {}, studied: {}, weeklyGoalMinutes: 60, grades: {} });
});

describe("chấm bài", () => {
  it("đếm câu chấm được, bỏ qua câu tự luận", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);
    expect(screen.getByText("0/2 câu")).toBeTruthy();
  });

  it("chọn đúng thì báo Đúng và hiện giải thích", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.click(screen.getByRole("button", { name: "2" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);

    expect(screen.getByText("Đúng")).toBeTruthy();
    expect(screen.getByText("Cộng hai số.")).toBeTruthy();
    expect(screen.getByText("1/2 câu · đúng 1")).toBeTruthy();
  });

  it("chọn sai thì báo Chưa đúng", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.click(screen.getByRole("button", { name: "3" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);

    expect(screen.getByText(/Chưa đúng/)).toBeTruthy();
    expect(screen.getByText("1/2 câu · đúng 0")).toBeTruthy();
  });

  it("câu điền sai thì hiện đáp án đúng", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.change(screen.getByLabelText("Thủ đô Hàn Quốc?"), { target: { value: "부산" } });
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[1]);

    expect(screen.getByText(/Chưa đúng — 서울/)).toBeTruthy();
  });

  it("chưa trả lời thì không bấm kiểm tra được", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);
    const check = screen.getAllByRole("button", { name: "Kiểm tra" })[0] as HTMLButtonElement;
    expect(check.disabled).toBe(true);
  });

  it("ghi lượt chấm vào lịch sử học", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.click(screen.getByRole("button", { name: "2" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);

    const today = useActivityStore.getState().days[dayKey(new Date())];
    expect(today).toMatchObject({ quizChecked: 1, quizCorrect: 1 });
  });

  it("sửa đáp án sau khi chấm thì bỏ kết quả cũ", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.click(screen.getByRole("button", { name: "2" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);
    expect(screen.getByText("Đúng")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "3" }));
    expect(screen.queryByText("Đúng")).toBeNull();
    expect(screen.getByText("0/2 câu")).toBeTruthy();
  });

  it("làm lại từ đầu xoá sạch bài làm của chính bài tập đó", () => {
    useQuizStore.setState({ pages: { "khac:1": { answers: { x: 1 }, checked: ["x"] } } });
    render(<QuizBody quizId="t:1" sections={SECTIONS} />);

    fireEvent.click(screen.getByRole("button", { name: "2" }));
    fireEvent.click(screen.getByRole("button", { name: "Làm lại từ đầu" }));

    expect(useQuizStore.getState().pages["t:1"]).toBeUndefined();
    expect(useQuizStore.getState().pages["khac:1"]).toBeDefined();
  });
});

describe("kiểu bài học: đánh số câu và nhãn A/B/C/D", () => {
  const HAI_MUC: QuizSection[] = [
    { title: "Mục 1", items: [SECTIONS[0].items[0]] },
    {
      title: "Mục 2",
      items: [
        { id: "c2", kind: "choice", prompt: "2 + 2 = ?", options: ["3", "4"], answer: 1 },
      ],
    },
  ];

  it("đánh số chạy xuyên các mục, không đếm lại từ đầu mỗi mục", () => {
    render(<QuizBody quizId="t:1" sections={HAI_MUC} numbered />);

    expect(screen.getByText("Câu 1")).toBeTruthy();
    expect(screen.getByText("Câu 2")).toBeTruthy();
  });

  it("mỗi phương án có nhãn chữ cái theo thứ tự", () => {
    render(<QuizBody quizId="t:1" sections={SECTIONS} numbered />);

    const nhan = screen.getAllByRole("button", { pressed: false }).slice(0, 3);
    expect(nhan.map((b) => b.textContent)).toEqual(["A1", "B2", "C3"]);
  });

  it("không có số thì không đánh số — bài tập trong sách giữ nguyên", () => {
    render(<QuizBody quizId="t:2" sections={SECTIONS} />);

    expect(screen.queryByText("Câu 1")).toBeNull();
    expect(screen.getByText("연습 1")).toBeTruthy();
  });

  it("làm hết thì hiện dòng tổng kết", () => {
    render(<QuizBody quizId="t:1" sections={HAI_MUC} numbered />);

    // Nhãn chữ cái là aria-hidden nên tên khả truy cập chỉ còn nội dung phương án.
    fireEvent.click(screen.getByRole("button", { name: "2" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);
    // Câu 1 cũng có phương án "3"; cái thứ hai mới là của câu 2.
    fireEvent.click(screen.getAllByRole("button", { name: "3" })[1]);
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);

    expect(screen.getByText(/Xong rồi/)).toBeTruthy();
    expect(screen.getByText("đúng 1/2")).toBeTruthy();
  });
});

describe("bài tập trong sách vẫn chạy như cũ", () => {
  const sections = getPageQuiz("step1", 19);

  it("trang sách thật có bài tập để làm", () => {
    expect(sections.length).toBeGreaterThan(0);
  });

  it("QuizPanel lưu dưới khoá bookId:page, đúng định dạng đã có", () => {
    render(<QuizPanel bookId="step1" page={19} sections={sections} />);

    const first = sections.flatMap((s) => s.items).find((i) => i.kind === "choice");
    if (!first || first.kind !== "choice") throw new Error("trang 19 không có câu trắc nghiệm");

    // Trang sách có nhiều câu trùng phương án — lấy câu đầu tiên.
    fireEvent.click(screen.getAllByRole("button", { name: first.options[0] })[0]);

    expect(useQuizStore.getState().pages["step1:19"]?.answers[first.id]).toBe(0);
  });

  it("đọc được bài làm đã lưu từ phiên trước", () => {
    const first = sections.flatMap((s) => s.items).find((i) => i.kind === "choice");
    if (!first || first.kind !== "choice") throw new Error("trang 19 không có câu trắc nghiệm");

    // Đúng hình dạng mà bản trước đã ghi vào localStorage.
    useQuizStore.setState({
      pages: { "step1:19": { answers: { [first.id]: first.answer }, checked: [first.id] } },
    });
    render(<QuizPanel bookId="step1" page={19} sections={sections} />);

    expect(screen.getByText("Đúng")).toBeTruthy();
  });
});

describe("câu tự kiểm tra của bài học IT", () => {
  const course = listCourses()[0];
  const lesson = course.modules
    .flatMap((m) => m.lessons)
    .find((l) => l.quiz !== undefined);

  it("có ít nhất một bài học kèm câu hỏi", () => {
    expect(lesson).toBeDefined();
    expect(getLesson(course, lesson!.slug)?.quiz?.[0].items.length).toBeGreaterThan(0);
  });

  it("chấm và lưu dưới khoá riêng của bài học", () => {
    const quizId = lessonQuizId(course.id, lesson!.slug);
    render(<QuizBody quizId={quizId} sections={lesson!.quiz!} />);

    const item = lesson!.quiz![0].items[0];
    if (item.kind !== "choice") throw new Error("câu đầu phải là trắc nghiệm");

    fireEvent.click(screen.getByRole("button", { name: item.options[item.answer] }));
    fireEvent.click(screen.getAllByRole("button", { name: "Kiểm tra" })[0]);

    expect(screen.getByText("Đúng")).toBeTruthy();
    expect(useQuizStore.getState().pages[quizId]?.checked).toEqual([item.id]);
    // Không đụng tới vùng dữ liệu của sách.
    expect(Object.keys(useQuizStore.getState().pages)).toEqual([quizId]);
  });
});
