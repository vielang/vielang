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
    `${src}\nreturn { isImage, isImmutableAsset, VERSION, MAX_IMAGES, OFFLINE_URL };`
  );
  return factory(stubSelf, { open: () => {}, keys: () => [] }) as {
    isImage: (url: URL) => boolean;
    isImmutableAsset: (url: URL) => boolean;
    VERSION: string;
    MAX_IMAGES: number;
    OFFLINE_URL: string;
  };
}

const rules = loadRules();
const url = (p: string) => new URL(p, "https://kiip.example");

describe("ảnh", () => {
  it("nhận ảnh đi qua bộ tối ưu của Next", () => {
    expect(rules.isImage(url("/_next/image?url=%2Fx.webp&w=1920&q=90"))).toBe(true);
  });

  it("nhận ảnh theo đuôi tệp", () => {
    for (const p of ["/a.webp", "/a.avif", "/a.png", "/a.jpg", "/a.JPEG"]) {
      expect(rules.isImage(url(p))).toBe(true);
    }
  });

  it("không nhận tài liệu hay dữ liệu", () => {
    for (const p of ["/learn/csharp-core/x/y", "/it/csharp-core", "/manifest.webmanifest"]) {
      expect(rules.isImage(url(p))).toBe(false);
    }
  });
});

describe("tài nguyên bất biến", () => {
  it("nhận chunk của Next — tên có băm nội dung nên cache trước là an toàn", () => {
    expect(rules.isImmutableAsset(url("/_next/static/chunks/abc123.js"))).toBe(true);
  });

  it("KHÔNG nhận tài liệu HTML", () => {
    // Đây là luật quan trọng nhất trong cả file: HTML mà rơi vào nhánh cache
    // trước thì người dùng kẹt ở bản cũ, deploy bao nhiêu lần cũng không thấy.
    for (const p of ["/", "/learn/csharp-core/x/y", "/my", "/offline"]) {
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

  it("giới hạn số ảnh giữ lại", () => {
    expect(rules.MAX_IMAGES).toBeGreaterThan(0);
    expect(rules.MAX_IMAGES).toBeLessThanOrEqual(1000);
  });

  it("tên cache có gắn phiên bản để dọn được bản cũ", () => {
    expect(rules.VERSION).toMatch(/^v\d+$/);
  });
});

describe("không TỰ tiếp quản giữa chừng", () => {
  const swSrc = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");

  it("không nhường chỗ ngay lúc cài", () => {
    // Tráo service worker khi trang đang mở có thể khiến nó đi xin chunk của
    // phiên bản khác. Lúc cài thì tuyệt đối không được tự nhường.
    const body = swSrc();
    const install = body.indexOf('addEventListener("install"');
    const activate = body.indexOf('addEventListener("activate"');

    expect(body.slice(install, activate)).not.toMatch(/skipWaiting\s*\(/);
  });

  it("chỉ nhường chỗ khi người dùng bấm cập nhật", () => {
    // Người dùng bấm thì trang được tải lại ngay sau đó nên an toàn — xem
    // components/service-worker.tsx.
    const body = swSrc();
    const message = body.indexOf('addEventListener("message"');
    const skip = body.search(/self\.skipWaiting\s*\(/);

    expect(message).toBeGreaterThan(-1);
    expect(skip).toBeGreaterThan(message);
    expect(body.slice(message, skip)).toContain("SKIP_WAITING");
  });

  it("giao diện tải lại trang khi service worker đổi", () => {
    // Thiếu bước này thì bấm "Cập nhật" xong trang CŨ vẫn chạy với service
    // worker MỚI — đúng cảnh lệch phiên bản mà cả thiết kế này né tránh.
    const ui = readFileSync(
      path.resolve(process.cwd(), "src", "components", "service-worker.tsx"),
      "utf8"
    );

    expect(ui).toContain("controllerchange");
    expect(ui).toContain("window.location.reload()");
  });
});

describe("dọn cache cũ", () => {
  it("lên phiên bản mới thì xoá mọi cache không còn dùng", () => {
    const body = readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");
    expect(body).toMatch(/keep\.has\(n\) \? null : caches\.delete\(n\)/);
  });
});

/**
 * Vercel Analytics tải script từ chính origin của app (`/_vercel/insights/`).
 * Service worker này chặn MỌI request GET cùng origin, nên ranh giới đó phải
 * được giữ có chủ đích chứ không phải may mà đúng.
 *
 * Hỏng ở đây lại hỏng lặng lẽ: trang vẫn chạy, không lỗi nào bắn ra, chỉ có
 * số liệu là đứng im — mà đứng im thì nhìn y hệt "chưa ai vào xem".
 */
describe("đường của Vercel Analytics", () => {
  const src = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");
  const vercel = url("/_vercel/insights/script.js");

  it("không bị xếp vào tài nguyên cache-trước", () => {
    // Xếp nhầm vào đây là script analytics bị đóng băng ở bản đầu tiên tải
    // được, không bao giờ cập nhật nữa.
    expect(rules.isImmutableAsset(vercel)).toBe(false);
  });

  it("không bị nhầm là ảnh", () => {
    expect(rules.isImage(vercel)).toBe(false);
  });

  it("beacon là POST nên service worker buông ngay từ đầu", () => {
    // Chốt duy nhất giữ cho beacon đi thẳng ra mạng. Bỏ dòng này đi là mọi
    // lượt xem trang đều chui qua service worker.
    const body = src();
    const handler = body.indexOf('self.addEventListener("fetch"');
    const guard = body.indexOf('request.method !== "GET"', handler);
    const firstRespond = body.indexOf("event.respondWith", handler);

    expect(guard).toBeGreaterThan(handler);
    expect(guard).toBeLessThan(firstRespond);
  });
});
