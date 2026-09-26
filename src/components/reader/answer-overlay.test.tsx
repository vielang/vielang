import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { AnswerOverlay } from "./answer-overlay";
import type { AnswerKey } from "@/lib/page-answers";
import { useActivityStore } from "@/lib/activity-store";

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
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);

    expect(screen.getByLabelText("Xem đáp án: 읽기")).toBeTruthy();
  });

  it("vùng chạm đủ to cho ngón tay", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);

    expect(trigger().className).toMatch(/\bsize-11\b/);
  });

  it("gắn dấu để page-viewer không tính cú chạm là tap ẩn thanh công cụ", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);

    expect(trigger().hasAttribute("data-answer-key")).toBe(true);
  });

  it("khung định vị không nhận chạm", () => {
    const { container } = render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    const frame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.style.width === "3%"
    );

    expect(frame).toBeDefined();
    expect(frame!.className).toContain("pointer-events-none");
  });

  it("trang không có đáp án thì không vẽ gì", () => {
    const { container } = render(<AnswerOverlay answerKeys={[]} bookId="step1" page={19} />);

    expect(container.innerHTML).toBe("");
  });
});

describe("bong bóng đáp án", () => {
  it("chưa bấm thì chưa lộ đáp án", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.body.textContent).not.toContain("아니요 ✓");
  });

  it("bấm thì hiện đủ mọi câu, theo đúng thứ tự", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    const text = screen.getByRole("dialog").textContent!;

    expect(text).toContain("읽기");
    const at = ["1)", "2)", "3)"].map((label) => text.indexOf(label));
    expect(at.every((i) => i >= 0)).toBe(true);
    expect(at).toEqual([...at].sort((a, b) => a - b));
    expect(text).toContain("네 ✓");
  });

  it("ghi rõ là đáp án của sách, kèm trang để tự đối chiếu", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());

    expect(screen.getByRole("dialog").textContent).toContain("tr.212");
  });

  it("danh sách đáp án dài thì cuộn bên trong, không tràn khỏi màn hình", () => {
    // Bảng chia động từ của sách bài tập có bài 15–18 dòng; điện thoại cầm
    // ngang chỉ cao ~390px.
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    const list = screen.getByText("네 ✓").parentElement!.parentElement!;

    expect(list.className).toMatch(/max-h-/);
    expect(list.className).toMatch(/\boverflow-y-auto\b/);
  });

  it("bấm lại chính chấm đó thì đóng", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    tap(trigger());

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("bấm ra ngoài hoặc Esc thì đóng", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    fireEvent.pointerDown(document.body, { bubbles: true });
    expect(screen.queryByRole("dialog")).toBeNull();

    tap(trigger());
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("tự chấm trong bong bóng đáp án", () => {
  beforeEach(() => useActivityStore.setState({ grades: {} }));

  it("chọn một mức thì lưu lại, và bong bóng vẫn mở", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    tap(screen.getByRole("button", { name: "Sai vài câu" }));

    expect(useActivityStore.getState().grades["step1:19:p19-read"]?.grade).toBe(0.5);
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Sai vài câu" }).getAttribute("aria-pressed")
    ).toBe("true");
  });

  it("chấm lại mức khác thì đè lần trước", () => {
    render(<AnswerOverlay answerKeys={KEYS} bookId="step1" page={19} />);
    tap(trigger());
    tap(screen.getByRole("button", { name: "Sai nhiều" }));
    tap(screen.getByRole("button", { name: "Đúng hết" }));

    expect(useActivityStore.getState().grades["step1:19:p19-read"]?.grade).toBe(1);
  });
});
