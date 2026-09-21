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

  it("mới gõ một chữ cái Latinh thì chưa hiện", () => {
    index();
    type("d");

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
    type("d");

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
 * Ô vẫn LỌC trong lúc ghép, chỉ nín phần báo không tìm thấy. Tạm ngưng lọc
 * cho tới `compositionend` nghe thì gọn nhưng là một canh bạc: nhiều bàn
 * phím Hàn trên Android giữ nguyên một mạch ghép chữ cho tới khi gõ dấu
 * cách, nên ô sẽ đứng im suốt cả từ. Đây là loại lỗi CHỈ xuất hiện trên bàn
 * phím thật, không bao giờ tái hiện bằng chuột và bàn phím Latinh.
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

  it("ghép xong thì lọc", () => {
    // Nếu chỉ hạ cờ ở `compositionend` mà không lấy giá trị từ sự kiện, thì
    // chữ vừa ghép xong không bao giờ được đem đi lọc: gõ xong cả từ mà
    // danh sách vẫn trống.
    index();
    composeKorean();

    expect(screen.getByText("명 이에요/예요")).toBeTruthy();
  });

  it("không kêu không-tìm-thấy vì mảnh dở dang", () => {
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

/**
 * Lỗi người dùng báo: "chỉ search được tiếng Việt, không search được tiếng
 * Hàn". Thủ phạm là ngưỡng hai ký tự — đuôi ngữ pháp tiếng Hàn phần lớn dài
 * đúng MỘT âm tiết nên không bao giờ với tới, còn từ tiếng Việt tự nhiên đã
 * dài hơn nên không ai thấy gì bất thường.
 */
describe("tra bằng tiếng Hàn một âm tiết", () => {
  it("một âm tiết là đã lọc — đây chính là chỗ từng hỏng", () => {
    index();
    type("면");

    expect(screen.getByText("동 형 -으면")).toBeTruthy();
  });

  it("vẫn lọc đúng chứ không đổ bừa cả danh sách", () => {
    // Hạ ngưỡng mà quên lọc thì test trên vẫn xanh trong khi ô hoá ra vô
    // dụng: gõ gì cũng ra đủ bảy chục mục.
    index();
    type("면");

    expect(screen.queryByText("명 이에요/예요")).toBeNull();
  });

  it("không còn bắt gõ thêm", () => {
    index();
    type("면");

    expect(screen.queryByText(/Gõ thêm/)).toBeNull();
  });

  it("lọc ngay giữa lúc bộ gõ đang ghép, không đợi ghép xong", () => {
    // Trên bàn phím giữ một mạch ghép chữ tới tận dấu cách, đợi
    // `compositionend` nghĩa là không bao giờ lọc.
    index();
    const box = searchBox();
    fireEvent.compositionStart(box);
    fireEvent.change(box, { target: { value: "면" } });

    expect(screen.getByText("동 형 -으면")).toBeTruthy();
  });

  it("ghép xong mà thật sự không khớp thì mới báo", () => {
    // Nín trong lúc ghép là đúng, nín luôn sau khi ghép xong là nuốt mất câu
    // trả lời duy nhất người dùng cần.
    index();
    const box = searchBox();
    fireEvent.compositionStart(box);
    fireEvent.change(box, { target: { value: "쀍" } });
    fireEvent.compositionEnd(box, { target: { value: "쀍" } });

    expect(screen.getByText(/Không tìm thấy/)).toBeTruthy();
  });
});

describe("câu giới thiệu khi chưa gõ gì", () => {
  it("kể tên sách lấy từ DỮ LIỆU chứ không chép tay", () => {
    // Chép cứng "Sơ cấp 1 và Sơ cấp 2" vào câu này đã sai một lần, đúng lúc
    // soạn xong ngữ pháp Trung cấp 1 — người dùng đọc được một câu sai mà
    // không có gì báo lỗi. Giờ phải suy ra từ chính danh sách mục.
    render(
      <GrammarIndex
        entries={[...ENTRIES, { ...ENTRIES[0], id: "x", bookId: "step3" }]}
        bookTitles={{ ...TITLES, step3: "Trung cấp 1" }}
      />
    );

    expect(screen.getByText(/Trung cấp 1/)).toBeTruthy();
    expect(screen.getByText(/3 điểm ngữ pháp/)).toBeTruthy();
  });
});
