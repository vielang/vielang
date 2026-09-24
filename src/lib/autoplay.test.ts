import { describe, expect, it } from "vitest";
import { buildPlaylist, startIndexFor } from "./autoplay";
import { getAudioPages } from "./audio";

describe("buildPlaylist", () => {
  it("đi theo thứ tự trang, trong trang theo thứ tự track", () => {
    const queue = buildPlaylist("step1");
    expect(queue[0]).toMatchObject({ page: 1, type: "intro" });
    // Bài 1 bắt đầu trang 12: "말하기와 듣기" ở trang 18 có S rồi L, "발음" ở trang 21.
    expect(queue.slice(1, 4).map((t) => [t.page, t.type])).toEqual([
      [18, "S"],
      [18, "L"],
      [21, "P"],
    ]);
    const pages = queue.map((t) => t.page);
    expect(pages).toEqual([...pages].sort((a, b) => a - b));
  });

  it("phủ đủ mọi trang có bài nghe", () => {
    for (const bookId of ["step1", "wb-step3", "en-elementary"]) {
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
  const queue = buildPlaylist("step1");

  it("bắt đầu từ bài đầu tiên ở trang này hoặc sau đó", () => {
    expect(queue[startIndexFor(queue, 18)]).toMatchObject({ page: 18, type: "S" });
    expect(queue[startIndexFor(queue, 19)]).toMatchObject({ page: 21, type: "P" });
  });

  it("đã qua bài cuối thì quay về đầu sách", () => {
    expect(startIndexFor(queue, 9999)).toBe(0);
  });
});
