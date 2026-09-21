"use client";

import { getPageUrl, type Book } from "@/lib/books";
import { getAudioPages, getPageAudio } from "@/lib/audio";

/**
 * Tải nguyên một cuốn sách về máy để đọc khi mất mạng.
 *
 * Mỗi cuốn có một cache riêng mang tên `kiip-book-<id>`. Nhờ vậy xoá sách là
 * xoá đúng một cache — không phải đi dò từng khoá xem tấm ảnh này thuộc cuốn
 * nào. Service worker đọc luôn từ đây (xem `public/sw.js`).
 *
 * Trang web tự ghi thẳng vào Cache API chứ không nhờ service worker tải hộ:
 * `caches` dùng được ở cả hai phía, mà làm ở đây thì báo tiến độ và cho huỷ
 * giữa chừng dễ hơn nhiều.
 */

const BOOK_CACHE_PREFIX = "kiip-book-";

/**
 * Ước lượng dung lượng mỗi trang, chỉ để báo trước cho người dùng.
 *
 * Đo thật trên R2 (3 trang mẫu mỗi cuốn), sau khi ảnh thôi đi qua bộ tối ưu
 * và tải thẳng bản gốc:
 *
 *   giáo trình tiếng Hàn  113–142 KB
 *   sách bài tập           ~41 KB  (trang thưa chữ, ít ảnh)
 *   giáo trình tiếng Anh  ~373 KB  (ảnh màu, nặng hơn hẳn)
 *
 * Lấy 180 KB: nhỉnh hơn nhóm tiếng Hàn — nhóm đông người dùng nhất — nên
 * con số báo ra hơi dư chứ không hụt. Sách tiếng Anh vẫn bị báo thiếu, nhưng
 * thà vậy còn hơn doạ người học tiếng Hàn bằng một con số gấp đôi sự thật.
 *
 * Con số cũ là 280 KB, hợp với thời còn tải bản đã tối ưu ở 1080px.
 */
const BYTES_PER_PAGE = 180 * 1024;

/**
 * Ước lượng dung lượng mỗi bài nghe. Đo thật trên R2, 3 file mẫu mỗi cuốn:
 *
 *   giáo trình tiếng Hàn  1205–1442 KB
 *   sách bài tập          ~1494 KB
 *   giáo trình tiếng Anh  ~1118 KB
 *
 * Đều tay hơn ảnh nhiều, nên một con số chung là đủ sát.
 */
const BYTES_PER_TRACK = 1300 * 1024;

/**
 * Bài nghe có gói theo được xuống máy hay không.
 *
 * ĐANG TẮT. Bài nghe nay trỏ thẳng R2 (xem `lib/audio.ts` để biết vì sao),
 * mà bucket chưa bật CORS — `fetch` từ trang nhận về bản opaque nên
 * `cache.put` không lưu được gì.
 *
 * Để nguyên thì tệ theo đúng kiểu mà cả file này sinh ra để tránh: người
 * dùng thấy báo cần 100MB, tải xong thấy lưu thiếu hơn nửa, rồi mất mạng
 * mở ra thì audio im lặng — tưởng đã mang sách theo mà hoá ra không.
 *
 * Bật CORS cho bucket rồi đổi cờ này thành `true` là xong, không phải sửa
 * gì thêm ở đây. (Muốn nghe được khi offline thì còn phải mở thêm origin
 * R2 ở chốt cùng-origin trong `sw.js` nữa.)
 */
export const AUDIO_CAN_BE_CACHED = false;

/** Tải bao nhiêu trang một lúc. Nhiều hơn thì mạng di động bắt đầu nghẽn. */
const CONCURRENCY = 4;

export function bookCacheName(bookId: string): string {
  return `${BOOK_CACHE_PREFIX}${bookId}`;
}

export function isBookCache(name: string): boolean {
  return name.startsWith(BOOK_CACHE_PREFIX);
}

/** "kiip-book-step1" -> "step1" */
export function bookIdFromCache(name: string): string {
  return name.slice(BOOK_CACHE_PREFIX.length);
}

/**
 * Số bài nghe của cả cuốn. Đếm thật qua bảng tra audio chứ không ước:
 * số track mỗi bài khác nhau giữa giáo trình (3), sách bài tập (2) và sách
 * tiếng Anh (thay đổi theo trang).
 */
export function countAudioTracks(bookId: string): number {
  return getAudioPages(bookId).reduce(
    (sum, page) => sum + getPageAudio(bookId, page).length,
    0
  );
}

