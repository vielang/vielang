import { describe, expect, it } from "vitest";
import { formatUpdated, groupArticles, type GuideArticle } from "./guide";

const article = (slug: string, group?: string): GuideArticle => ({
  slug,
  title: slug,
  summary: "",
  updated: "2026-09-27",
  facts: group ? { group } : {},
  html: "",
  headings: [],
  checks: 0,
});

describe("nhóm bài", () => {
  it("mục không chia nhóm thì trả về một nhóm chứa tất cả", () => {
    const all = [article("a"), article("b")];
    expect(groupArticles("khong-co-nhom", all)).toEqual([{ title: null, articles: all }]);
  });
});

describe("ngày cập nhật", () => {
  it("đổi YYYY-MM-DD sang dd/mm/yyyy", () => {
    expect(formatUpdated("2026-09-27")).toBe("27/09/2026");
  });

  it("giữ nguyên chuỗi không đúng định dạng", () => {
    expect(formatUpdated("tháng 9")).toBe("tháng 9");
  });
});
