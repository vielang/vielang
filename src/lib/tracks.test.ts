import { describe, expect, it } from "vitest";
import { TRACKS, activeTrack, trackHref } from "@/lib/tracks";
import { activeTab } from "@/lib/nav";
import { BOOKS } from "@/lib/books";
import { getSeries } from "@/lib/series";

describe("tracks", () => {
  it("có tab IT bên cạnh các ngôn ngữ", () => {
    const it = TRACKS.find((t) => t.slug === "it");
    expect(it).toMatchObject({ label: "IT", kind: "courses" });
    expect(TRACKS.some((t) => t.slug === "" && t.kind === "books")).toBe(true);
  });

  it("mảng mặc định sống ở route gốc", () => {
    expect(trackHref("")).toBe("/");
    expect(trackHref("it")).toBe("/it");
    expect(activeTrack("/")?.slug).toBe("");
  });

  it("nhận ra mảng IT ở cả trang khoá và trang học bài", () => {
    expect(activeTrack("/it")?.slug).toBe("it");
    expect(activeTrack("/it/csharp-core")?.slug).toBe("it");
    expect(activeTrack("/learn/csharp-core/nen-tang/bien")?.slug).toBe("it");
  });

  it("trang sách vẫn thuộc mảng mặc định, không lọt sang IT", () => {
    expect(activeTrack("/books/level1")?.slug).toBe("");
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
    expect(activeTab("/exam/102-topik1/practice")?.href).toBe("/exam/topik-i");
    expect(activeTab("/my")?.href).toBe("/my");
  });
});

describe("mục con đang mở", () => {
  it("sách tiếng Anh tô Tiếng Anh, đề thi tô ngôn ngữ của kỳ thi", () => {
    expect(activeTab("/en")?.activeChild?.("/en")).toBe("/en");
    expect(activeTab("/exam/102-topik2/practice")?.activeChild?.("/exam/102-topik2/practice")).toBe("/exam/topik-i");
    expect(activeTab("/cam-nang/visa/e7")?.activeChild?.("/cam-nang/visa/e7")).toBe("/cam-nang/visa");
  });
});

describe("bộ sách", () => {
  it("trang một bộ sách thuộc Thư viện, tô đúng ngôn ngữ", () => {
    expect(activeTab("/sach/english-file")?.href).toBe("/");
    expect(activeTrack("/sach/english-file")?.slug).toBe("en");
    expect(activeTrack("/sach/kiip")?.slug).toBe("");
  });

  it("ngôn ngữ chỉ có một bộ thì link bộ sách trỏ về trang ngôn ngữ", () => {
    const library = activeTab("/")!;
    const korean = library.children!.find((c) => c.label === "Tiếng Hàn")!;
    expect(korean.items?.map((i) => i.href)).toEqual(["/"]);
  });

  it("sách nào cũng thuộc một bộ có thật, cùng ngôn ngữ", () => {
    for (const book of BOOKS) {
      expect(getSeries(book.series)?.lang).toBe(book.lang);
    }
  });
});
