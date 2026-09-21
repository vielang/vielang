import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { GrammarIndex } from "./grammar-index";
import type { GrammarEntry } from "@/lib/page-grammar";

const ENTRIES: GrammarEntry[] = [
  {
    id: "p15-ieyo-yeyo",
    slug: "ieyo-yeyo",
    rect: [0, 0, 1, 1],
    title: "명 이에요/예요",
    vi: "Dùng khi nói tên người hoặc tên đồ vật.",
    exKo: "저는 후엔이에요.",
    exVi: "Tôi là Huyền.",
    bookId: "step1",
    page: 15,
  },
  {
    id: "p55-eumyeon",
    slug: "eumyeon",
    rect: [0, 0, 1, 1],
    title: "동 형 -으면",
    vi: "Nêu điều kiện hoặc giả định.",
    exKo: "시간이 있으면 밥 먹을까요?",
    exVi: "Nếu có thời gian thì mình đi ăn nhé?",
    bookId: "step2",
    page: 55,
  },
];

const TITLES = { step1: "Sơ cấp 1", step2: "Sơ cấp 2" };

function index() {
  return render(<GrammarIndex entries={ENTRIES} bookTitles={TITLES} />);
}

const searchBox = () => screen.getByLabelText("Tìm điểm ngữ pháp");

describe("danh sách", () => {
  it("hiện tất cả khi chưa gõ gì", () => {
    index();

    expect(screen.getByText("명 이에요/예요")).toBeTruthy();
    expect(screen.getByText("동 형 -으면")).toBeTruthy();
    expect(screen.getByText("2 điểm ngữ pháp")).toBeTruthy();
  });

  it("mỗi mục dẫn thẳng tới đúng trang trong sách", () => {
    // Đây là lý do tồn tại của cả trang này. Sai liên kết là tra cứu xong
    // vẫn không tới được chỗ cần xem.
    //
    // Kiểm CẢ HAI sách, không chỉ mục đầu: chốt cứng "step1" vào đường dẫn
    // vẫn làm mục đầu đúng, nên chỉ kiểm một mục là bỏ lọt đúng loại lỗi
    // khiến mọi mục Sơ cấp 2 mở nhầm sang Sơ cấp 1.
    index();

    expect(
      screen.getByText("명 이에요/예요").closest("a")?.getAttribute("href")
    ).toBe("/read/step1/15");
    expect(
      screen.getByText("동 형 -으면").closest("a")?.getAttribute("href")
    ).toBe("/read/step2/55");
  });

  it("nói rõ mục đó ở sách nào, trang nào", () => {
    index();

    expect(screen.getByText("Sơ cấp 1 · tr. 15")).toBeTruthy();
    expect(screen.getByText("Sơ cấp 2 · tr. 55")).toBeTruthy();
  });

  it("hiện cả nghĩa lẫn câu ví dụ", () => {
    index();

    expect(screen.getByText("Dùng khi nói tên người hoặc tên đồ vật.")).toBeTruthy();
    expect(screen.getByText("저는 후엔이에요.")).toBeTruthy();
    expect(screen.getByText("Tôi là Huyền.")).toBeTruthy();
  });
});

describe("tìm kiếm", () => {
  it("lọc theo tiếng Hàn", () => {
    index();
    fireEvent.change(searchBox(), { target: { value: "이에요" } });

    expect(screen.getByText("명 이에요/예요")).toBeTruthy();
    expect(screen.queryByText("동 형 -으면")).toBeNull();
  });

  it("lọc theo tiếng Việt KHÔNG dấu", () => {
    index();
    fireEvent.change(searchBox(), { target: { value: "dieu kien" } });

    expect(screen.getByText("동 형 -으면")).toBeTruthy();
    expect(screen.queryByText("명 이에요/예요")).toBeNull();
  });

  it("báo số kết quả tìm được", () => {
    index();
    fireEvent.change(searchBox(), { target: { value: "이에요" } });

    expect(screen.getByText(/1 kết quả/)).toBeTruthy();
  });

  it("không có kết quả thì chỉ đường thay vì để trang trống", () => {
    index();
    fireEvent.change(searchBox(), { target: { value: "xyzzy" } });

    expect(screen.getByText(/Không tìm thấy/)).toBeTruthy();
  });

  it("xoá được từ khoá để quay lại danh sách đầy đủ", () => {
    index();
    fireEvent.change(searchBox(), { target: { value: "이에요" } });
    fireEvent.click(screen.getByLabelText("Xoá từ khoá"));

    expect(screen.getByText("2 điểm ngữ pháp")).toBeTruthy();
  });

  it("chưa gõ gì thì không bày nút xoá", () => {
    index();

    expect(screen.queryByLabelText("Xoá từ khoá")).toBeNull();
  });
});
