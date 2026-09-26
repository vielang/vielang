/**
 * Service worker cho VieLang.
 *
 * Mục tiêu hẹp và cố ý hẹp: đọc lại được những trang sách ĐÃ XEM khi mất
 * mạng. Không đặt ra tham vọng chạy trọn vẹn offline.
 *
 * Service worker là thứ hỏng thì hỏng dai — nó sống qua cả lần tải lại
 * trang, nên một bản cache sai có thể khoá người dùng ở phiên bản cũ mãi.
 * Vì vậy ở đây chọn mọi mặc định theo hướng an toàn:
 *
 * - Tài liệu HTML đi MẠNG TRƯỚC. Không bao giờ phục vụ HTML cũ khi còn mạng,
 *   nên deploy bản mới là người dùng thấy ngay.
 * - Chunk của Next đi CACHE TRƯỚC, an toàn vì tên file có băm nội dung: nội
 *   dung đổi thì tên đổi, không có chuyện trùng tên khác ruột.
 * - Ảnh trang sách đi CACHE TRƯỚC và có giới hạn số lượng — đây là phần
 *   nặng nhất, mà cũng là phần đáng giữ nhất khi offline.
 * - Sách người dùng CHỦ ĐỘNG tải về nằm trong cache riêng `kiip-book-<id>`
 *   (xem `lib/offline-books.ts`) và luôn được tra TRƯỚC. Cache đó không bao
 *   giờ bị dọn tự động: người dùng đã cố ý tải thì chỉ người dùng được xoá.
 * - Payload RSC (lật trang trong app) đi MẠNG TRƯỚC, chỉ lấy bản cache khi
 *   mạng hỏng hẳn — xem `rscCacheKey`.
 * - KHÔNG tự gọi `skipWaiting()`: bản mới đứng chờ tới khi mọi tab đóng,
 *   hoặc tới khi CHÍNH NGƯỜI DÙNG bấm cập nhật (xem trình nghe "message").
 *   Tự tráo giữa chừng có thể khiến trang đang mở đi xin chunk của phiên bản
 *   khác; còn người dùng bấm thì trang được tải lại ngay sau đó nên an toàn.
 */
/**
 * Tên cache giữ tiền tố `kiip-` dù app đã đổi tên thành VieLang: đây là
 * KHOÁ TRA CACHE chứ không phải nhãn. Đáng ngại nhất là `kiip-book-` — cache
 * sách người dùng chủ động tải, cố ý không bao giờ bị dọn tự động. Đổi tiền
 * tố thì những cache cũ vừa không còn được tra, vừa lọt khỏi diện "giữ lại"
 * lẫn diện "người dùng xoá được", thành rác chiếm chỗ vĩnh viễn trên máy họ,
 * mà nhìn ra ngoài thì sách đã tải bỗng biến mất.
 */
const VERSION = "v2";
const DOC_CACHE = `kiip-doc-${VERSION}`;
const ASSET_CACHE = `kiip-asset-${VERSION}`;
const IMAGE_CACHE = `kiip-image-${VERSION}`;

/** Trang hiện khi mất mạng mà trang được xin lại chưa từng xem. */
const OFFLINE_URL = "/offline";

/** Ảnh trang sách rất nặng — giữ chừng này là đủ vài bài học gần nhất. */
const MAX_IMAGES = 300;

/** Tiền tố cache của sách tải chủ động — phải khớp `lib/offline-books.ts`. */
const BOOK_CACHE_PREFIX = "kiip-book-";

/**
 * Người dùng bấm "Cập nhật" trên thanh thông báo.
 *
 * Chỉ lúc này mới nhường chỗ cho bản mới — và bên giao diện tải lại trang
 * ngay khi thấy service worker đổi, nên không có cảnh trang cũ chạy với
 * service worker mới.
 */
self.addEventListener("message", (event) => {
  if (event.data === "SKIP_WAITING") self.skipWaiting();
});

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(DOC_CACHE).then((cache) => cache.add(OFFLINE_URL)).catch(() => {})
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keep = new Set([DOC_CACHE, ASSET_CACHE, IMAGE_CACHE]);
      const names = await caches.keys();
      await Promise.all(
        names.map((n) =>
          // Sách tải chủ động sống sót qua mọi lần nâng cấp service worker.
          // Dọn nhầm là người dùng mất cả cuốn vừa tải 60MB mà không hiểu vì sao.
          keep.has(n) || n.startsWith(BOOK_CACHE_PREFIX) ? null : caches.delete(n)
        )
      );
      await self.clients.claim();
    })()
  );
});

