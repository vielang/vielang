import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { BookActionsMenu } from "./book-actions-menu";
import { BOOKS } from "@/lib/books";
import { useDownloadStore } from "@/lib/download-store";
import { answersHidden, useReaderPrefsStore } from "@/lib/reader-prefs-store";

const [book, other] = BOOKS;

function menu(props: Partial<{ startedReading: boolean; hasAnswers: boolean }> = {}) {
  render(<BookActionsMenu book={book} startedReading={false} hasAnswers={false} {...props} />);
}

function openMenu() {
  fireEvent.click(screen.getByRole("button", { name: "Thêm tuỳ chọn cho sách này" }));
}

function setDownloads(partial: Partial<ReturnType<typeof useDownloadStore.getState>>) {
  useDownloadStore.setState({
    offlineBooks: [],
    hasLoaded: true,
    active: null,
    error: null,
    ...partial,
  });
}

beforeEach(() => {
  setDownloads({});
  useReaderPrefsStore.setState({ hiddenAnswerBooks: [] });
});

describe("menu ⋮ trên điện thoại", () => {
  it("đóng thì chỉ có một nút ⋮, không bày gì ra hàng nút", () => {
    menu({ startedReading: true, hasAnswers: true });
    expect(screen.queryByText(/Tải offline/)).toBeNull();
    expect(screen.queryByText("Đọc lại từ trang 1")).toBeNull();
  });

  it("mở ra có đủ các việc phụ", () => {
    menu({ startedReading: true, hasAnswers: true });
    openMenu();
    expect(screen.getByText("Đọc lại từ trang 1").closest("a")?.getAttribute("href")).toBe(
      `/read/${book.id}/1`
    );
    expect(screen.getByText(/Tải offline · \d+ MB/)).toBeTruthy();
    expect(screen.getByText("Đáp án: bật")).toBeTruthy();
  });

  it("chưa đọc trang nào thì chưa có mục đọc lại từ đầu", () => {
    menu();
    openMenu();
    expect(screen.queryByText("Đọc lại từ trang 1")).toBeNull();
  });

  it("hộp hỏi xoá cuốn đang chiếm chỗ vẫn ở lại sau khi menu đóng", () => {
    setDownloads({ offlineBooks: [other.id] });
    menu();
    openMenu();
    fireEvent.click(screen.getByText(/Tải offline/));

    // Menu đã đóng (mục trong menu biến mất) mà hộp vẫn còn.
    expect(screen.queryByText(/Tải offline ·/)).toBeNull();
    expect(screen.getByText("Đã đủ số sách tải về")).toBeTruthy();
    expect(screen.getByText(other.titleVi)).toBeTruthy();
  });

  it("đã tải thì mục trong menu dẫn tới hộp xác nhận xoá", () => {
    setDownloads({ offlineBooks: [book.id] });
    menu();
    openMenu();
    fireEvent.click(screen.getByText(/Đã tải offline/));
    expect(screen.getByText(`Xoá bản offline của ${book.titleVi}?`)).toBeTruthy();
  });

  it("đang tải thì phần trăm hiện NGOÀI menu, không phải mở ra mới thấy", () => {
    setDownloads({ active: { bookId: book.id, progress: { done: 30, total: 120 } } as never });
    menu();
    expect(screen.getByText("25%")).toBeTruthy();
  });

  it("bật/tắt đáp án của RIÊNG cuốn này, menu vẫn mở để thấy chữ đổi", () => {
    menu({ hasAnswers: true });
    openMenu();
    fireEvent.click(screen.getByText("Đáp án: bật"));
    expect(answersHidden(useReaderPrefsStore.getState(), book.id)).toBe(true);
    expect(screen.getByText("Đáp án: tắt")).toBeTruthy();
  });
});
