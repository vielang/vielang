import { beforeEach, describe, expect, it } from "vitest";
import {
  useProgressStore,
  getBookProgress,
  percentRead,
  resumePage,
} from "./progress-store";

const BOOK = "step1";

beforeEach(() => {
  localStorage.clear();
  useProgressStore.setState({ books: {}, hasHydrated: true });
});

describe("markPageRead", () => {
  it("thêm trang vào readPages và cập nhật lastPage", () => {
    useProgressStore.getState().markPageRead(BOOK, 5);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(progress.readPages).toEqual([5]);
    expect(progress.lastPage).toBe(5);
  });

  it("không thêm trùng khi đọc lại cùng 1 trang", () => {
    useProgressStore.getState().markPageRead(BOOK, 5);
    useProgressStore.getState().markPageRead(BOOK, 5);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(progress.readPages).toEqual([5]);
  });

  it("lastPage luôn là trang vừa đọc gần nhất, kể cả khi quay lại trang trước", () => {
    useProgressStore.getState().markPageRead(BOOK, 5);
    useProgressStore.getState().markPageRead(BOOK, 10);
    useProgressStore.getState().markPageRead(BOOK, 3);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(progress.readPages).toEqual([5, 10, 3]);
    expect(progress.lastPage).toBe(3);
  });

  it("tiến độ của 2 sách khác nhau độc lập với nhau", () => {
    useProgressStore.getState().markPageRead("step1", 1);
    useProgressStore.getState().markPageRead("step2", 1);
    useProgressStore.getState().markPageRead("step2", 2);
    const books = useProgressStore.getState().books;
    expect(getBookProgress(books, "step1").readPages).toEqual([1]);
    expect(getBookProgress(books, "step2").readPages).toEqual([1, 2]);
  });
});

describe("toggleBookmark", () => {
  it("bật rồi tắt bookmark cho 1 trang", () => {
    useProgressStore.getState().toggleBookmark(BOOK, 7);
    expect(
      getBookProgress(useProgressStore.getState().books, BOOK).bookmarks
    ).toEqual([7]);

    useProgressStore.getState().toggleBookmark(BOOK, 7);
    expect(
      getBookProgress(useProgressStore.getState().books, BOOK).bookmarks
    ).toEqual([]);
  });

  it("giữ danh sách bookmark theo thứ tự tăng dần", () => {
    useProgressStore.getState().toggleBookmark(BOOK, 20);
    useProgressStore.getState().toggleBookmark(BOOK, 5);
    useProgressStore.getState().toggleBookmark(BOOK, 12);
    expect(
      getBookProgress(useProgressStore.getState().books, BOOK).bookmarks
    ).toEqual([5, 12, 20]);
  });
});

describe("setLastPage", () => {
  it("cập nhật lastPage mà không đánh dấu đã đọc", () => {
    useProgressStore.getState().setLastPage(BOOK, 42);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(progress.lastPage).toBe(42);
    expect(progress.readPages).toEqual([]);
  });
});

describe("percentRead", () => {
  it("tính đúng phần trăm đã đọc, làm tròn", () => {
    useProgressStore.getState().markPageRead(BOOK, 1);
    useProgressStore.getState().markPageRead(BOOK, 2);
    useProgressStore.getState().markPageRead(BOOK, 3);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(percentRead(progress, 228)).toBe(1); // 3/228 ≈ 1.3% -> 1
  });

  it("trả 0 khi chưa đọc trang nào", () => {
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(percentRead(progress, 228)).toBe(0);
  });

  it("không chia cho 0 khi totalPages = 0", () => {
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(percentRead(progress, 0)).toBe(0);
  });
});

describe("resumePage", () => {
  it("mặc định về trang 1 khi chưa có tiến độ", () => {
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(resumePage(progress)).toBe(1);
  });

  it("trả về lastPage đã lưu", () => {
    useProgressStore.getState().markPageRead(BOOK, 88);
    const progress = getBookProgress(useProgressStore.getState().books, BOOK);
    expect(resumePage(progress)).toBe(88);
  });
});
