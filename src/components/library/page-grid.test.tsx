import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { PageGrid } from "./page-grid";
import { useProgressStore } from "@/lib/progress-store";
import { useReaderPrefsStore } from "@/lib/reader-prefs-store";

/**
 * Hàng huy hiệu ở góc trên mỗi ô trang là cách DUY NHẤT để biết trang nào có
 * gì trước khi mở ra. Sót một loại là cả một tính năng tàng hình ở màn hình
 * này — người dùng phải lật từng trang mới biết trang nào có.
 */
const BASE = {
  bookId: "step1",
  totalPages: 3,
  chapters: [],
  notePages: [] as number[],
  audioPages: [] as number[],
  translatedPages: [] as number[],
  grammarPages: [] as number[],
  answerPages: [] as number[],
};

function grid(overrides: Partial<typeof BASE> = {}) {
  return render(<PageGrid {...BASE} {...overrides} />);
}

beforeEach(() => useProgressStore.setState({ books: {} }));

describe("huy hiệu ngữ pháp", () => {
  it("hiện ở đúng trang có giải nghĩa ngữ pháp", () => {
    grid({ grammarPages: [2] });

    expect(screen.getAllByTitle("Có giải nghĩa ngữ pháp")).toHaveLength(1);
  });

  it("không hiện ở trang không có", () => {
    grid({ grammarPages: [] });

    expect(screen.queryByTitle("Có giải nghĩa ngữ pháp")).toBeNull();
  });

  it("đứng cạnh được với các huy hiệu khác trên cùng một trang", () => {
    // Trang ngữ pháp cũng có thể có audio hoặc bài giảng. Điều kiện hiện
    // hàng huy hiệu phải tính cả ngữ pháp, nếu không thì trang CHỈ có ngữ
    // pháp sẽ không hiện gì cả.
    grid({ grammarPages: [1], audioPages: [1], notePages: [1] });

    expect(screen.getByTitle("Có giải nghĩa ngữ pháp")).toBeTruthy();
    expect(screen.getByTitle("Có audio")).toBeTruthy();
    expect(screen.getByTitle("Có bài giảng")).toBeTruthy();
  });

  it("trang CHỈ có ngữ pháp vẫn hiện huy hiệu", () => {
    // Chốt riêng vì đây đúng là chỗ dễ sót: quên thêm `hasGrammar` vào điều
    // kiện hiện hàng thì huy hiệu chỉ xuất hiện khi trang tình cờ có thứ
    // khác đi kèm.
    grid({ grammarPages: [3] });

    expect(screen.getByTitle("Có giải nghĩa ngữ pháp")).toBeTruthy();
  });
});

describe("các huy hiệu sẵn có", () => {
  it("vẫn chạy như cũ", () => {
    grid({ notePages: [1], audioPages: [2], translatedPages: [3] });

    expect(screen.getByTitle("Có bài giảng")).toBeTruthy();
    expect(screen.getByTitle("Có audio")).toBeTruthy();
    expect(screen.getByTitle("Có bản dịch tiếng Việt")).toBeTruthy();
  });
});

describe("huy hiệu đáp án sách", () => {
  it("hiện ở đúng trang có đáp án, kể cả khi trang không có gì khác", () => {
    grid({ answerPages: [2] });

    expect(screen.getAllByTitle("Có đáp án sách")).toHaveLength(1);
  });

  it("không hiện ở trang không có", () => {
    grid({ answerPages: [] });

    expect(screen.queryByTitle("Có đáp án sách")).toBeNull();
  });
});

describe("huy hiệu đáp án khi người dùng tắt đáp án của cuốn này", () => {
  it("thôi hiện — nó hứa một thứ người dùng sẽ không thấy trên trang", () => {
    useReaderPrefsStore.setState({ hiddenAnswerBooks: ["step1"] });
    grid({ answerPages: [2] });

    expect(screen.queryByTitle("Có đáp án sách")).toBeNull();
    useReaderPrefsStore.setState({ hiddenAnswerBooks: [] });
  });

  it("tắt cuốn khác thì cuốn này vẫn hiện", () => {
    useReaderPrefsStore.setState({ hiddenAnswerBooks: ["wb-step1"] });
    grid({ answerPages: [2] });

    expect(screen.getByTitle("Có đáp án sách")).toBeTruthy();
    useReaderPrefsStore.setState({ hiddenAnswerBooks: [] });
  });
});
