import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it, vi } from "vitest";

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

describe("payload RSC (lật trang trong app)", () => {
  const src = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");

  function rules() {
    const stubSelf = { addEventListener: () => {}, location: { origin: "https://x" } };
    const factory = new Function(
      "self",
      "caches",
      `${src()}\nreturn { isRscRequest, rscCacheKey };`
    );
    return factory(stubSelf, {}) as {
      isRscRequest: (req: { headers: Headers }, url: URL) => boolean;
      rscCacheKey: (url: URL) => string;
    };
  }

  const req = (headers: Record<string, string> = {}) => ({
    headers: new Headers(headers),
  });

  it("nhận yêu cầu mang header rsc", () => {
    expect(rules().isRscRequest(req({ rsc: "1" }), url("/read/step1/11"))).toBe(true);
  });

  it("nhận cả dạng đã bị 307 sang ?_rsc", () => {
    // Next trả 307 từ `rsc: 1` sang `?_rsc=<băm>` — tuỳ lúc ta bắt được cái nào.
    expect(rules().isRscRequest(req(), url("/read/step1/11?_rsc=abc"))).toBe(true);
  });

  it("không nhận nhầm yêu cầu thường", () => {
    expect(rules().isRscRequest(req(), url("/read/step1/11"))).toBe(false);
  });

  it("khoá cache bỏ hết tham số, khớp hai đầu", () => {
    // Băm `_rsc` đổi theo trạng thái router nên không dùng làm khoá được.
    const { rscCacheKey } = rules();
    expect(rscCacheKey(url("/read/step1/11?_rsc=abc"))).toBe(
      "/read/step1/11?__offline_rsc=1"
    );
    expect(rscCacheKey(url("/read/step1/11?_rsc=xyz"))).toBe(
      rscCacheKey(url("/read/step1/11"))
    );
  });

  it("khoá khớp đúng hàm bên tải về", () => {
    const lib = readFileSync(
      path.resolve(process.cwd(), "src", "lib", "offline-books.ts"),
      "utf8"
    );
    const suffix = lib.match(/\?__offline_rsc=1/)?.[0];

    expect(suffix).toBe("?__offline_rsc=1");
    expect(src()).toContain("?__offline_rsc=1");
  });

  it("đi MẠNG TRƯỚC, cache chỉ là lưới đỡ", () => {
    // Bản RSC khai `Vary: next-router-state-tree` — nội dung đổi theo chỗ
    // router đang đứng, nên đem bản cũ ra dùng lúc còn mạng là chuốc lỗi.
    const body = src();
    const fn = body.indexOf("async function rscNetworkFirst");
    const network = body.indexOf("await fetch(request)", fn);
    const fallback = body.indexOf("matchDownloadedBook", fn);

    expect(network).toBeGreaterThan(fn);
    expect(network).toBeLessThan(fallback);
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

  it("không bị nhầm là ảnh trang sách", () => {
    expect(rules.isPageImage(vercel)).toBe(false);
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

/**
 * Bài nghe của cuốn đã tải về máy.
 *
 * Phần cắt khoảng (`Range`) là thứ quyết định iPhone có nghe được hay không,
 * nên chạy hàm thật chứ không chỉ dò chữ trong file.
 */
describe("bài nghe offline", () => {
  const src = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");

  /** Bản trả về 1000 byte, giả như đã nằm sẵn trong cache. */
  const cached = () =>
    new Response(new Uint8Array(1000), {
      headers: { "Content-Type": "audio/mpeg" },
    });

  /**
   * Nạp service worker với một Cache API giả.
   *
   * `downloaded = false` là giả cảnh chưa tải cuốn nào về.
   */
  function audioRules(downloaded = true) {
    const stubSelf = { addEventListener: () => {}, location: { origin: "https://x" } };
    const stubCaches = {
      keys: async () => (downloaded ? ["kiip-book-step1"] : []),
      open: async () => ({ match: async () => (downloaded ? cached() : undefined) }),
    };
    const factory = new Function(
      "self",
      "caches",
      `${src()}\nreturn { isAudio, sliceForRange, serveAudio };`
    );
    return factory(stubSelf, stubCaches) as {
      isAudio: (url: URL) => boolean;
      sliceForRange: (res: Response, range: string) => Promise<Response>;
      serveAudio: (req: Request) => Promise<Response>;
    };
  }

  const { isAudio, sliceForRange } = audioRules();

  it("nhận ra file mp3, không nhầm với ảnh trang", () => {
    expect(isAudio(url("/img/books/step1/audio/1-S.mp3"))).toBe(true);
    expect(isAudio(url("/img/books/step1/pages/0020.webp"))).toBe(false);
  });

  it("đòi khoảng thì trả 206 kèm Content-Range", async () => {
    // Safari/iOS từ chối phát hẳn nếu đòi khoảng mà nhận về 200 — tức là
    // sách đã tải về vẫn không nghe được trên iPhone.
    const res = await sliceForRange(cached(), "bytes=0-99");

    expect(res.status).toBe(206);
    expect(res.headers.get("Content-Range")).toBe("bytes 0-99/1000");
    expect(res.headers.get("Content-Length")).toBe("100");
    expect((await res.arrayBuffer()).byteLength).toBe(100);
  });

  it("tua tới giữa bài (khoảng hở đuôi) thì lấy tới hết file", async () => {
    const res = await sliceForRange(cached(), "bytes=500-");

    expect(res.headers.get("Content-Range")).toBe("bytes 500-999/1000");
    expect((await res.arrayBuffer()).byteLength).toBe(500);
  });

  it("đòi quá cỡ file thì cắt về đúng mép, không vỡ", async () => {
    const res = await sliceForRange(cached(), "bytes=900-99999");

    expect(res.headers.get("Content-Range")).toBe("bytes 900-999/1000");
    expect((await res.arrayBuffer()).byteLength).toBe(100);
  });

  it("dạng lạ thì trả nguyên bản chứ không đoán bừa", async () => {
    // `bytes=-100` là "100 byte cuối" — thẻ <audio> không dùng dạng này,
    // đoán bừa thì sai còn tệ hơn nhường cho mạng.
    const res = await sliceForRange(cached(), "bytes=-100");

    expect(res.status).toBe(200);
  });

  it("KHÔNG tự cache bài nghe nghe lướt", () => {
    // Một bài nghe 1,3MB, gấp hơn hai chục lần một trang. Tự cache là đầy
    // máy người ta mà họ không hề xin.
    //
    // Dò `.put(` chứ không dò `cache.put`: đổi tên biến một cái là lọt.
    const body = src();
    const fn = body.indexOf("async function serveAudio");
    const next = body.indexOf("async function sliceForRange");
    const region = body.slice(fn, next);

    expect(region).toContain("matchDownloadedBook");
    expect(region).not.toContain(".put(");
  });
});

/**
 * `serveAudio` chạy thật, với Cache API giả.
 *
 * Tách khỏi nhóm trên vì nhóm đó chỉ kiểm từng hàm rời. Kiểm rời thôi là
 * chưa đủ: gỡ đúng dòng nối `serveAudio` vào `sliceForRange` thì mọi test
 * rời vẫn xanh, mà trên iPhone thì sách đã tải về không nghe được.
 */
describe("phục vụ bài nghe từ cuốn đã tải", () => {
  const src = () =>
    readFileSync(path.resolve(process.cwd(), "public", "sw.js"), "utf8");

  const cached = () =>
    new Response(new Uint8Array(1000), {
      headers: { "Content-Type": "audio/mpeg" },
    });

  function load(downloaded: boolean) {
    const stubSelf = { addEventListener: () => {}, location: { origin: "https://x" } };
    const stubCaches = {
      keys: async () => (downloaded ? ["kiip-book-step1"] : []),
      open: async () => ({ match: async () => (downloaded ? cached() : undefined) }),
    };
    const factory = new Function("self", "caches", `${src()}\nreturn { serveAudio };`);
    return factory(stubSelf, stubCaches) as {
      serveAudio: (req: Request) => Promise<Response>;
    };
  }

  const audioRequest = (range?: string) =>
    new Request("https://x/img/books/step1/audio/1-S.mp3", {
      headers: range ? { Range: range } : {},
    });

  it("đòi khoảng thì cắt ra 206, không trả nguyên bản 200", async () => {
    const res = await load(true).serveAudio(audioRequest("bytes=100-199"));

    expect(res.status).toBe(206);
    expect(res.headers.get("Content-Range")).toBe("bytes 100-199/1000");
  });

  it("không đòi khoảng thì trả nguyên bài", async () => {
    const res = await load(true).serveAudio(audioRequest());

    expect(res.status).toBe(200);
    expect((await res.arrayBuffer()).byteLength).toBe(1000);
  });

  it("chưa tải cuốn nào thì ra thẳng mạng", async () => {
    const fetched = new Response("từ mạng");
    vi.stubGlobal("fetch", vi.fn(async () => fetched));

    const res = await load(false).serveAudio(audioRequest());

    expect(await res.text()).toBe("từ mạng");
    vi.unstubAllGlobals();
  });
});
