import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { ContinueReading } from "./continue-reading";
import { BOOKS } from "@/lib/books";
import { useProgressStore, type BookProgress } from "@/lib/progress-store";

/**
 * Dải "Đọc tiếp" là lối tắt, không phải thư viện thứ hai — nên thứ tự và số
 * lượng mới là phần đáng kiểm, chứ không phải giao diện.
 */
const korean = BOOKS.filter((b) => b.lang === "ko").slice(0, 4);

function progress(lastPage: number, updatedAt: string): BookProgress {
  return { lastPage, readPages: [lastPage], bookmarks: [], updatedAt };
}

function setProgress(entries: Record<string, BookProgress>) {
  useProgressStore.setState({ books: entries, hasHydrated: true });
}

beforeEach(() => {
  localStorage.clear();
  useProgressStore.setState({ books: {}, hasHydrated: true });
});

describe("khi chưa đọc gì", () => {
  it("không hiện gì cả, không chiếm chỗ của lưới sách", () => {
    render(<ContinueReading books={korean} />);
    expect(screen.queryByText("Đọc tiếp")).toBeNull();
  });
});

describe("thứ tự", () => {
  it("cuốn vừa gấp lại đứng đầu", () => {
    setProgress({
      [korean[0].id]: progress(10, "2026-09-01T00:00:00.000Z"),
      [korean[1].id]: progress(20, "2026-09-20T00:00:00.000Z"),
      [korean[2].id]: progress(30, "2026-09-10T00:00:00.000Z"),
    });
    render(<ContinueReading books={korean} />);

    const titles = screen
      .getAllByRole("link")
      .map((a) => a.textContent ?? "");
    expect(titles[0]).toContain(korean[1].titleVi);
    expect(titles[1]).toContain(korean[2].titleVi);
    expect(titles[2]).toContain(korean[0].titleVi);
  });

  it("chỉ giữ tối đa 3 cuốn", () => {
    setProgress(
      Object.fromEntries(
        korean.map((b, i) => [b.id, progress(5, `2026-09-0${i + 1}T00:00:00.000Z`)])
      )
    );
    render(<ContinueReading books={korean} />);

    expect(screen.getAllByRole("link")).toHaveLength(3);
  });
});

describe("đường dẫn", () => {
  it("đi thẳng tới trang đang đọc dở, không về trang chi tiết sách", () => {
    setProgress({ [korean[0].id]: progress(42, "2026-09-20T00:00:00.000Z") });
    render(<ContinueReading books={korean} />);

    expect(screen.getByRole("link")).toHaveProperty(
      "href",
      expect.stringContaining(`/read/${korean[0].id}/42`)
    );
  });
});

describe("lọc theo ngôn ngữ", () => {
  it("bỏ qua sách không thuộc danh sách đang xem", () => {
    // Đọc dở sách tiếng Hàn thì không việc gì phải hiện ở trang tiếng Anh.
    setProgress({ [korean[0].id]: progress(10, "2026-09-20T00:00:00.000Z") });
    render(<ContinueReading books={[]} />);

    expect(screen.queryByText("Đọc tiếp")).toBeNull();
  });
});

describe("trước khi nạp xong tiến độ", () => {
  it("chưa vẽ gì, tránh lệch với HTML server render", () => {
    setProgress({ [korean[0].id]: progress(10, "2026-09-20T00:00:00.000Z") });
    useProgressStore.setState({ hasHydrated: false });
    render(<ContinueReading books={korean} />);

    expect(screen.queryByText("Đọc tiếp")).toBeNull();
  });
});
