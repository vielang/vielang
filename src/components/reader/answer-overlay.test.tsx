import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { AnswerOverlay } from "./answer-overlay";
import type { AnswerKey } from "@/lib/page-answers";
import { getPageAnswers } from "@/lib/page-answers";

const KEYS: AnswerKey[] = [
  {
    id: "p19-read",
    rect: [0.87, 0.517, 0.03, 0.024],
    section: "읽기",
    source: 212,
    answers: [
      { label: "1)", text: "네 ✓" },
      { label: "2)", text: "아니요 ✓" },
      { label: "3)", text: "아니요 ✓" },
    ],
  },
];

const trigger = () => screen.getByLabelText(/đáp án: 읽기/);

/** Xem chú thích cùng tên ở `grammar-overlay.test.tsx`. */
function tap(el: HTMLElement) {
  fireEvent.pointerDown(el, { bubbles: true });
  fireEvent.click(el);
}

describe("chấm đáp án", () => {
  it("mời xem đáp án, kèm tên mục", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);

    expect(screen.getByLabelText("Xem đáp án: 읽기")).toBeTruthy();
  });

  it("vùng chạm đủ to cho ngón tay", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);

    expect(trigger().className).toMatch(/\bsize-11\b/);
  });

  it("gắn dấu để page-viewer không tính cú chạm là tap ẩn thanh công cụ", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);

    expect(trigger().hasAttribute("data-answer-key")).toBe(true);
  });

  it("khung định vị không nhận chạm", () => {
    const { container } = render(<AnswerOverlay answerKeys={KEYS} />);
    const frame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.style.width === "3%"
    );

    expect(frame).toBeDefined();
    expect(frame!.className).toContain("pointer-events-none");
  });

  it("trang không có đáp án thì không vẽ gì", () => {
    const { container } = render(<AnswerOverlay answerKeys={[]} />);

    expect(container.innerHTML).toBe("");
  });
});

describe("bong bóng đáp án", () => {
  it("chưa bấm thì chưa lộ đáp án", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);

    expect(screen.queryByRole("tooltip")).toBeNull();
    expect(document.body.textContent).not.toContain("아니요 ✓");
  });

  it("bấm thì hiện đủ mọi câu, theo đúng thứ tự", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);
    tap(trigger());
    const text = screen.getByRole("tooltip").textContent!;

    expect(text).toContain("읽기");
    const at = ["1)", "2)", "3)"].map((label) => text.indexOf(label));
    expect(at.every((i) => i >= 0)).toBe(true);
    expect(at).toEqual([...at].sort((a, b) => a - b));
    expect(text).toContain("네 ✓");
  });

  it("ghi rõ là đáp án của sách, kèm trang để tự đối chiếu", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);
    tap(trigger());

    expect(screen.getByRole("tooltip").textContent).toContain("tr.212");
  });

  it("bấm lại chính chấm đó thì đóng", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);
    tap(trigger());
    tap(trigger());

    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("bấm ra ngoài hoặc Esc thì đóng", () => {
    render(<AnswerOverlay answerKeys={KEYS} />);
    tap(trigger());
    fireEvent.pointerDown(document.body, { bubbles: true });
    expect(screen.queryByRole("tooltip")).toBeNull();

    tap(trigger());
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});

describe("dữ liệu đáp án Sơ cấp 1, bài 1", () => {
  // Chép từ bảng 모범 답안 trang 212. Test này giữ cho dữ liệu khỏi trôi
  // khỏi sách khi có người sửa tay file JSON.
  it("trang 18 — 듣기", () => {
    const [key] = getPageAnswers("step1", 18);

    expect(key.section).toBe("듣기");
    expect(key.source).toBe(212);
    expect(key.answers).toEqual([
      { label: "1)", text: "네 ✓" },
      { label: "2)", text: "②" },
    ]);
  });

  it("trang 19 — 읽기", () => {
    const [key] = getPageAnswers("step1", 19);

    expect(key.section).toBe("읽기");
    expect(key.answers.map((a) => a.text)).toEqual(["네 ✓", "아니요 ✓", "아니요 ✓"]);
  });

  it("khớp với đáp án của bài tập tự làm trong quiz", async () => {
    // Hai nguồn cùng chép từ một bảng đáp án — lệch nhau là một bên chép sai.
    const { getPageQuiz } = await import("@/lib/quiz");
    const quiz19 = getPageQuiz("step1", 19).flatMap((s) => s.items);
    const picked = quiz19
      .filter((i) => i.kind === "choice")
      .map((i) => (i.kind === "choice" ? i.options[i.answer] : ""));

    const [key] = getPageAnswers("step1", 19);

    expect(picked).toEqual(key.answers.map((a) => a.text.replace(/\s*✓$/, "")));
  });
});
