import { beforeEach, describe, expect, it } from "vitest";
import { useProgressStore } from "./progress-store";
import { useQuizStore } from "./quiz-store";
import { useReaderPrefsStore } from "./reader-prefs-store";
import { useActivityStore } from "./activity-store";

/** Giả lập tab khác ghi xuống localStorage rồi trình duyệt báo sang tab này. */
async function otherTabWrites(key: string, state: unknown) {
  const newValue = JSON.stringify({ state, version: 0 });
  localStorage.setItem(key, newValue);
  window.dispatchEvent(new StorageEvent("storage", { key, newValue }));
  // `rehydrate` với localStorage chạy đồng bộ nhưng trả về promise.
  await Promise.resolve();
}

beforeEach(() => {
  localStorage.clear();
  useProgressStore.setState({ books: {} });
  useQuizStore.setState({ pages: {} });
  useReaderPrefsStore.setState({ hiddenAnswerBooks: [] });
});

describe("đồng bộ giữa các tab", () => {
  it("tiến độ đọc: tab này thấy trang tab kia vừa đọc", async () => {
    await otherTabWrites("kiip-progress-v1", {
      books: { step1: { lastPage: 30, readPages: [29, 30], bookmarks: [], updatedAt: "x" } },
    });

    expect(useProgressStore.getState().books.step1?.lastPage).toBe(30);
  });

  it("…và lần ghi kế tiếp của tab này GIỮ dữ liệu tab kia, không ghi đè", async () => {
    await otherTabWrites("kiip-progress-v1", {
      books: { step1: { lastPage: 30, readPages: [30], bookmarks: [], updatedAt: "x" } },
    });
    useProgressStore.getState().markPageRead("step2", 5);
    const saved = JSON.parse(localStorage.getItem("kiip-progress-v1")!).state.books;

    expect(Object.keys(saved).sort()).toEqual(["step1", "step2"]);
  });

  it("bài làm quiz", async () => {
    await otherTabWrites("kiip-quiz-v1", {
      pages: { "step1:19": { answers: { a: 1 }, checked: ["a"] } },
    });

    expect(useQuizStore.getState().pages["step1:19"]?.checked).toEqual(["a"]);
  });

  it("tuỳ chọn trang đọc", async () => {
    await otherTabWrites("kiip-reader-prefs-v1", {
      pageLayout: "double",
      hasSeenReaderHelp: true,
      hiddenAnswerBooks: ["wb-step1"],
    });

    expect(useReaderPrefsStore.getState().hiddenAnswerBooks).toEqual(["wb-step1"]);
  });

  it("tab kia xoá sạch dữ liệu thì tab này cũng xoá, không khôi phục bản cũ", async () => {
    // `rehydrate` mà kho trống sẽ ghi trả bản đang giữ trong bộ nhớ xuống —
    // tức là hồi sinh đúng thứ người dùng vừa cố ý xoá.
    useActivityStore.setState({ studied: { step1: [1] } });
    localStorage.clear();
    window.dispatchEvent(new StorageEvent("storage", { key: null, newValue: null }));
    await Promise.resolve();

    expect(useActivityStore.getState().studied).toEqual({});
    expect(localStorage.getItem("kiip-activity-v1") ?? "").not.toContain("step1");
  });

  it("khoá của store khác thì không nạp lại lung tung", async () => {
    useQuizStore.setState({ pages: { "step1:18": { answers: {}, checked: ["x"] } } });
    localStorage.setItem("kiip-notes-v1", "{}");
    window.dispatchEvent(new StorageEvent("storage", { key: "kiip-notes-v1", newValue: "{}" }));
    await Promise.resolve();

    expect(useQuizStore.getState().pages["step1:18"]?.checked).toEqual(["x"]);
  });
});
