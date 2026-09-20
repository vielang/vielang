import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { BookmarkList } from "./bookmark-list";
import { BOOKS } from "@/lib/books";
import { useProgressStore, type BookProgress } from "@/lib/progress-store";

const [bookA, bookB] = BOOKS;

function progress(bookmarks: number[]): BookProgress {
  return { lastPage: 1, readPages: [], bookmarks, updatedAt: "2026-09-20T00:00:00.000Z" };
}

function setBookmarks(entries: Record<string, BookProgress>) {
  useProgressStore.setState({ books: entries, hasHydrated: true });
}

beforeEach(() => {
  localStorage.clear();
  useProgressStore.setState({ books: {}, hasHydrated: true });
});

describe("khi chưa ghim trang nào", () => {
  it("chỉ cho người dùng biết cách ghim, không để màn trống", () => {
    render(<BookmarkList books={BOOKS} />);

    expect(screen.getByText("Chưa đánh dấu trang nào")).toBeTruthy();
    expect(screen.getByText("B")).toBeTruthy();
  });
});

describe("danh sách", () => {
  it("gom theo sách và bỏ qua sách không ghim trang nào", () => {
    setBookmarks({ [bookA.id]: progress([12, 5]) });
    render(<BookmarkList books={BOOKS} />);

    expect(screen.getByText(bookA.titleVi)).toBeTruthy();
    expect(screen.queryByText(bookB.titleVi)).toBeNull();
  });

  it("xếp trang theo số tăng dần, không theo thứ tự ghim", () => {
    setBookmarks({ [bookA.id]: progress([30, 5, 12]) });
    render(<BookmarkList books={BOOKS} />);

    const order = screen
      .getAllByRole("link")
      .map((a) => a.getAttribute("href"))
      .filter((h): h is string => !!h && h.startsWith("/read/"));
    expect(order).toEqual([
      `/read/${bookA.id}/5`,
      `/read/${bookA.id}/12`,
      `/read/${bookA.id}/30`,
    ]);
  });

  it("đếm đúng số trang của từng cuốn", () => {
    setBookmarks({ [bookA.id]: progress([1, 2, 3]) });
    render(<BookmarkList books={BOOKS} />);

    expect(screen.getByText("3 trang")).toBeTruthy();
  });
});

describe("bỏ ghim ngay tại đây", () => {
  it("gỡ trang khỏi danh sách", () => {
    setBookmarks({ [bookA.id]: progress([5, 12]) });
    render(<BookmarkList books={BOOKS} />);

    fireEvent.click(screen.getByLabelText("Bỏ đánh dấu trang 5"));

    expect(useProgressStore.getState().books[bookA.id].bookmarks).toEqual([12]);
    expect(screen.queryByLabelText("Bỏ đánh dấu trang 5")).toBeNull();
  });

  it("bỏ hết thì quay về màn hướng dẫn", () => {
    setBookmarks({ [bookA.id]: progress([5]) });
    render(<BookmarkList books={BOOKS} />);

    fireEvent.click(screen.getByLabelText("Bỏ đánh dấu trang 5"));

    expect(screen.getByText("Chưa đánh dấu trang nào")).toBeTruthy();
  });
});

describe("trước khi nạp xong", () => {
  it("giữ chỗ thay vì nhảy từ trống sang có", () => {
    setBookmarks({ [bookA.id]: progress([5]) });
    useProgressStore.setState({ hasHydrated: false });
    render(<BookmarkList books={BOOKS} />);

    expect(screen.queryByText("Chưa đánh dấu trang nào")).toBeNull();
    expect(screen.queryByText(bookA.titleVi)).toBeNull();
  });
});