/**
 * Dung lượng ước tính của một cuốn, tính bằng byte.
 *
 * Audio chiếm phần lớn chứ không phải ảnh — một bài nghe nặng gấp cả chục
 * lần một trang sách. Với Sơ cấp 1: ~30MB ảnh nhưng ~70MB audio. Bỏ audio
 * ra khỏi con số này là báo thiếu tới ba lần, người dùng bấm tải xong mới
 * ngã ngửa vì hết chỗ.
 */
export function estimateBytes(book: Book): number {
  return (
    book.totalPages * BYTES_PER_PAGE +
    (AUDIO_CAN_BE_CACHED ? countAudioTracks(book.id) * BYTES_PER_TRACK : 0)
  );
}

export function formatBytes(bytes: number): string {
  const mb = bytes / 1048576;
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}

/**
 * Đường dẫn ảnh đúng như trình duyệt sẽ xin khi đọc sách.
 *
 * PHẢI khớp từng ký tự với `getPageUrl` mà `page-viewer.tsx` dùng, vì service
 * worker tra cache theo nguyên URL. Lệch một ký tự là tải cả cuốn về rồi lúc
 * mất mạng vẫn trắng trang.
 *
 * Trước đây chỗ này phải gọi `getImageProps` để dựng lại chuỗi
 * `/_next/image?...` cho khớp. Nay ảnh không qua bộ tối ưu nữa nên URL chính
 * là đường ảnh, gọi thẳng là xong.
 */
export function pageImageUrl(bookId: string, page: number): string {
  return getPageUrl(bookId, page);
}

/** Đường dẫn tài liệu HTML của một trang sách. */
function pageDocUrl(bookId: string, page: number): string {
  return `/read/${bookId}/${page}`;
}

/**
 * Khoá cache tự đặt cho payload RSC của một trang.
 *
 * Không dùng thẳng đường dẫn mà Next xin, vì nó kèm tham số `?_rsc=<băm>`
 * đổi theo trạng thái router — không đoán trước được lúc tải về. Tự đặt một
 * khoá riêng thì cả hai đầu (bên tải và service worker) cùng tính ra được.
 *
 * PHẢI khớp với hàm cùng tên trong `public/sw.js`.
 */
export function rscCacheKey(bookId: string, page: number): string {
  return `${pageDocUrl(bookId, page)}?__offline_rsc=1`;
}

/**
 * Tải payload RSC của một trang và cất dưới khoá tự đặt.
 *
 * Cần nó vì lật trang trong app là điều hướng phía client: Next xin payload
 * RSC chứ không tải lại cả tài liệu HTML. Không có payload thì mất mạng bấm
 * mũi tên lật trang sẽ không ăn, dù gõ thẳng URL vẫn mở được.
 *
 * Dựng lại Response sạch thay vì cất nguyên bản: bản gốc mang `Vary` liệt kê
 * cả `next-router-state-tree`, mà Cache API đem `Vary` ra đối chiếu thì
 * không bao giờ khớp lại được.
 */
async function cacheRsc(cache: Cache, bookId: string, page: number, signal?: AbortSignal) {
  const response = await fetch(pageDocUrl(bookId, page), {
    headers: { RSC: "1" },
    signal,
  });
  if (!response.ok) return;

  const headers = new Headers(response.headers);
  headers.delete("Vary");
  await cache.put(
    rscCacheKey(bookId, page),
    new Response(await response.blob(), { headers })
  );
}

/** Các cuốn đang có trên máy. Cache API là nguồn sự thật, không phải store. */
/**
 * Cuốn này được tải về từ thời ảnh còn đi qua `/_next/image` phải không?
 *
 * Cách nhận: xem thử có khoá nào còn mang dạng URL cũ. Đời cache mới lưu theo
 * `/img/books/...`, nên chỉ cần thấy một khoá `/_next/image` là biết cả cuốn
 * thuộc đời cũ.
 */
async function isStaleScheme(cache: Cache): Promise<boolean> {
  const keys = await cache.keys();
  return keys.some((req) => new URL(req.url).pathname === "/_next/image");
}

/**
 * Danh sách sách đã tải, ĐỒNG THỜI dọn những cuốn tải theo đời URL cũ.
 *
 * Vì sao phải dọn: đường ảnh đã đổi từ `/_next/image?url=...` sang
 * `/img/books/...` (xem `next.config.ts`). Service worker tra cache theo
 * nguyên URL, nên mấy cuốn tải từ trước sẽ không bao giờ khớp nữa. Để nguyên
 * thì giao diện vẫn khoe "Đã tải offline" mà người dùng mất mạng mở ra lại
 * trắng trang — kiểu hỏng tệ nhất, vì họ tin là mình đã mang sách theo rồi.
 *
 * Xoá đi thì nút quay về "Tải offline", họ tải lại một lần là xong. Mất công
 * một lần còn hơn tưởng có mà hoá ra không.
 */
