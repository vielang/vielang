import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { TranslationOverlay } from "./translation-overlay";
import type { TranslationRegion } from "@/lib/page-translation";

const REGIONS: TranslationRegion[] = [
  {
    id: "r1",
    rect: [0.1, 0.2, 0.3, 0.1],
    label: "Hội thoại",
    ko: "안녕하세요?",
    vi: "Xin chào?",
  },
  {
    id: "r2",
    rect: [0.5, 0.2, 0.3, 0.1],
    label: "Bài đọc",
    ko: "저는 학생이에요.",
    vi: "Tôi là học sinh.",
  },
];

const open = (label: string) => screen.getByLabelText(new RegExp(label));

describe("chấm dịch", () => {
  it("lúc đóng thì mời xem bản dịch", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    expect(screen.getByLabelText("Xem bản dịch: Hội thoại")).toBeTruthy();
  });

  it("mở ra thì đổi thành lời mời ĐÓNG", () => {
    // Bấm ra ngoài vốn đã đóng được, nhưng không có gì nói ra điều đó —
    // đổi nhãn và icon là cách để người dùng thấy có đường đóng.
    render(<TranslationOverlay regions={REGIONS} />);
    fireEvent.click(open("Xem bản dịch: Hội thoại"));

    expect(screen.getByLabelText("Đóng bản dịch: Hội thoại")).toBeTruthy();
    expect(screen.getByText("Xin chào?")).toBeTruthy();
  });

  it("báo trạng thái mở/đóng cho trình đọc màn hình", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    const button = open("Xem bản dịch: Hội thoại");
    expect(button.getAttribute("aria-expanded")).toBe("false");

    fireEvent.click(button);
    expect(open("Đóng bản dịch: Hội thoại").getAttribute("aria-expanded")).toBe("true");
  });
});

/**
 * Trình duyệt thật bắn `pointerdown` TRƯỚC `click`. Bong bóng nghe
 * `pointerdown` ở pha bắt để đóng khi bấm ra ngoài, nên chỉ bắn mỗi
 * `click` là bỏ lọt đúng loại lỗi do hai thứ đó giẫm chân nhau.
 */
function tap(el: HTMLElement) {
  fireEvent.pointerDown(el, { bubbles: true });
  fireEvent.click(el);
}

describe("bấm lại chính vùng đang mở", () => {
  it("đóng bong bóng thay vì mở lại", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    tap(open("Xem bản dịch: Hội thoại"));
    tap(open("Đóng bản dịch: Hội thoại"));

    expect(screen.queryByText("Xin chào?")).toBeNull();
    expect(screen.getByLabelText("Xem bản dịch: Hội thoại")).toBeTruthy();
  });
});

describe("bấm ra ngoài", () => {
  it("vẫn đóng được như cũ", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    tap(open("Xem bản dịch: Hội thoại"));
    fireEvent.pointerDown(document.body, { bubbles: true });

    expect(screen.queryByText("Xin chào?")).toBeNull();
  });
});

describe("chuyển sang vùng khác", () => {
  it("mở vùng mới chứ không đóng hẳn", () => {
    // Mỗi trang có nhiều đoạn; đọc xong đoạn này bấm sang đoạn kế là chuyện
    // thường, không nên bắt bấm hai lần.
    render(<TranslationOverlay regions={REGIONS} />);
    tap(open("Xem bản dịch: Hội thoại"));
    tap(open("Xem bản dịch: Bài đọc"));

    expect(screen.getByText("Tôi là học sinh.")).toBeTruthy();
    expect(screen.queryByText("Xin chào?")).toBeNull();
    expect(screen.getByLabelText("Xem bản dịch: Hội thoại")).toBeTruthy();
  });
});

describe("trang không có vùng dịch nào", () => {
  it("không vẽ gì cả", () => {
    const { container } = render(<TranslationOverlay regions={[]} />);
    expect(container.innerHTML).toBe("");
  });
});

/**
 * Chỉ cái chấm bấm được, KHÔNG phải cả đoạn văn.
 *
 * Trước đây cả khung đoạn là nút. Đo trên 239 vùng thật: một vùng chiếm
 * trung bình 18% diện tích trang, trang nặng nhất tới 56%. Chừng đó trang
 * thành nút vô hình, mà `page-viewer` lại bỏ qua tap bắt đầu trên vùng dịch
 * — nên hơn nửa trang chết cử chỉ chạm để bật thanh công cụ.
 */
describe("vùng bấm", () => {
  /** Khung định vị của một vùng — nhận ra bằng chiều rộng theo tỉ lệ. */
  function frameOf(container: HTMLElement, widthPercent: string) {
    return Array.from(container.querySelectorAll("div")).find(
      (el) => el.style.width === widthPercent
    );
  }

  it("khung đoạn văn không phải là nút và không nhận chạm", () => {
    const { container } = render(<TranslationOverlay regions={REGIONS} />);
    const frame = frameOf(container, "30%");

    expect(frame).toBeDefined();
    expect(frame!.tagName).toBe("DIV");
    expect(frame!.className).toContain("pointer-events-none");
  });

  it("vùng chạm quanh chấm đủ to cho ngón tay", () => {
    // size-11 = 44px, mức tối thiểu Apple khuyến nghị. Để trần 20px thì
    // trên điện thoại bấm mười lần trượt ba.
    render(<TranslationOverlay regions={REGIONS} />);

    expect(open("Xem bản dịch: Hội thoại").className).toMatch(/\bsize-11\b/);
  });

  it("dấu hiệu cho page-viewer nằm trên NÚT chứ không trên khung đoạn", () => {
    // `page-viewer` đọc `data-translate-region` để biết "tap này thuộc về
    // vùng dịch, đừng bật/tắt thanh công cụ". Gắn nhầm lên khung đoạn là
    // nuốt mất cử chỉ chạm trên tới 56% diện tích trang.
    const { container } = render(<TranslationOverlay regions={REGIONS} />);
    const tagged = Array.from(
      container.querySelectorAll("[data-translate-region]")
    );

    expect(tagged).toHaveLength(REGIONS.length);
    for (const el of tagged) expect(el.tagName).toBe("BUTTON");
  });

  it("chạm vào giữa đoạn văn KHÔNG bung bản dịch", () => {
    const { container } = render(<TranslationOverlay regions={REGIONS} />);
    tap(frameOf(container, "30%")!);

    expect(screen.queryByText("Xin chào?")).toBeNull();
  });
});

describe("chỗ đặt chấm tuỳ chỉnh (`dot`)", () => {
  const withDot: TranslationRegion[] = [
    { id: "d1", rect: [0.1, 0.1, 0.5, 0.2], label: "Văn hoá và thông tin", vi: "…", dot: [0.62, 0.28] },
  ];

  it("chấm canh TÂM vào đúng điểm đã chọn, không nằm dưới mép vùng chữ", () => {
    render(<TranslationOverlay regions={withDot} />);
    const button = screen.getByLabelText("Xem bản dịch: Văn hoá và thông tin");
    const frame = button.parentElement as HTMLElement;

    expect(parseFloat(frame.style.left)).toBeCloseTo(62);
    expect(parseFloat(frame.style.top)).toBeCloseTo(28);
    expect(button.className).toMatch(/-translate-y-1\/2/);
    expect(button.className).not.toMatch(/top-full/);
  });

  it("không có `dot` thì vẫn nằm dưới mép vùng chữ như cũ", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    const button = screen.getByLabelText("Xem bản dịch: Hội thoại");

    expect(button.className).toMatch(/top-full/);
    expect((button.parentElement as HTMLElement).style.width).toBe("30%");
  });
});
