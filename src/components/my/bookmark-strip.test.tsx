import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { BookmarkStrip, pinnedPages } from "./bookmark-strip";
import { BOOKS } from "@/lib/books";
import type { BookProgress } from "@/lib/progress-store";

const [a, b] = BOOKS;

function progress(bookmarks: number[], updatedAt: string): BookProgress {
  return { lastPage: 1, readPages: [], bookmarks, updatedAt };
}

describe("trang đã đánh dấu trong Góc học tập", () => {
  it("sách đọc gần nhất lên trước, trong một sách theo số trang", () => {
    const pins = pinnedPages(BOOKS, {
      [a.id]: progress([30, 12], "2026-09-01T00:00:00Z"),
      [b.id]: progress([5], "2026-09-20T00:00:00Z"),
    });
    expect(pins.map((p) => `${p.book.id}:${p.page}`)).toEqual([`${b.id}:5`, `${a.id}:12`, `${a.id}:30`]);
  });

  it("chưa ghim trang nào thì không vẽ gì", () => {
    const { container } = render(
      <BookmarkStrip books={BOOKS} progressByBook={{ [a.id]: progress([], "2026-09-01") }} />
    );
    expect(container.innerHTML).toBe("");
  });

  it("mỗi ảnh mở đúng trang, kèm lối sang danh sách đầy đủ", () => {
    render(<BookmarkStrip books={BOOKS} progressByBook={{ [a.id]: progress([12], "2026-09-01") }} />);
    expect(screen.getByRole("link", { name: /trang 12/ }).getAttribute("href")).toBe(`/read/${a.id}/12`);
    expect(screen.getByRole("link", { name: /Xem tất cả/ }).getAttribute("href")).toBe("/my/danh-dau");
  });
});
