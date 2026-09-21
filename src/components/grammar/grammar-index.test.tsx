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
    exVi: "Trường hợp có thời gian thì mình đi ăn nhé?",
    bookId: "step2",
    page: 55,
  },
];

const TITLES = { step1: "Sơ cấp 1", step2: "Sơ cấp 2" };

function index() {
  return render(<GrammarIndex entries={ENTRIES} bookTitles={TITLES} />);
}

const searchBox = () => screen.getByLabelText("Tra cứu ngữ pháp");
const type = (value: string) =>
  fireEvent.change(searchBox(), { target: { value } });

/**
 * Ô này nằm ngay đầu trang thư viện, thay chỗ tiêu đề. Nếu gõ một ký tự đã
 * đổ ra hàng chục kết quả thì nó đẩy tụt lưới sách xuống mỗi lần chạm nhầm
 * — mà phần lớn người mở thư viện là để đọc tiếp, không phải để tra.
 */
describe("chưa gõ đủ thì chưa đổ danh sách", () => {
  it("chưa gõ gì thì không hiện mục nào", () => {
    index();

    expect(screen.queryByText("명 이에요/예요")).toBeNull();
    expect(screen.queryByText("동 형 -으면")).toBeNull();
  });

  it("mới gõ một ký tự cũng chưa hiện", () => {
    index();
    type("이");

    expect(screen.queryByText("명 이에요/예요")).toBeNull();
  });

  it("nhưng vẫn cho biết trong kho có gì", () => {
    // Ô này thế chỗ tiêu đề "Thư viện", nên phải gánh luôn phần thông tin
    // mà tiêu đề từng mang — nếu không thì trang mở ra chỉ còn một ô trống.
    index();

    expect(screen.getByText(/2 điểm ngữ pháp/)).toBeTruthy();
  });

  it("chưa gõ đủ thì KHÔNG báo 'không tìm thấy'", () => {
    // Gõ một chữ mà đã bảo không tìm thấy là sai và gây hoang mang.
    index();
    type("이");

    expect(screen.queryByText(/Không tìm thấy/)).toBeNull();
  });
});

describe("gõ đủ rồi", () => {
  it("lọc theo tiếng Hàn", () => {
    index();
    type("이에요");

    expect(screen.getByText("명 이에요/예요")).toBeTruthy();
    expect(screen.queryByText("동 형 -으면")).toBeNull();
  });

  it("lọc theo tiếng Việt KHÔNG dấu", () => {
    index();
    type("dieu kien");

    expect(screen.getByText("동 형 -으면")).toBeTruthy();
    expect(screen.queryByText("명 이에요/예요")).toBeNull();
  });

  it("mỗi mục dẫn thẳng tới đúng trang trong sách", () => {
    // Đây là lý do tồn tại của ô này. Kiểm CẢ HAI sách, không chỉ mục đầu:
    // chốt cứng "step1" vào đường dẫn vẫn làm mục đầu đúng, nên chỉ kiểm
    // một mục là bỏ lọt đúng loại lỗi khiến mọi mục Sơ cấp 2 mở nhầm sách.
    index();

    type("이에요");
    expect(
      screen.getByText("명 이에요/예요").closest("a")?.getAttribute("href")
    ).toBe("/read/step1/15");

    type("dieu kien");
    expect(
      screen.getByText("동 형 -으면").closest("a")?.getAttribute("href")
    ).toBe("/read/step2/55");
  });

  it("nói rõ mục đó ở sách nào, trang nào", () => {
    index();
    type("이에요");

    expect(screen.getByText(/Sơ cấp 1/)).toBeTruthy();
    expect(screen.getByText(/tr\./)).toBeTruthy();
  });

  it("hiện cả nghĩa lẫn câu ví dụ", () => {
    index();
    type("이에요");

    expect(screen.getByText("Dùng khi nói tên người hoặc tên đồ vật.")).toBeTruthy();
    expect(screen.getByText("저는 후엔이에요.")).toBeTruthy();
    expect(screen.getByText("Tôi là Huyền.")).toBeTruthy();
  });

  it("báo số kết quả tìm được", () => {
    index();
    type("이에요");

    expect(screen.getByText(/1 kết quả/)).toBeTruthy();
  });

  it("không khớp gì thì chỉ đường thay vì để trống", () => {
    index();
    type("xyzzy");

    expect(screen.getByText(/Không tìm thấy/)).toBeTruthy();
  });
});

