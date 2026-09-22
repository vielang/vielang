import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MyPageView } from "./my-page-view";
import { BOOKS } from "@/lib/books";
import { dayKey, EMPTY_DAY } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { useProgressStore } from "@/lib/progress-store";
import { useQuizStore } from "@/lib/quiz-store";
import { getPageQuiz, isGradable } from "@/lib/quiz";

beforeEach(() => {
  localStorage.clear();
  useActivityStore.setState({ days: {}, studied: {}, weeklyGoalMinutes: 60 });
  useProgressStore.setState({ books: {}, hasHydrated: true });
  useQuizStore.setState({ pages: {} });
});

describe("My page", () => {
  it("chưa học gì thì mời chọn sách, không vẽ một trang toàn số 0", () => {
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/Chưa có dữ liệu học nào/)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Chọn sách để học" })).toBeTruthy();
  });

  it("hiện chuỗi ngày, phút tuần này và sách đang học", () => {
    useActivityStore.setState({
      days: { [dayKey(new Date())]: { ...EMPTY_DAY, activeMs: 30 * 60_000 } },
      studied: { step1: [18, 19] },
    });
    useProgressStore.setState({
      books: {
        step1: { lastPage: 19, readPages: [17, 18, 19], bookmarks: [], updatedAt: "2025-01-01" },
      },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText("1 ngày")).toBeTruthy();
    expect(screen.getByText(/\/ 60 phút/).parentElement?.textContent).toContain("30");
    expect(screen.getByText(/Đã học 2\/228 trang · đã xem 3 trang/)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Đọc tiếp — trang 19" })).toBeTruthy();
  });

  it("liệt kê câu còn sai, bấm vào là tới đúng trang", () => {
    const item = getPageQuiz("step1", 19)
      .flatMap((s) => s.items)
      .find(isGradable)!;
    useProgressStore.setState({
      books: { step1: { lastPage: 19, readPages: [19], bookmarks: [], updatedAt: "" } },
    });
    useQuizStore.setState({
      pages: { "step1:19": { answers: { [item.id]: 999 }, checked: [item.id] } },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/0\/1/)).toBeTruthy();
    const link = screen.getByText(item.prompt).closest("a");
    expect(link?.getAttribute("href")).toBe("/read/step1/19");
  });
});
