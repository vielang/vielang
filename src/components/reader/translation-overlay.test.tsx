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

describe("bấm lại chính vùng đang mở", () => {
  it("đóng bong bóng thay vì mở lại", () => {
    render(<TranslationOverlay regions={REGIONS} />);
    fireEvent.click(open("Xem bản dịch: Hội thoại"));
    fireEvent.click(open("Đóng bản dịch: Hội thoại"));

    expect(screen.queryByText("Xin chào?")).toBeNull();
    expect(screen.getByLabelText("Xem bản dịch: Hội thoại")).toBeTruthy();
  });
});

describe("chuyển sang vùng khác", () => {
  it("mở vùng mới chứ không đóng hẳn", () => {
    // Mỗi trang có nhiều đoạn; đọc xong đoạn này bấm sang đoạn kế là chuyện
    // thường, không nên bắt bấm hai lần.
    render(<TranslationOverlay regions={REGIONS} />);
    fireEvent.click(open("Xem bản dịch: Hội thoại"));
    fireEvent.click(open("Xem bản dịch: Bài đọc"));

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
