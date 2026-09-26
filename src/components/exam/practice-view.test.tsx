import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { PracticeView } from "./practice-view";
import { useExamStore } from "@/lib/exam-store";
import type { Exam, ExamQuestion } from "@/lib/exams";

/** Đề mẫu tự soạn: một câu Part 5 có giải thích, một khối Part 6 hai chỗ trống. */
const OPTIONS: ExamQuestion["options"] = [
  { html: "review" },
  { html: "reviewing" },
  { html: "reviewed" },
  { html: "to review" },
];

const exam: Exam = {
  id: "toeic-mau",
  round: 1,
  year: 2026,
  level: "TOEIC",
  title: "Đề mẫu",
  assetDir: "mau",
  source: "Đề mẫu tự soạn",
  sections: [
    {
      id: "reading",
      title: "Reading",
      minutes: 75,
      groups: [
        { from: 101, to: 101, instruction: "Part 5 — Incomplete Sentences." },
        {
          from: 131,
          to: 132,
          instruction: "Part 6 — Questions 131-132 refer to the following memo.",
          passage: '<div class="exam-box"><b>MEMO</b><br>The lobby <b>(131)</b> ------- on Monday. <b>(132)</b> -------</div>',
        },
      ],
      questions: [
        {
          no: 101,
          points: 1,
          answer: 1,
          layout: 2,
          prompt: { html: "The manager will ------- the report." },
          options: OPTIONS,
          explanation: "Sau will cần động từ nguyên mẫu.",
        },
        { no: 131, points: 1, answer: 2, layout: 2, prompt: { html: "" }, options: OPTIONS },
        { no: 132, points: 1, answer: 3, layout: 1, prompt: { html: "" }, options: OPTIONS },
      ],
    },
  ],
};

beforeEach(() => {
  // jsdom không cuộn trang; mở thẳng một câu (`initialNo`) thì màn luyện gọi scrollTo.
  window.scrollTo = () => {};
  localStorage.clear();
  useExamStore.setState({ practice: {}, attempts: [] });
});

describe("luyện từng câu", () => {
  it("lựa chọn mang nhãn (A)–(D) như đề in", () => {
    render(<PracticeView exam={exam} sectionId="reading" />);
    for (const [label, text] of [
      ["(A)", "review"],
      ["(B)", "reviewing"],
      ["(C)", "reviewed"],
      ["(D)", "to review"],
    ]) {
      expect(screen.getByRole("radio", { name: `${label} ${text}` })).toBeTruthy();
    }
  });

  it("giải thích chỉ hiện SAU khi kiểm tra", () => {
    render(<PracticeView exam={exam} sectionId="reading" />);
    expect(screen.queryByText("Sau will cần động từ nguyên mẫu.")).toBeNull();

    fireEvent.click(screen.getByRole("radio", { name: "(C) reviewed" }));
    fireEvent.click(screen.getByRole("button", { name: /Kiểm tra/ }));

    expect(screen.getByText("Sau will cần động từ nguyên mẫu.")).toBeTruthy();
    expect(screen.getByText(/đáp án là \(A\)/)).toBeTruthy();
  });

  it("Part 6: văn bản có chỗ trống hiện trên các câu của khối, đề câu rỗng không hiện", () => {
    const { container } = render(<PracticeView exam={exam} sectionId="reading" initialNo={131} />);
    expect(screen.getByText("MEMO")).toBeTruthy();
    expect(screen.getByText("Part 6")).toBeTruthy();
    expect(container.querySelector("#q-131")).toBeTruthy();
    expect(container.querySelector("#q-132")).toBeTruthy();
    // Văn bản đứng TRƯỚC câu đầu tiên của khối.
    const passage = screen.getByText("MEMO");
    expect(passage.compareDocumentPosition(container.querySelector("#q-131")!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});
