import { describe, expect, it } from "vitest";
import { excerpt, pageMetadata } from "@/lib/seo";
import { grammarPattern } from "@/lib/page-grammar";
import { bookLessons, lessonOfPage } from "@/lib/book-lessons";

describe("excerpt", () => {
  it("giữ nguyên đoạn ngắn, bỏ thẻ HTML và khoảng trắng thừa", () => {
    expect(excerpt("<p>Xin   chào</p>\n<b>bạn</b>")).toBe("Xin chào bạn");
  });

  it("cắt ở ranh giới từ và thêm dấu …", () => {
    const out = excerpt("một hai ba bốn năm sáu bảy tám", 20);
    expect(out.length).toBeLessThanOrEqual(20);
    expect(out.endsWith("…")).toBe(true);
    expect(out).toBe("một hai ba bốn năm…");
  });
});

describe("pageMetadata", () => {
  it("gắn canonical theo đúng đường dẫn của trang", () => {
    expect(pageMetadata({ title: "A", path: "/ngu-phap" }).alternates?.canonical).toBe("/ngu-phap");
  });

  it("chỉ gắn noindex khi được yêu cầu", () => {
    expect(pageMetadata({ title: "A", path: "/" }).robots).toBeUndefined();
    expect(pageMetadata({ title: "A", path: "/my", noindex: true }).robots).toEqual({ index: false, follow: true });
  });
});

describe("grammarPattern", () => {
  it("bỏ nhãn từ loại ở đầu", () => {
    expect(grammarPattern("명 이에요/예요")).toBe("이에요/예요");
    expect(grammarPattern("동 형 -아요/어요")).toBe("-아요/어요");
    expect(grammarPattern("-지요?")).toBe("-지요?");
  });
});

describe("bookLessons", () => {
  it("chỉ có bài chứa bản dịch hoặc ngữ pháp", () => {
    const lessons = bookLessons("step1");
    expect(lessons.length).toBeGreaterThan(0);
    for (const l of lessons) expect(l.pages.length).toBeGreaterThan(0);
  });

  it("tìm được bài của một trang", () => {
    expect(lessonOfPage("step1", 18)).toBe(1);
  });
});