describe("xoá từ khoá", () => {
  it("quay lại trạng thái chưa tra, không đổ danh sách", () => {
    index();
    type("이에요");
    fireEvent.click(screen.getByLabelText("Xoá từ khoá"));

    expect(screen.queryByText("명 이에요/예요")).toBeNull();
    expect(screen.getByText(/2 điểm ngữ pháp/)).toBeTruthy();
  });

  it("chưa gõ gì thì không bày nút xoá", () => {
    index();

    expect(screen.queryByLabelText("Xoá từ khoá")).toBeNull();
  });
});

/**
 * Bàn phím tiếng Hàn (và gõ Telex tiếng Việt) ghép chữ dần dần, bắn
 * `compositionstart` → nhiều `change` với mảnh dở dang → `compositionend`.
 *
 * Lọc trên mấy mảnh đó vừa vô nghĩa vừa làm danh sách nhấp nháy, và trên
 * một số bàn phím còn làm đứt mạch ghép chữ — gõ xong cả từ mà ô vẫn như
 * chưa nhận. Đây là loại lỗi CHỈ xuất hiện trên bàn phím thật, không bao
 * giờ tái hiện bằng chuột và bàn phím Latinh, nên phải chốt ở đây.
 */
describe("bộ gõ ghép chữ (IME)", () => {
  /** Gõ `이에` qua bộ gõ Hàn: ㅇ → 이 → 이ㅇ → 이에. */
  function composeKorean() {
    const box = searchBox();
    fireEvent.compositionStart(box);
    for (const step of ["ㅇ", "이", "이ㅇ"]) {
      fireEvent.change(box, { target: { value: step } });
    }
    fireEvent.change(box, { target: { value: "이에" } });
    fireEvent.compositionEnd(box, { target: { value: "이에" } });
  }

  it("ô vẫn hiện đúng từng bước người ta gõ", () => {
    index();
    composeKorean();

    expect((searchBox() as HTMLInputElement).value).toBe("이에");
  });

  it("ghép xong thì LỌC — đây chính là chỗ từng hỏng", () => {
    // Nếu chỉ hạ cờ ở `compositionend` mà không lấy giá trị từ sự kiện, thì
    // chữ vừa ghép xong không bao giờ được đem đi lọc: gõ xong cả từ mà
    // danh sách vẫn trống.
    index();
    composeKorean();

    expect(screen.getByText("명 이에요/예요")).toBeTruthy();
  });

  it("không lọc theo mảnh dở dang giữa chừng", () => {
    index();
    const box = searchBox();
    fireEvent.compositionStart(box);
    fireEvent.change(box, { target: { value: "이ㅇ" } });

    // "이ㅇ" không khớp mục nào; nếu đem đi lọc thì hiện "không tìm thấy",
    // nhấp nháy đúng lúc người ta đang gõ dở.
    expect(screen.queryByText(/Không tìm thấy/)).toBeNull();
  });
});

describe("khi mới gõ một ký tự", () => {
  it("nói rõ là phải gõ thêm, không im lặng", () => {
    // Gõ một ký tự mà giao diện không đổi gì thì trông y như hỏng — người
    // dùng không có cách nào đoán ra là cần gõ thêm.
    index();
    type("d");

    expect(screen.getByText(/Gõ thêm 1 ký tự nữa/)).toBeTruthy();
  });
});