/** Ảnh trang sách: qua bộ tối ưu của Next, hoặc thẳng từ kho R2. */
function isPageImage(url) {
  return (
    url.pathname === "/_next/image" ||
    /\.(webp|avif|png|jpe?g)$/i.test(url.pathname)
  );
}

/**
 * Next xin payload RSC khi lật trang trong app: header `rsc: 1`, rồi server
 * 307 sang `?_rsc=<băm>`. Nhận cả hai dạng vì tuỳ lúc ta bắt được cái nào.
 */
function isRscRequest(request, url) {
  return request.headers.get("rsc") !== null || url.searchParams.has("_rsc");
}

/**
 * Khoá cache tự đặt cho payload RSC — băm `_rsc` đổi theo trạng thái router
 * nên không dùng làm khoá được.
 *
 * PHẢI khớp `rscCacheKey` trong `lib/offline-books.ts`.
 */
function rscCacheKey(url) {
  return `${url.pathname}?__offline_rsc=1`;
}

/**
 * Payload RSC: MẠNG TRƯỚC, cache chỉ là lưới đỡ khi mất mạng.
 *
 * Cố ý không cache-trước như ảnh. Bản trả về của RSC khai `Vary` gồm cả
 * `next-router-state-tree`, tức nội dung đổi theo chỗ router đang đứng —
 * đem một bản cũ ra dùng khi vẫn còn mạng là chuốc lấy lỗi render khó truy.
 * Còn khi đã mất mạng thì bản đầy đủ đã tải vẫn hơn là hỏng hẳn.
 */
async function rscNetworkFirst(request, url) {
  try {
    return await fetch(request);
  } catch (err) {
    const hit = await matchDownloadedBook(rscCacheKey(url));
    if (hit) return hit;
    throw err;
  }
}

/** Chunk/CSS/phông do build sinh ra — tên có băm nội dung nên không đổi ruột. */
function isImmutableAsset(url) {
  return (
    url.pathname.startsWith("/_next/static/") ||
    url.pathname.startsWith("/excalidraw/fonts/")
  );
}

/** Xoá bớt từ đầu cache khi vượt hạn — khoá trong Cache API xếp theo thứ tự thêm vào. */
async function trim(cacheName, max) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();
  if (keys.length <= max) return;
  await Promise.all(keys.slice(0, keys.length - max).map((k) => cache.delete(k)));
}

/**
 * Tra khắp các cache sách đã tải.
 *
 * `caches.match` không nhận danh sách cache nên phải tự đi từng cái; số cuốn
 * tải về giới hạn ở 1 nên vòng lặp này gần như luôn chỉ chạy một nhịp.
 */
async function matchDownloadedBook(request) {
  const names = (await caches.keys()).filter((n) => n.startsWith(BOOK_CACHE_PREFIX));
  for (const name of names) {
    const cache = await caches.open(name);
    const hit = await cache.match(request);
    if (hit) return hit;
  }
  return undefined;
}

/** Bài nghe của sách — nhận theo đuôi tệp, cùng cách `isPageImage` làm. */
function isAudio(url) {
  return /\.mp3$/i.test(url.pathname);
}

/**
 * Bài nghe: CHỈ lấy từ cuốn người dùng đã chủ động tải, còn lại ra thẳng mạng.
 *
 * Cố ý KHÔNG cache tự động như ảnh. Một bài nghe nặng 1,3MB — gấp mười một
 * trang sách. Nghe lướt vài bài là đầy máy người ta, mà họ có xin đâu. Ai
 * muốn mang theo thì bấm tải cả cuốn.
 *
 * TẠM THỜI KHÔNG VỚI TỚI: bài nghe nay trỏ thẳng R2 (xem `lib/audio.ts`
 * để biết vì sao), mà service worker bỏ qua mọi request khác origin — nên
 * nhánh này không chạy nữa. Giữ lại chứ không xoá: bật CORS cho bucket R2
 * rồi mở thêm origin đó ở chốt bên dưới là nó sống lại nguyên vẹn, kể cả
 * phần cắt khoảng byte vốn là chỗ khó nhất.
 *
 * Trình duyệt vốn đã tự cache rồi (R2 gửi `immutable`, hạn một năm), nên
 * nghe lại lần hai vẫn không tốn mạng dù ta không đụng tay vào.
 */
