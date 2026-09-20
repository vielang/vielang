import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { OfflineDownload } from "./offline-download";
import { TooltipProvider } from "@/components/ui/tooltip";
import { BOOKS } from "@/lib/books";
import { useDownloadStore } from "@/lib/download-store";

const [book, other] = BOOKS;

function widget() {
  render(
    <TooltipProvider>
      <OfflineDownload book={book} />
    </TooltipProvider>
  );
}

function setStore(partial: Partial<ReturnType<typeof useDownloadStore.getState>>) {
  useDownloadStore.setState({
    offlineBooks: [],
    hasLoaded: true,
    active: null,
    error: null,
    ...partial,
  });
}

beforeEach(() => setStore({}));

describe("chưa tải", () => {
  it("nói luôn dung lượng trên nhãn nút, không tốn thêm một dòng", () => {
    widget();
    expect(screen.getByRole("button").textContent).toMatch(/Tải offline · \d+ MB/);
  });
});

describe("đang tải", () => {
  beforeEach(() =>
    setStore({ active: { bookId: book.id, progress: { done: 57, total: 228 } } })
  );

  it("hiện phần trăm chứ không phải phân số", () => {
    widget();
    expect(screen.getByText("25%")).toBeTruthy();
  });

  it("KHÔNG vẽ thêm thanh tiến độ", () => {
    // Cả hàng nút vừa khít màn điện thoại; thêm một thanh 80px là nó tụt
    // xuống dòng mới giữa lúc đang tải rồi lại nhảy lên khi xong. Ngay phía
    // trên cũng đã có một thanh tiến độ ĐỌC rồi.
    widget();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });

  it("vẫn tra được số trang chính xác khi cần", () => {
    widget();
    expect(screen.getByTitle("Đang tải 57/228 trang")).toBeTruthy();
  });

  it("huỷ được giữa chừng", () => {
    const cancel = vi.fn();
    useDownloadStore.setState({ cancel });
    widget();
    fireEvent.click(screen.getByLabelText("Huỷ tải về"));

    expect(cancel).toHaveBeenCalled();
  });
});

describe("đã tải", () => {
  it("gộp nhãn và nút xoá làm một", () => {
    setStore({ offlineBooks: [book.id] });
    widget();

    expect(screen.getByText("Đã tải offline")).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(1);
  });
});

describe("hết chỗ", () => {
  it("bấm tải thì hỏi xoá cuốn đang chiếm chỗ, không tải thêm", () => {
    setStore({ offlineBooks: [other.id] });
    widget();
    fireEvent.click(screen.getByText(/Tải offline/));

    expect(screen.getByText("Đã đủ số sách tải về")).toBeTruthy();
    expect(screen.getByText(other.titleVi)).toBeTruthy();
  });
});

describe("chưa đọc xong danh sách sách đã tải", () => {
  it("chưa vẽ gì, tránh nút nhảy từ “tải về” sang “đã tải”", () => {
    setStore({ hasLoaded: false });
    widget();

    expect(screen.queryByRole("button")).toBeNull();
  });
});
