import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { ResultView } from "./result-view";
import { useExamStore, type MockAttempt } from "@/lib/exam-store";
import { scaledScore, type Exam, type ExamQuestion } from "@/lib/exams";

/** Đề mẫu tự soạn chỉ có phần đọc: bốn câu Part 5. */
const OPTIONS: ExamQuestion["options"] = [{ html: "in" }, { html: "on" }, { html: "at" }, { html: "by" }];
const q = (no: number, answer: ExamQuestion["answer"], explanation?: string): ExamQuestion => ({
  no,
  points: 1,
  answer,
  layout: 4,
  prompt: { html: `Sentence ${no} -------.` },
  options: OPTIONS,
  explanation,
});

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
      groups: [{ from: 101, to: 104, instruction: "Part 5 — Incomplete Sentences." }],
      questions: [q(101, 1), q(102, 2), q(103, 3), q(104, 4, "Giới từ by đi với hạn chót.")],
    },
  ],
};

// Đúng 3/4 câu, sai câu 104.
const attempt: MockAttempt = {
  id: "a1",
  examId: exam.id,
  startedAt: "2026-09-01T00:00:00.000Z",
  finishedAt: "2026-09-01T01:00:00.000Z",
  answers: { "reading:101": 1, "reading:102": 2, "reading:103": 3, "reading:104": 1 },
  sectionIndex: 0,
  deadline: 0,
};

beforeEach(() => useExamStore.setState({ practice: {}, attempts: [attempt] }));

describe("kết quả thi thử", () => {
  it("điểm quy đổi ước tính của phần + số câu đúng, kèm lời nói rõ không phải điểm ETS", () => {
    render(<ResultView exam={exam} attemptId="a1" />);
    const score = scaledScore("reading", 3, 4);
    expect(screen.getAllByText(String(score)).length).toBeGreaterThan(0);
    expect(screen.getByText(/đúng 3\/4 câu/)).toBeTruthy();
    expect(screen.getByText("Điểm quy đổi ước tính — không phải điểm chính thức của ETS.")).toBeTruthy();
  });

  it("đề chỉ có phần đọc: mốc là một nửa, ghi tổng tương đương", () => {
    render(<ResultView exam={exam} attemptId="a1" />);
    expect(screen.getByText("Mục tiêu 225 / 300 / 375 / 450 — tương đương tổng ~450 / 600 / 750 / 900")).toBeTruthy();
  });

  it("xem lại câu sai: nhãn (A)–(D) và lời giải thích", () => {
    render(<ResultView exam={exam} attemptId="a1" />);
    expect(screen.getByText("Giới từ by đi với hạn chót.")).toBeTruthy();
    expect(screen.getByRole("radio", { name: /\(D\) by/ })).toBeTruthy();
  });
});