async function serveAudio(request) {
  const hit = await matchDownloadedBook(request);
  if (!hit) return fetch(request);

  const range = request.headers.get("range");
  if (!range) return hit;
  return sliceForRange(hit, range);
}

/**
 * Cắt một khúc của bản trả về đã cache, trả về 206.
 *
 * BẮT BUỘC, không phải cho đẹp: Cache API luôn trả nguyên bản 200, mà
 * <audio> thì xin theo khoảng (`Range`) mỗi lần tua. Safari/iOS từ chối
 * phát hẳn nếu đòi khoảng mà nhận về 200 — tức là trên iPhone, sách đã tải
 * về vẫn không nghe được. Chrome dễ tính hơn nhưng tua thì vẫn hỏng.
 *
 * Chỉ nhận dạng `bytes=START-END` và `bytes=START-`; dạng nhiều khoảng hay
 * đếm ngược từ cuối thì nhường cho mạng — trình duyệt không dùng mấy dạng
 * đó cho thẻ <audio>.
 */
async function sliceForRange(response, range) {
  const match = /^bytes=(\d*)-(\d*)$/.exec(range.trim());
  if (!match || match[1] === "") return response;

  const buffer = await response.arrayBuffer();
  const size = buffer.byteLength;
  const start = Number(match[1]);
  const end = match[2] === "" ? size - 1 : Math.min(Number(match[2]), size - 1);
  if (start > end) return response;

  return new Response(buffer.slice(start, end + 1), {
    status: 206,
    statusText: "Partial Content",
    headers: {
      "Content-Type": response.headers.get("Content-Type") || "audio/mpeg",
      "Content-Length": String(end - start + 1),
      "Content-Range": `bytes ${start}-${end}/${size}`,
      "Accept-Ranges": "bytes",
    },
  });
}

async function cacheFirst(request, cacheName, max) {
  const downloaded = await matchDownloadedBook(request);
  if (downloaded) return downloaded;

  const cache = await caches.open(cacheName);
  const hit = await cache.match(request);
  if (hit) return hit;

  const response = await fetch(request);
  // Chỉ giữ bản trả về lành lặn. `opaque` (no-cors, vd ảnh từ R2) không đọc
  // được trạng thái nên cũng bỏ qua, tránh cache nhầm một lỗi 404.
  if (response.ok) {
    await cache.put(request, response.clone());
    if (max) await trim(cacheName, max);
  }
  return response;
}

async function networkFirst(request) {
  const cache = await caches.open(DOC_CACHE);
  try {
    const response = await fetch(request);
    if (response.ok) await cache.put(request, response.clone());
    return response;
  } catch (err) {
    // Sách đã tải được tra trước: đó là thứ người dùng chủ động mang theo.
    const downloaded = await matchDownloadedBook(request);
    if (downloaded) return downloaded;
    const hit = await cache.match(request);
    if (hit) return hit;
    const offline = await cache.match(OFFLINE_URL);
    if (offline) return offline;
    throw err;
  }
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  // Chỉ đụng tới GET cùng origin. Ảnh từ R2 đã đi qua /_next/image nên cũng
  // là cùng origin; mọi thứ khác để trình duyệt tự lo.
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  if (request.mode === "navigate") {
    event.respondWith(networkFirst(request));
    return;
  }
  if (isRscRequest(request, url)) {
    event.respondWith(rscNetworkFirst(request, url));
    return;
  }
  if (isImmutableAsset(url)) {
    event.respondWith(cacheFirst(request, ASSET_CACHE));
    return;
  }
  if (isAudio(url)) {
    event.respondWith(serveAudio(request));
    return;
  }
  if (isPageImage(url)) {
    event.respondWith(cacheFirst(request, IMAGE_CACHE, MAX_IMAGES));
  }
});
