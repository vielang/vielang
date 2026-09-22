import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { MyPageView } from "./my-page-view";
import { BOOKS } from "@/lib/books";
import { dayKey, EMPTY_DAY } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { useProgressStore } from "@/lib/progress-store";
import { useQuizStore } from "@/lib/quiz-store";
import { getPageQuiz, isGradable } from "@/lib/quiz";

const studiedToday = (minutes: number) => ({
  [dayKey(new Date())]: { ...EMPTY_DAY, activeMs: minutes * 60_000 },
});

beforeEach(() => {
  localStorage.clear();
  useActivityStore.setState({ days: {}, studied: {}, weeklyGoalMinutes: 60, grades: {} });
  useProgressStore.setState({ books: {}, hasHydrated: true });
  useQuizStore.setState({ pages: {} });
});

describe("lần đầu vào", () => {
  it("mời bắt đầu, không vẽ một trang toàn số 0", () => {
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/Bắt đầu hành trình/)).toBeTruthy();
    expect(screen.getByRole("link", { name: /Chọn sách để học/ })).toBeTruthy();
  });

  it("chưa có gì để lưu thì không mời lưu ra file, chỉ mời nhập", () => {
    render(<MyPageView books={BOOKS} />);

    expect(screen.queryByRole("button", { name: /Lưu ra file/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Nhập từ file/ })).toBeTruthy();
  });
});

describe("lời nhắn đồng hành và việc làm ngay", () => {
  it("ghi nhận hôm nay và mời học tiếp đúng trang đang dở", () => {
    useActivityStore.setState({ days: studiedToday(30), studied: { step1: [18, 19] } });
    useProgressStore.setState({
      books: {
        step1: { lastPage: 19, readPages: [17, 18, 19], bookmarks: [], updatedAt: "2025-01-01" },
      },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText("Hôm nay bạn đã học 30 phút")).toBeTruthy();
    const cta = screen.getByRole("link", { name: /Học tiếp Giáo trình Sơ cấp 1 · trang 19/ });
    expect(cta.getAttribute("href")).toBe("/read/step1/19");
  });

  it("tuần này: số phút so với mục tiêu, đổi được mục tiêu", () => {
    useActivityStore.setState({ days: studiedToday(30) });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/\/ 60 phút/).parentElement?.textContent).toContain("30");
    fireEvent.click(screen.getByRole("button", { name: "Đổi mục tiêu" }));
    fireEvent.click(screen.getByRole("radio", { name: "150 phút" }));

    expect(useActivityStore.getState().weeklyGoalMinutes).toBe(150);
  });
});

describe("sách đang học", () => {
  it("tên ngắn gọn và số trang đã học", () => {
    useActivityStore.setState({ studied: { step1: [18, 19] } });
    useProgressStore.setState({
      books: { step1: { lastPage: 19, readPages: [19], bookmarks: [], updatedAt: "" } },
    });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText("2/228 trang")).toBeTruthy();
  });
});

describe("gợi ý và năng lực", () => {
  it("chưa tự chấm gì thì chỉ cách để có gợi ý", () => {
    useActivityStore.setState({ days: studiedToday(5) });
    render(<MyPageView books={BOOKS} />);

    expect(screen.getByText(/mình sẽ gợi ý bài nào nên ôn lại/)).toBeTruthy();
    expect(screen.queryByText("Kỹ năng của bạn")).toBeNull();
  });

  it("có lượt tự chấm sai thì gợi ý ôn đúng điểm ngữ pháp, dẫn tới trang giải thích", () => {
    useActivityStore.setState({ grades: { "wb-step1:12:p12-1": { grade: 0, at: "" } } });
    render(<MyPageView books={BOOKS} />);

    const tip = screen.getByRole("link", { name: /Ôn lại ngữ pháp 명 이에요\/예요/ });
    expect(tip.getAttribute("href")).toBe("/read/step1/15");
    expect(screen.getByText("Kỹ năng của bạn")).toBeTruthy();
    expect(screen.getByText(/không phải điểm thi/)).toBeTruthy();
  });

  it("câu quiz còn sai có trong gợi ý và trong danh sách chi tiết", () => {
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

    expect(screen.getByRole("link", { name: /Làm lại câu quiz còn sai/ })).toBeTruthy();
    const detail = screen.getByText(item.prompt).closest("a");
    expect(detail?.getAttribute("href")).toBe("/read/step1/19");
  });
});
