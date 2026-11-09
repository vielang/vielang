import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { MyPageView } from "./my-page-view";
import { dayKey, EMPTY_DAY } from "@/lib/activity";
import { useActivityStore } from "@/lib/activity-store";
import { useProgressStore } from "@/lib/progress-store";
import { courseCard, courseProgressId, lessonHref, listCourses } from "@/lib/courses";

const studiedToday = (minutes: number) => ({
  [dayKey(new Date())]: { ...EMPTY_DAY, activeMs: minutes * 60_000 },
});

const course = courseCard(listCourses()[0]);

beforeEach(() => {
  localStorage.clear();
  useActivityStore.setState({ days: {}, weeklyGoalMinutes: 60 });
  useProgressStore.setState({ books: {}, hasHydrated: true });
});

describe("lần đầu vào", () => {
  it("mời bắt đầu, không vẽ một trang toàn số 0", () => {
    render(<MyPageView courses={[course]} />);

    expect(screen.getByText(/Bắt đầu hành trình/)).toBeTruthy();
    expect(screen.getByRole("link", { name: /Chọn khoá học/ })).toBeTruthy();
  });

  it("chưa có gì để lưu thì không mời lưu ra file, chỉ mời nhập", () => {
    render(<MyPageView courses={[course]} />);

    expect(screen.queryByRole("button", { name: /Lưu ra file/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Nhập từ file/ })).toBeTruthy();
  });
});

describe("lời nhắn đồng hành và việc làm ngay", () => {
  it("ghi nhận hôm nay và mời học tiếp bài chưa học đầu tiên", () => {
    useActivityStore.setState({ days: studiedToday(30) });
    useProgressStore.setState({
      books: { [courseProgressId(course.id)]: { lastPage: 1, readPages: [1], bookmarks: [], updatedAt: "" } },
    });
    render(<MyPageView courses={[course]} />);

    expect(screen.getByText("Hôm nay bạn đã học 30 phút")).toBeTruthy();
    const cta = screen.getByRole("link", { name: new RegExp(`Học tiếp ${course.title}`) });
    expect(cta.getAttribute("href")).toBe(lessonHref(course.id, course.lessons[1].slug));
  });

  it("tuần này: số phút so với mục tiêu, đổi được mục tiêu", () => {
    useActivityStore.setState({ days: studiedToday(30) });
    render(<MyPageView courses={[course]} />);

    expect(screen.getByText(/\/ 60 phút/).parentElement?.textContent).toContain("30");
    fireEvent.click(screen.getByRole("button", { name: "Đổi mục tiêu" }));
    fireEvent.click(screen.getByRole("radio", { name: "150 phút" }));

    expect(useActivityStore.getState().weeklyGoalMinutes).toBe(150);
  });
});

describe("khoá học đang học", () => {
  it("đã học vài bài thì hiện tiến độ và mời học tiếp bài chưa học đầu tiên", () => {
    useProgressStore.setState({
      books: { [courseProgressId(course.id)]: { lastPage: 1, readPages: [1], bookmarks: [], updatedAt: "" } },
    });
    render(<MyPageView courses={[course]} />);

    expect(screen.getByText("Khoá học đang học")).toBeTruthy();
    expect(screen.getByText(`Đã học 1/${course.total} bài`)).toBeTruthy();
    const next = course.lessons[1];
    const link = screen.getByRole("link", { name: `Học tiếp: ${next.title}` });
    expect(link.getAttribute("href")).toBe(lessonHref(course.id, next.slug));
  });

  it("chưa học bài nào thì không hiện khoá đó", () => {
    useActivityStore.setState({ days: studiedToday(5) });
    render(<MyPageView courses={[course]} />);

    expect(screen.queryByText("Khoá học đang học")).toBeNull();
  });
});
