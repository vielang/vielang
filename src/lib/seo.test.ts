import { describe, expect, it } from "vitest";
import { excerpt, pageMetadata } from "@/lib/seo";

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

