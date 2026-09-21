import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { GrammarOverlay } from "./grammar-overlay";
import type { GrammarPoint } from "@/lib/page-grammar";

const POINTS: GrammarPoint[] = [
  {
    id: "p15-ieyo-yeyo",
    slug: "ieyo-yeyo",
    rect: [0.63, 0.028, 0.33, 0.048],
    title: "명 이에요/예요",
    ko: "사람, 사물 이름을 말할 때 사용해요.",
    vi: "Dùng khi nói tên người hoặc tên đồ vật.",
    html: "<h2>Cách dùng</h2><table><tr><td>후엔</td></tr></table>",
  },
];

const openButton = () => screen.getByLabelText(/Xem giải thích ngữ pháp/);

describe("chấm ngữ pháp", () => {
  it("mời người dùng xem giải thích, kèm tên điểm ngữ pháp", () => {
    render(<GrammarOverlay points={POINTS} />);

    expect(
      screen.getByLabelText("Xem giải thích ngữ pháp: 명 이에요/예요")
    ).toBeTruthy();
  });

  it("vùng chạm đủ to cho ngón tay", () => {
    // size-11 = 44px. Giống chấm dịch: nét vẽ vẫn 20px, phần dôi ra là lề
    // vô hình — to cái chấm lên thì chọc vào dáng bản in của trang.
    render(<GrammarOverlay points={POINTS} />);

    expect(openButton().className).toMatch(/\bsize-11\b/);
  });

  it("chấm nằm BÊN TRÁI tiêu đề", () => {
    // Tiêu đề ngữ pháp luôn sát mép phải trang, để chấm bên phải là rơi ra
    // ngoài giấy.
    render(<GrammarOverlay points={POINTS} />);

    expect(openButton().className).toMatch(/\bright-full\b/);
  });

  it("khung tiêu đề không nhận chạm, chỉ để định vị", () => {
    const { container } = render(<GrammarOverlay points={POINTS} />);
    const frame = Array.from(container.querySelectorAll("div")).find(
      (el) => el.style.width === "33%"
    );

    expect(frame).toBeDefined();
    expect(frame!.className).toContain("pointer-events-none");
  });

  it("trang không có ngữ pháp thì không vẽ gì", () => {
    const { container } = render(<GrammarOverlay points={[]} />);

    expect(container.innerHTML).toBe("");
  });
});

describe("tấm phủ giải thích", () => {
  it("hiện tiêu đề, nghĩa ngắn và phần giải thích khi bấm", () => {
    render(<GrammarOverlay points={POINTS} />);
    fireEvent.click(openButton());

    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(screen.getByText("Dùng khi nói tên người hoặc tên đồ vật.")).toBeTruthy();
    expect(screen.getByText("Cách dùng")).toBeTruthy();
  });

  it("chưa bấm thì chưa hiện gì", () => {
    render(<GrammarOverlay points={POINTS} />);

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("đóng được bằng nút X", () => {
    render(<GrammarOverlay points={POINTS} />);
    fireEvent.click(openButton());
    fireEvent.click(screen.getByLabelText("Đóng"));

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("đóng được bằng cách bấm ra nền mờ", () => {
    // Đường thoát mà ai cũng đoán ra — thiếu nó thì trên điện thoại người
    // dùng dễ thấy bí.
    render(<GrammarOverlay points={POINTS} />);
    fireEvent.click(openButton());
    fireEvent.click(screen.getByLabelText("Đóng giải thích ngữ pháp"));

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("đóng được bằng phím Esc", () => {
    render(<GrammarOverlay points={POINTS} />);
    fireEvent.click(openButton());
    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("giữ được bảng trong phần giải thích", () => {
    // Lý do chọn tấm phủ thay vì bong bóng nhỏ như bản dịch: giải thích ngữ
    // pháp nào cũng có bảng chia theo patchim, nhét vào 24rem là vỡ hết.
    render(<GrammarOverlay points={POINTS} />);
    fireEvent.click(openButton());

    expect(screen.getByRole("dialog").querySelector("table")).toBeTruthy();
  });
});
