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
    exKo: "가: 제이슨이에요? 나: 아니요, 잠시드예요.",
    exVi: "A: Là Jason phải không? B: Không, là Jamshid.",
  },
];

const trigger = () => screen.getByLabelText(/nghĩa ngữ pháp/);

/**
 * Trình duyệt thật bắn `pointerdown` TRƯỚC `click`, mà bong bóng nghe
 * `pointerdown` ở pha bắt để đóng khi bấm ra ngoài — chỉ bắn mỗi `click` là
 * bỏ lọt đúng loại lỗi do hai thứ đó giẫm chân nhau.
 */
function tap(el: HTMLElement) {
  fireEvent.pointerDown(el, { bubbles: true });
  fireEvent.click(el);
}

describe("chấm ngữ pháp", () => {
  it("mời xem nghĩa, kèm tên điểm ngữ pháp", () => {
    render(<GrammarOverlay points={POINTS} />);

    expect(screen.getByLabelText("Xem nghĩa ngữ pháp: 명 이에요/예요")).toBeTruthy();
  });

  it("vùng chạm đủ to cho ngón tay", () => {
    // size-11 = 44px. Giống chấm dịch: nét vẽ vẫn 20px, phần dôi ra là lề
    // vô hình — to cái chấm lên thì chọc vào dáng bản in của trang.
    render(<GrammarOverlay points={POINTS} />);

    expect(trigger().className).toMatch(/\bsize-11\b/);
  });

  it("chấm nằm BÊN TRÁI tiêu đề", () => {
    // Tiêu đề ngữ pháp luôn sát mép phải trang, để chấm bên phải là rơi ra
    // ngoài giấy.
    render(<GrammarOverlay points={POINTS} />);

    expect(trigger().className).toMatch(/\bright-full\b/);
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

describe("bong bóng nghĩa", () => {
  it("hiện tiêu đề và định nghĩa khi bấm", () => {
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());

    const bubble = screen.getByRole("tooltip");
    expect(bubble.textContent).toContain("명 이에요/예요");
    expect(bubble.textContent).toContain("Dùng khi nói tên người hoặc tên đồ vật.");
  });

  it("chưa bấm thì chưa hiện gì", () => {
    render(<GrammarOverlay points={POINTS} />);

    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("hiện cả câu ví dụ tiếng Hàn và bản dịch của nó", () => {
    // Định nghĩa thuần thì đúng nhưng khô — "trợ từ chủ ngữ, đánh dấu chủ
    // thể" chẳng giúp gì cho người mới. Một câu thật mới làm nó rơi xuống.
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    const bubble = screen.getByRole("tooltip");

    expect(bubble.textContent).toContain("가: 제이슨이에요? 나: 아니요, 잠시드예요.");
    expect(bubble.textContent).toContain("A: Là Jason phải không? B: Không, là Jamshid.");
  });

  it("ví dụ nằm SAU định nghĩa", () => {
    // Đọc định nghĩa trước rồi mới soi ví dụ. Đảo lại thì câu Hàn đập vào
    // mắt trước, đúng thứ người ta đang không hiểu.
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    const text = screen.getByRole("tooltip").textContent!;

    expect(text.indexOf(POINTS[0].vi)).toBeLessThan(text.indexOf(POINTS[0].exKo));
  });

  it("CHỈ có định nghĩa ngắn, không phải cả bài giảng", () => {
    // Bản đầu làm cả tấm phủ với bảng chia theo patchim, ví dụ và mục lưu ý.
    // Đọc giữa lúc đang học thì quá dài và rối, mà còn che mất trang sách
    // đang xem. Muốn học sâu thì đã có tab bài giảng.
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    const bubble = screen.getByRole("tooltip");

    expect(bubble.querySelector("table")).toBeNull();
    expect(bubble.textContent!.length).toBeLessThan(200);
  });

  it("bấm lại chính chấm đó thì đóng, không mở lại", () => {
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    tap(trigger());

    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("đang mở thì chấm đổi thành dấu đóng", () => {
    // Bấm ra ngoài vốn đã đóng được, nhưng không có gì nói ra điều đó.
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());

    expect(screen.getByLabelText(/^Đóng nghĩa ngữ pháp/)).toBeTruthy();
  });

  it("bấm ra ngoài thì đóng", () => {
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    fireEvent.pointerDown(document.body, { bubbles: true });

    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("đóng được bằng phím Esc", () => {
    render(<GrammarOverlay points={POINTS} />);
    tap(trigger());
    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});
