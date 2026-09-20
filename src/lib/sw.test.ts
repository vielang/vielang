import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Kiểm luật phân loại của service worker.
 *
 * Service worker là thứ hỏng thì hỏng dai — nó sống qua cả lần tải lại
 * trang, nên phân loại sai một đường là người dùng có thể bị khoá ở phiên
 * bản cũ. Không có cách nào chạy thử nó trong jsdom, nhưng phần quyết định
 * "đường này đi cache kiểu gì" thì thuần tuý và kiểm được.
 *
 * Nạp thẳng file nguồn rồi lấy ra hai hàm phân loại, thay vì chép luật sang
 * đây — chép sang là hai bản trôi khỏi nhau lúc nào không biết.
 */
function loadRules() {
  const src = readFileSync(
    path.resolve(process.cwd(), "public", "sw.js"),
    "utf8"
  );
  const stubSelf = { addEventListener: () => {}, location: { origin: "https://x" } };
  const factory = new Function(
    "self",
    "caches",
    `${src}\nreturn { isPageImage, isImmutableAsset, VERSION, MAX_IMAGES, OFFLINE_URL };`
  );
  return factory(stubSelf, { open: () => {}, keys: () => [] }) as {
    isPageImage: (url: URL) => boolean;
    isImmutableAsset: (url: URL) => boolean;
    VERSION: string;
    MAX_IMAGES: number;
    OFFLINE_URL: string;
  };
}

const rules = loadRules();
const url = (p: string) => new URL(p, "https://kiip.example");

describe("ảnh trang sách", () => {
  it("nhận ảnh đi qua bộ tối ưu của Next", () => {
    expect(rules.isPageImage(url("/_next/image?url=%2Fx.webp&w=1920&q=90"))).toBe(true);
  });

  it("nhận ảnh theo đuôi tệp", () => {
    for (const p of ["/a.webp", "/a.avif", "/a.png", "/a.jpg", "/a.JPEG"]) {
      expect(rules.isPageImage(url(p))).toBe(true);
    }
  });

  it("không nhận tài liệu hay dữ liệu", () => {
    for (const p of ["/read/step1/10", "/books/step1", "/manifest.webmanifest"]) {
      expect(rules.isPageImage(url(p))).toBe(false);
    }
  });
});

describe("tài nguyên bất biến", () => {
  it("nhận chunk của Next — tên có băm nội dung nên cache trước là an toàn", () => {
    expect(rules.isImmutableAsset(url("/_next/static/chunks/abc123.js"))).toBe(true);
  });

  it("nhận phông Excalidraw tự phục vụ", () => {
    expect(rules.isImmutableAsset(url("/excalidraw/fonts/Excalifont/a.woff2"))).toBe(true);
  });

  it("KHÔNG nhận tài liệu HTML", () => {
    // Đây là luật quan trọng nhất trong cả file: HTML mà rơi vào nhánh cache
    // trước thì người dùng kẹt ở bản cũ, deploy bao nhiêu lần cũng không thấy.
    for (const p of ["/", "/read/step1/10", "/bookmarks", "/offline"]) {
      expect(rules.isImmutableAsset(url(p))).toBe(false);
    }
  });

  it("KHÔNG nhận ảnh — ảnh có cache riêng còn giới hạn số lượng", () => {
    expect(rules.isImmutableAsset(url("/_next/image?url=x"))).toBe(false);
  });
});

describe("cấu hình", () => {
  it("có trang dự phòng lúc mất mạng", () => {
    expect(rules.OFFLINE_URL).toBe("/offline");
  });

  it("giới hạn số ảnh giữ lại — ảnh trang sách nặng vài trăm KB mỗi tấm", () => {
    expect(rules.MAX_IMAGES).toBeGreaterThan(0);
    expect(rules.MAX_IMAGES).toBeLessThanOrEqual(1000);
  });

  it("tên cache có gắn phiên bản để dọn được bản cũ", () => {
    expect(rules.VERSION).toMatch(/^v\d+$/);
  });
});

describe("không tự tiếp quản giữa chừng", () => {
  it("không gọi skipWaiting", () => {
    // Tráo service worker khi trang đang mở có thể khiến nó đi xin chunk của
    // phiên bản khác. Bản mới chờ tới khi mọi tab đóng lại.
    // Tìm lời GỌI chứ không tìm chữ — chính comment trong sw.js cũng nhắc
    // tới tên hàm này để giải thích vì sao không dùng.
    const src = readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");
    expect(src).not.toMatch(/self\.skipWaiting\s*\(/);
  });
});

describe("sách tải chủ động", () => {
  const src = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");

  it("dùng đúng tiền tố cache mà bên tải về đang ghi", () => {
    // Hai file không import được nhau (một bên là script thô trong public),
    // nên lệch tiền tố là offline im lặng không hoạt động.
    const lib = readFileSync(
      path.resolve(process.cwd(), "src", "lib", "offline-books.ts"),
      "utf8"
    );
    const prefix = lib.match(/BOOK_CACHE_PREFIX = "([^"]+)"/)?.[1];

    expect(prefix).toBe("kiip-book-");
    expect(src()).toContain(`BOOK_CACHE_PREFIX = "${prefix}"`);
  });

  it("KHÔNG dọn cache sách khi nâng cấp service worker", () => {
    // Dọn nhầm là người dùng mất cả cuốn vừa tải 60MB mà không hiểu vì sao.
    expect(src()).toMatch(/keep\.has\(n\) \|\| n\.startsWith\(BOOK_CACHE_PREFIX\)/);
  });

  it("tra sách đã tải TRƯỚC cả cache cơ hội lẫn mạng", () => {
    const body = src();
    const inCacheFirst = body.indexOf("async function cacheFirst");
    const lookup = body.indexOf("matchDownloadedBook(request)", inCacheFirst);
    const opportunistic = body.indexOf("caches.open(cacheName)", inCacheFirst);

    expect(lookup).toBeGreaterThan(inCacheFirst);
    expect(lookup).toBeLessThan(opportunistic);
  });

  it("mất mạng thì tài liệu cũng tra sách đã tải", () => {
    const body = src();
    const inNetworkFirst = body.indexOf("async function networkFirst");
    expect(body.indexOf("matchDownloadedBook", inNetworkFirst)).toBeGreaterThan(
      inNetworkFirst
    );
  });
});
