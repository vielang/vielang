import { describe, expect, it } from "vitest";
import {
  GUIDE_SECTIONS,
  VISA_PATHS,
  getArticle,
  groupArticles,
  schoolRegion,
  schoolTopik,
  schoolTuitionFrom,
  sectionArticles,
} from "./guide";
import { GUIDE_GROUPS } from "./guide-groups";

describe("sơ đồ lộ trình visa", () => {
  it("mọi bước đều trỏ tới một bài có thật", () => {
    // Đổi slug một bài mà quên sửa sơ đồ thì ô đó thành chữ chết — chặn ở đây.
    for (const path of VISA_PATHS) {
      for (const step of path.steps) {
        expect(getArticle(step.sectionId, step.slug), `${path.title} › ${step.code}`).toBeDefined();
      }
    }
  });

  it("mã trên sơ đồ khớp mã của bài visa nó trỏ tới", () => {
    for (const path of VISA_PATHS) {
      for (const step of path.steps.filter((s) => s.sectionId === "visa")) {
        expect(getArticle("visa", step.slug)?.facts.code).toBe(step.code);
      }
    }
  });
});

describe("nhóm bài", () => {
  it("mục có chia nhóm thì bài nào cũng thuộc một nhóm, không sót bài nào", () => {
    for (const section of GUIDE_SECTIONS) {
      if (!GUIDE_GROUPS[section.id]) continue;
      const all = sectionArticles(section.id);
      const grouped = groupArticles(section.id, all).flatMap((g) => g.articles);
      expect(grouped.length).toBe(all.length);
    }
  });

  it("giữ đúng thứ tự nhóm đã định", () => {
    const titles = groupArticles("visa", sectionArticles("visa")).map((g) => g.title);
    const order = GUIDE_GROUPS.visa!;
    expect(titles).toEqual(order.filter((t) => titles.includes(t)));
  });

  it("mục không chia nhóm thì trả về một nhóm chứa tất cả", () => {
    const all = sectionArticles("truong");
    expect(groupArticles("truong", all)).toEqual([{ title: null, articles: all }]);
  });
});

describe("thông số trường cho bộ lọc", () => {
  it("vùng suy từ thành phố", () => {
    expect(schoolRegion("Seoul")).toBe("Seoul");
    expect(schoolRegion("Suwon (Gyeonggi)")).toBe("Gyeonggi – Incheon");
    expect(schoolRegion("Daegu")).toBe("Tỉnh khác");
  });

  it("TOPIK lấy mức chung, bỏ ngoại lệ trong ngoặc", () => {
    expect(schoolTopik("3")).toBe(3);
    expect(schoolTopik("0")).toBe(0);
    expect(schoolTopik("Cấp 3 (ngành thể thao, nghệ thuật: cấp 2)")).toBe(3);
    expect(schoolTopik("Không bắt buộc (một số ngành cần cấp 4)")).toBe(0);
    expect(schoolTopik(undefined)).toBeNull();
  });

  it("học phí lấy mức thấp nhất, dấu phẩy thập phân kiểu Việt", () => {
    expect(schoolTuitionFrom("3,3–5,1 triệu won/học kỳ (năm 2026)")).toBe(3.3);
    expect(schoolTuitionFrom("1,9–2,7 triệu won/học kỳ")).toBe(1.9);
    expect(schoolTuitionFrom(undefined)).toBeNull();
  });

  it("mọi trường đang có đều đọc ra được vùng, TOPIK và học phí", () => {
    // Bài mới ghi thông số theo kiểu lạ thì bộ lọc lặng lẽ bỏ sót trường đó.
    for (const a of sectionArticles("truong")) {
      if (a.facts.topik !== undefined) expect(schoolTopik(a.facts.topik), a.slug).not.toBeNull();
      expect(schoolTuitionFrom(a.facts.tuition), a.slug).not.toBeNull();
    }
  });
});
