import { describe, expect, it } from "vitest";
import { buildPlaylist, startIndexFor } from "./autoplay";
import { getAudioPages } from "./audio";

describe("buildPlaylist", () => {
  it("đi theo thứ tự trang, trong trang theo thứ tự track", () => {
    const queue = buildPlaylist("en-elementary");
    // Trang 7 có "1.2" rồi "1.3", trang 8 bắt đầu bằng "1.5".
    expect(queue.slice(0, 3).map((t) => [t.page, t.type])).toEqual([
      [7, "1.2"],
      [7, "1.3"],
      [8, "1.5"],
    ]);
    const pages = queue.map((t) => t.page);
    expect(pages).toEqual([...pages].sort((a, b) => a - b));
  });

  it("phủ đủ mọi trang có bài nghe", () => {
    for (const bookId of ["en-elementary", "en-pre-intermediate"]) {
      const pages = new Set(buildPlaylist(bookId).map((t) => t.page));
      expect([...pages]).toEqual(getAudioPages(bookId));
    }
  });

  it("không phát 2 lần một track in ở 2 trang liền nhau", () => {
    const queue = buildPlaylist("en-pre-intermediate");
    const urls = queue.map((t) => t.url);
    expect(new Set(urls).size).toBe(urls.length);
    // "2.16" in ở cả trang 18 và 19 — giữ lần đầu, ở trang 18.
    expect(queue.filter((t) => t.type === "2.16").map((t) => t.page)).toEqual([18]);
  });

  it("sách không có bài nghe thì danh sách rỗng", () => {
    expect(buildPlaylist("khong-co-sach-nay")).toEqual([]);
  });
});

describe("startIndexFor", () => {
  const queue = buildPlaylist("en-elementary");

  it("bắt đầu từ bài đầu tiên ở trang này hoặc sau đó", () => {
    expect(queue[startIndexFor(queue, 8)]).toMatchObject({ page: 8, type: "1.5" });
    // Trang 15 không có bài nghe — nhảy tới trang 16.
    expect(queue[startIndexFor(queue, 15)]).toMatchObject({ page: 16, type: "2.3" });
  });

  it("đã qua bài cuối thì quay về đầu sách", () => {
    expect(startIndexFor(queue, 9999)).toBe(0);
  });
});
