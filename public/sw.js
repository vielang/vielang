/**
 * Service worker cho KIIP Reader.
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
 * - KHÔNG gọi `skipWaiting()`: bản mới chỉ tiếp quản khi mọi tab đã đóng.
 *   Tráo service worker giữa chừng có thể khiến trang đang mở đi xin chunk
 *   của phiên bản khác.
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
  if (isImmutableAsset(url)) {
    event.respondWith(cacheFirst(request, ASSET_CACHE));
    return;
  }
  if (isPageImage(url)) {
    event.respondWith(cacheFirst(request, IMAGE_CACHE, MAX_IMAGES));
  }
});
