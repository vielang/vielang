import { describe, expect, it } from "vitest";
import { TRACKS, activeTrack, trackHref } from "@/lib/tracks";
import { activeTab } from "@/lib/nav";
import { BOOKS } from "@/lib/books";
import { getSeries } from "@/lib/series";

describe("tracks", () => {
  it("IT là mảng mặc định ở trang gốc, bên cạnh các ngôn ngữ", () => {
    const it = TRACKS.find((t) => t.slug === "");
    expect(it).toMatchObject({ label: "IT", kind: "courses" });
    expect(TRACKS.some((t) => t.slug === "en" && t.kind === "books")).toBe(true);
  });

  it("mảng mặc định sống ở route gốc", () => {
    expect(trackHref("")).toBe("/");
    expect(trackHref("en")).toBe("/en");
    expect(activeTrack("/")?.slug).toBe("");
  });

  it("nhận ra mảng IT ở cả trang khoá và trang học bài", () => {
    expect(activeTrack("/it")?.slug).toBe("");
    expect(activeTrack("/it/csharp-core")?.slug).toBe("");
    expect(activeTrack("/learn/csharp-core/nen-tang/bien")?.slug).toBe("");
  });

  it("trang sách tô đúng ngôn ngữ của cuốn đó, không lọt sang IT", () => {
    expect(activeTrack("/books/en-elementary")?.slug).toBe("en");
    expect(activeTrack("/en")?.slug).toBe("en");
  });
});

describe("activeTab", () => {
  it("các trang IT nằm trong tab Thư viện", () => {
    for (const path of ["/it", "/it/csharp-core", "/learn/csharp-core/nen-tang/bien"]) {
      expect(activeTab(path)?.href).toBe("/");
    }
  });

  it("không đổi hành vi của các tab khác", () => {
    expect(activeTab("/")?.href).toBe("/");
    expect(activeTab("/en")?.href).toBe("/");
    expect(activeTab("/my")?.href).toBe("/my");
  });
});

describe("mục con đang mở", () => {
  it("sách tiếng Anh tô Tiếng Anh", () => {
    expect(activeTab("/en")?.activeChild?.("/en")).toBe("/en");
    expect(activeTab("/books/en-elementary")?.activeChild?.("/books/en-elementary")).toBe("/en");
  });
});

describe("bộ sách", () => {
  it("trang một bộ sách thuộc Thư viện, tô đúng ngôn ngữ", () => {
    expect(activeTab("/sach/english-file")?.href).toBe("/");
    expect(activeTrack("/sach/english-file")?.slug).toBe("en");
    expect(activeTrack("/sach/khong-co")?.slug).toBe("");
  });

  it("ngôn ngữ chỉ có một bộ thì link bộ sách trỏ về trang ngôn ngữ", () => {
    const library = activeTab("/")!;
    const english = library.children!.find((c) => c.label === "Tiếng Anh")!;
    expect(english.items?.map((i) => i.href)).toEqual(["/en"]);
  });

  it("sách nào cũng thuộc một bộ có thật, cùng ngôn ngữ", () => {
    for (const book of BOOKS) {
      expect(getSeries(book.series)?.lang).toBe(book.lang);
    }
  });
});