export async function listOfflineBooks(): Promise<string[]> {
  if (typeof caches === "undefined") return [];
  try {
    const names = (await caches.keys()).filter(isBookCache);
    const ids: string[] = [];
    for (const name of names) {
      const cache = await caches.open(name);
      if (await isStaleScheme(cache)) {
        await caches.delete(name);
        continue;
      }
      ids.push(bookIdFromCache(name));
    }
    return ids;
  } catch {
    return [];
  }
}

export async function deleteOfflineBook(bookId: string): Promise<void> {
  if (typeof caches === "undefined") return;
  try {
    await caches.delete(bookCacheName(bookId));
  } catch {
    /* trình duyệt chặn Cache API — không có gì để xoá */
  }
}

export interface DownloadProgress {
  done: number;
  total: number;
}

/** Một việc cần tải: hoặc một trang (ảnh + HTML + RSC), hoặc một bài nghe. */
type Task =
  | { kind: "page"; page: number }
  | { kind: "audio"; url: string };

/**
 * Xếp việc theo thứ tự đọc: trang 1, audio của trang 1, trang 2…
 *
 * Đếm bài nghe thành ĐƠN VỊ RIÊNG chứ không gộp vào trang của nó. Một trang
 * có audio nặng ~4MB, gấp hơn hai chục lần trang thường — gộp thì thanh tiến
 * độ đứng im cả nửa phút ở đúng 18 bước đó, nhìn y như treo và người dùng
 * bấm huỷ.
 */
function buildTasks(book: Book): Task[] {
  const tasks: Task[] = [];
  for (let page = 1; page <= book.totalPages; page++) {
    tasks.push({ kind: "page", page });
    if (AUDIO_CAN_BE_CACHED) {
      for (const track of getPageAudio(book.id, page)) {
        tasks.push({ kind: "audio", url: track.url });
      }
    }
  }
  return tasks;
}

/**
 * Tải cả cuốn: ảnh trang, HTML, payload RSC và bài nghe. Gọi `onProgress`
 * sau mỗi phần xong, và dừng ngay khi `signal` bị huỷ.
 *
 * Phần nào tải hỏng thì BỎ QUA chứ không làm hỏng cả lượt: mạng di động rớt
 * một nhịp là chuyện thường, mất một trang còn hơn mất cả cuốn. Số phần thật
 * sự tải được trả về ở cuối để bên gọi biết mà nói lại.
 */
export async function downloadBook(
  book: Book,
  {
    onProgress,
    signal,
  }: { onProgress?: (p: DownloadProgress) => void; signal?: AbortSignal } = {}
): Promise<{ saved: number; total: number }> {
  const cache = await caches.open(bookCacheName(book.id));
  const tasks = buildTasks(book);
  const total = tasks.length;
  let done = 0;
  let saved = 0;

  /** Tải một đường rồi cất vào cache. Bỏ qua bản trả về hỏng. */
  async function save(url: string) {
    const response = await fetch(url, { signal });
    if (response.ok) await cache.put(url, response);
  }

  async function worker() {
    for (;;) {
      if (signal?.aborted) return;
      const task = tasks.shift();
      if (task === undefined) return;
      try {
        if (task.kind === "audio") {
          // Audio nằm CÙNG cache `kiip-book-<id>` với ảnh, không tách riêng:
          // nhờ vậy xoá sách vẫn chỉ là xoá đúng một cache, ảnh và audio đi
          // cùng nhau, không có đường nào sót lại chiếm chỗ trên máy.
          await save(task.url);
        } else {
          // `cache.addAll` sẽ vứt cả nhóm nếu một đường hỏng — tự thêm từng
          // cái để giữ được phần đã tải.
          await Promise.all([
            save(pageImageUrl(book.id, task.page)),
            save(pageDocUrl(book.id, task.page)),
            cacheRsc(cache, book.id, task.page, signal),
          ]);
        }
        saved++;
      } catch {
        /* phần này hỏng — đi tiếp, xem chú thích ở trên */
      }
      done++;
      onProgress?.({ done, total });
    }
  }

  await Promise.all(Array.from({ length: CONCURRENCY }, worker));
  return { saved, total };
}
