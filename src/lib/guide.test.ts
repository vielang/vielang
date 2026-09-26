import { describe, expect, it } from "vitest";
import { GUIDE_SECTIONS, formatUpdated, groupArticles, sectionArticles, type GuideArticle } from "./guide";
import { GUIDE_GROUPS } from "./guide-groups";

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

describe("nội dung cẩm nang", () => {
  // Đọc guide.json đã build (npm run build-content): mục trống thì tab dẫn vào trang trống.
  it.each(GUIDE_SECTIONS.map((s) => [s.id]))("mục %s có ít nhất một bài", (id) => {
    expect(sectionArticles(id).length).toBeGreaterThan(0);
  });

  it("mục có chia nhóm thì bài nào cũng thuộc một nhóm hợp lệ", () => {
    for (const section of GUIDE_SECTIONS) {
      const groups = GUIDE_GROUPS[section.id];
      if (!groups) continue;
      for (const a of sectionArticles(section.id)) {
        expect(groups, `${section.id}/${a.slug}`).toContain(a.facts.group);
      }
    }
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
