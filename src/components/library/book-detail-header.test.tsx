import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { BookDetailHeader } from "./book-detail-header";
import { TooltipProvider } from "@/components/ui/tooltip";
import { BOOKS } from "@/lib/books";
import { useProgressStore, type BookProgress } from "@/lib/progress-store";

/**
 * Đầu trang này từng có sáu khối chồng nhau, đẩy lưới trang — thứ người ta
 * thật sự vào đây để xem — xuống tít dưới. Test giữ lại phần đã gom gọn.
 */
const book = BOOKS[0];

function setProgress(p: Partial<BookProgress> | null) {
  useProgressStore.setState({
    books: p
      ? { [book.id]: { lastPage: 1, readPages: [], bookmarks: [], updatedAt: "", ...p } }
      : {},
    hasHydrated: true,
  });
}

function header() {
  render(
    <TooltipProvider>
      <BookDetailHeader book={book} />
    </TooltipProvider>
  );
}

beforeEach(() => {
  localStorage.clear();
  setProgress(null);
});

describe("dữ kiện tĩnh", () => {
  it("gom vào một dòng thay vì mỗi thứ một khối", () => {
    header();
    const meta = screen.getByText(`Cấp ${book.level}`).parentElement;

    expect(meta?.textContent).toContain(`${book.totalPages} trang`);
  });
});

describe("khi chưa đọc trang nào", () => {
  it("không vẽ thanh tiến độ 0%", () => {
    // Thanh 0% không nói thêm điều gì mà vẫn ăn một dòng.
    header();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });

  it("không nhắc phần trăm", () => {
    header();
    expect(screen.queryByText(/đã đọc/)).toBeNull();
  });

  it("nút chính mời bắt đầu", () => {
    header();
    expect(screen.getByText("Bắt đầu đọc")).toBeTruthy();
  });

  it("chưa bày nút đọc lại từ đầu", () => {
    // Chưa đọc gì thì "đọc lại từ đầu" là vô nghĩa.
    header();
    expect(screen.queryByLabelText("Đọc lại từ trang 1")).toBeNull();
  });
});

describe("khi đang đọc dở", () => {
  beforeEach(() => setProgress({ lastPage: 8, readPages: [1, 2, 8] }));

  it("nút chính đi thẳng tới trang đang dở", () => {
    header();
    const link = screen.getByText("Đọc tiếp — trang 8").closest("a");

    expect(link?.getAttribute("href")).toBe(`/read/${book.id}/8`);
  });

  it("lúc này mới vẽ thanh tiến độ", () => {
    header();
    expect(screen.getByRole("progressbar")).toBeTruthy();
  });

  it("có lối đọc lại từ đầu, nhưng chỉ là icon", () => {
    header();
    const back = screen.getByLabelText("Đọc lại từ trang 1");

    expect(back.closest("a")?.getAttribute("href")).toBe(`/read/${book.id}/1`);
    expect(back.textContent).toBe("");
  });
});
