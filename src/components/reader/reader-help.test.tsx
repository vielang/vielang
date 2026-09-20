import { beforeEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { ReaderHelp } from "./reader-help";
import { useReaderPrefsStore } from "@/lib/reader-prefs-store";

/**
 * Bảng hướng dẫn phải tự hiện ĐÚNG MỘT LẦN. Hiện thiếu thì người dùng không
 * bao giờ biết app làm được gì; hiện thừa thì thành phiền mỗi lần mở sách —
 * mà `ReaderView` lại remount sau MỖI lần lật trang, nên "thừa" rất dễ xảy ra.
 */
function Host() {
  const [open, setOpen] = useState(false);
  return <ReaderHelp open={open} onOpenChange={setOpen} />;
}

const seen = () => useReaderPrefsStore.getState().hasSeenReaderHelp;

beforeEach(() => {
  localStorage.clear();
  useReaderPrefsStore.setState({ pageLayout: "single", hasSeenReaderHelp: false });
});

describe("lần mở sách đầu tiên", () => {
  it("tự hiện lên", () => {
    render(<Host />);
    expect(screen.getByText("Cách dùng trang đọc")).toBeTruthy();
  });

  it("nói ra đúng mấy thao tác không có dấu hiệu gì trên màn", () => {
    render(<Host />);
    expect(screen.getByText(/Vuốt ngang/)).toBeTruthy();
    expect(screen.getByText(/Chạm vào một đoạn chữ Hàn/)).toBeTruthy();
    expect(screen.getByText(/Kéo các bảng công cụ/)).toBeTruthy();
  });
});

describe("sau khi đã xem", () => {
  it("bấm “Đã hiểu” thì ghi nhớ và đóng lại", () => {
    render(<Host />);
    fireEvent.click(screen.getByText("Đã hiểu"));

    expect(seen()).toBe(true);
    expect(screen.queryByText("Cách dùng trang đọc")).toBeNull();
  });

  it("KHÔNG tự hiện lại ở những lần mount sau", () => {
    // Lật trang là `ReaderView` remount — không chặn thì bảng bật lên lại
    // sau mỗi trang.
    const first = render(<Host />);
    fireEvent.click(screen.getByText("Đã hiểu"));
    first.unmount();

    render(<Host />);
    expect(screen.queryByText("Cách dùng trang đọc")).toBeNull();
  });

  it("đóng bằng Esc cũng tính là đã xem", () => {
    render(<Host />);
    fireEvent.keyDown(document.body, { key: "Escape" });

    expect(seen()).toBe(true);
  });
});

describe("chưa bấm đã hiểu", () => {
  it("lật trang ngay lúc bảng vừa bật thì lần sau vẫn được xem", () => {
    // Cờ chỉ ghi lúc ĐÓNG, không phải lúc mở.
    const first = render(<Host />);
    expect(seen()).toBe(false);
    first.unmount();

    render(<Host />);
    expect(screen.getByText("Cách dùng trang đọc")).toBeTruthy();
  });
});

describe("mở lại bằng tay", () => {
  it("mở được kể cả khi đã xem rồi", () => {
    useReaderPrefsStore.setState({ hasSeenReaderHelp: true });
    function Manual() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <button type="button" onClick={() => setOpen(true)}>
            mở
          </button>
          <ReaderHelp open={open} onOpenChange={setOpen} />
        </>
      );
    }
    render(<Manual />);
    expect(screen.queryByText("Cách dùng trang đọc")).toBeNull();

    fireEvent.click(screen.getByText("mở"));
    expect(screen.getByText("Cách dùng trang đọc")).toBeTruthy();
  });
});

describe("phím tắt được liệt kê", () => {
  it("khớp với phím mà trang đọc thật sự nghe", () => {
    // Danh sách này rất dễ trôi khỏi `reader-view` — chốt lại ở đây.
    // Đọc riêng các thẻ <kbd>: chữ "?" còn xuất hiện trong câu mô tả nữa.
    render(<Host />);
    const keys = Array.from(document.querySelectorAll("kbd")).map(
      (k) => k.textContent
    );

    expect(keys).toEqual(
      expect.arrayContaining(["←", "→", "Home", "End", "N", "D", "B", "?", "Esc"])
    );
  });
});

describe("không rò rỉ giữa các phiên", () => {
  it("cờ đã xem được lưu xuống trình duyệt", () => {
    render(<Host />);
    fireEvent.click(screen.getByText("Đã hiểu"));

    expect(localStorage.getItem("kiip-reader-prefs-v1")).toContain(
      "hasSeenReaderHelp"
    );
  });
});

// Hộp thoại của Radix đo layout khi mở — jsdom trả 0 cho mọi phép đo, nhưng
// không ảnh hưởng tới những gì test này kiểm.
vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn() }) }));
