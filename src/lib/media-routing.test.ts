import { beforeEach, describe, expect, it, vi } from "vitest";
import { BOOKS, getPageUrl } from "./books";
import { getPageAudio } from "./audio";

const R2 = "https://pub-test.r2.dev";

/**
 * Ảnh và bài nghe CỐ Ý đi hai đường khác nhau. Đây là chỗ đã hỏng thật một
 * lần, nên chốt lại cả hai chiều.
 *
 * Bài nghe đi THẲNG R2: thẻ <audio> tải theo từng khoảng byte (`Range`), mà
 * cache biên của Vercel lấy khoá chỉ theo đường dẫn — nó giữ 2 byte thăm dò
 * đầu tiên rồi trả lại cho mọi khoảng khác. Mã vẫn 206, không lỗi nào bắn
 * ra, và audio chết trên mọi thiết bị.
 *
 * Ảnh trang thì NGƯỢC LẠI, phải đi qua `/img/...` cùng origin: `sw.js` bỏ
 * qua mọi request khác origin, nên trỏ thẳng R2 là mất cả cache ảnh lẫn đọc
 * offline. Ảnh không dính lỗi kia vì nó tải trọn file, không theo khoảng.
 */
beforeEach(() => {
  vi.stubEnv("NEXT_PUBLIC_IMAGE_BASE_URL", R2);
});

function everyAudioUrl(): string[] {
  const urls = new Set<string>();
  for (const book of BOOKS) {
    for (let page = 1; page <= book.totalPages; page++) {
      for (const track of getPageAudio(book.id, page)) urls.add(track.url);
    }
  }
  return [...urls];
}

describe("bài nghe đi thẳng R2", () => {
  it("mọi URL audio đều là địa chỉ R2 tuyệt đối", () => {
    const urls = everyAudioUrl();
    expect(urls.length).toBeGreaterThan(100);
    for (const url of urls) expect(url.startsWith(`${R2}/books/`)).toBe(true);
  });

  it("KHÔNG một URL audio nào đi qua proxy cùng origin", () => {
    // Đây chính là lỗi đã xảy ra: chuyển audio sang `/img/...` cho tiện cache
    // offline, và cache biên nuốt mất mọi khoảng byte.
    for (const url of everyAudioUrl()) expect(url.startsWith("/img/")).toBe(false);
  });
});

describe("ảnh trang vẫn đi qua proxy cùng origin", () => {
  it("URL ảnh là đường tương đối /img/...", () => {
    // Đổi ảnh sang R2 trần là service worker thôi chặn được chúng, và tính
    // năng tải sách về đọc offline chết theo.
    for (const book of BOOKS.slice(0, 4)) {
      expect(getPageUrl(book.id, 15).startsWith("/img/books/")).toBe(true);
    }
  });

  it("ảnh KHÔNG trỏ thẳng sang R2", () => {
    for (const book of BOOKS.slice(0, 4)) {
      expect(getPageUrl(book.id, 15).startsWith("http")).toBe(false);
    }
  });
});
