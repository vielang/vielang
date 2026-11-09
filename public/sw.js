/**
 * Service worker cho VieLang.
 *
 * Mục tiêu hẹp và cố ý hẹp: đọc lại được những bài ĐÃ MỞ khi mất mạng.
 * Không đặt ra tham vọng chạy trọn vẹn offline.
 *
 * Service worker là thứ hỏng thì hỏng dai — nó sống qua cả lần tải lại
 * trang, nên một bản cache sai có thể khoá người dùng ở phiên bản cũ mãi.
 * Vì vậy ở đây chọn mọi mặc định theo hướng an toàn:
 *
 * - Tài liệu HTML đi MẠNG TRƯỚC. Không bao giờ phục vụ HTML cũ khi còn mạng,
 *   nên deploy bản mới là người dùng thấy ngay.
 * - Chunk của Next đi CACHE TRƯỚC, an toàn vì tên file có băm nội dung: nội
 *   dung đổi thì tên đổi, không có chuyện trùng tên khác ruột.
 * - Ảnh đi CACHE TRƯỚC và có giới hạn số lượng.
 * - Payload RSC (chuyển trang trong app) để trình duyệt tự lo.
 * - KHÔNG tự gọi `skipWaiting()`: bản mới đứng chờ tới khi mọi tab đóng,
 *   hoặc tới khi CHÍNH NGƯỜI DÙNG bấm cập nhật (xem trình nghe "message").
 *   Tự tráo giữa chừng có thể khiến trang đang mở đi xin chunk của phiên bản
 *   khác; còn người dùng bấm thì trang được tải lại ngay sau đó nên an toàn.
 *
 * Tên cache giữ tiền tố `kiip-` từ tên cũ của app: đây là KHOÁ TRA CACHE
 * chứ không phải nhãn. Lên phiên bản mới (VERSION) thì mọi cache không còn
 * dùng — kể cả cache sách tải offline của bản cũ — được dọn ở bước activate.
 */
const VERSION = "v3";
const DOC_CACHE = `kiip-doc-${VERSION}`;
const ASSET_CACHE = `kiip-asset-${VERSION}`;
const IMAGE_CACHE = `kiip-image-${VERSION}`;

/** Trang hiện khi mất mạng mà trang được xin lại chưa từng xem. */
const OFFLINE_URL = "/offline";

/** Giữ chừng này ảnh là đủ — ảnh chỉ là icon, ảnh minh hoạ nhỏ. */
const MAX_IMAGES = 100;

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
      await Promise.all(names.map((n) => (keep.has(n) ? null : caches.delete(n))));
      await self.clients.claim();
    })()
  );
});

/** Ảnh: qua bộ tối ưu của Next, hoặc file tĩnh. */
function isImage(url) {
  return url.pathname === "/_next/image" || /\.(webp|avif|png|jpe?g|svg)$/i.test(url.pathname);
}

/** Chunk/CSS/phông do build sinh ra — tên có băm nội dung nên không đổi ruột. */
function isImmutableAsset(url) {
  return url.pathname.startsWith("/_next/static/");
}

/** Xoá bớt từ đầu cache khi vượt hạn — khoá trong Cache API xếp theo thứ tự thêm vào. */
async function trim(cacheName, max) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();
  if (keys.length <= max) return;
  await Promise.all(keys.slice(0, keys.length - max).map((k) => cache.delete(k)));
}

async function cacheFirst(request, cacheName, max) {
  const cache = await caches.open(cacheName);
  const hit = await cache.match(request);
  if (hit) return hit;

  const response = await fetch(request);
  // Chỉ giữ bản trả về lành lặn, tránh cache nhầm một lỗi 404.
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
    const hit = await cache.match(request);
    if (hit) return hit;
    const offline = await cache.match(OFFLINE_URL);
    if (offline) return offline;
    throw err;
  }
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  // Chỉ đụng tới GET cùng origin; mọi thứ khác để trình duyệt tự lo.
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
  if (isImage(url)) {
    event.respondWith(cacheFirst(request, IMAGE_CACHE, MAX_IMAGES));
  }
});
