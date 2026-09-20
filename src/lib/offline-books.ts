"use client";

import { getImageProps } from "next/image";
import { getPageUrl, type Book } from "@/lib/books";

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

/** Bề rộng ảnh tải về. Đủ nét để phóng to đọc chữ Hàn mà chưa thành nặng. */
const IMAGE_WIDTH = 1080;

/** Ước lượng dung lượng mỗi trang, chỉ để báo trước cho người dùng. */
const BYTES_PER_PAGE = 280 * 1024;

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

/** Dung lượng ước tính của một cuốn, tính bằng byte. */
export function estimateBytes(book: Book): number {
  return book.totalPages * BYTES_PER_PAGE;
}

export function formatBytes(bytes: number): string {
  const mb = bytes / 1048576;
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}

/**
 * Đường dẫn ảnh đúng như trình duyệt sẽ xin khi đọc sách.
 *
 * Đi qua `getImageProps` — API công khai của next/image — thay vì tự ghép
 * chuỗi `/_next/image?...`, để không phụ thuộc vào định dạng bên trong của
 * Next. Lấy nhánh `1x` trong `srcSet`: nhánh `2x` nét hơn nhưng nặng gấp
 * đôi, mà cả cuốn thì gấp đôi là hơn trăm MB.
 */
export function pageImageUrl(bookId: string, page: number): string {
  const { props } = getImageProps({
    src: getPageUrl(bookId, page),
    alt: "",
    width: IMAGE_WIDTH,
    height: Math.round(IMAGE_WIDTH * 1.3),
    quality: 90,
  });
  const oneX = props.srcSet?.split(",").find((c) => c.trim().endsWith("1x"));
  return oneX ? oneX.trim().split(/\s+/)[0] : props.src;
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
export async function listOfflineBooks(): Promise<string[]> {
  if (typeof caches === "undefined") return [];
  try {
    return (await caches.keys()).filter(isBookCache).map(bookIdFromCache);
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

/**
 * Tải cả cuốn. Gọi `onProgress` sau mỗi trang xong, và dừng ngay khi
 * `signal` bị huỷ.
 *
 * Trang nào tải hỏng thì BỎ QUA chứ không làm hỏng cả lượt: mạng di động
 * rớt một nhịp là chuyện thường, mất một trang còn hơn mất cả cuốn. Số trang
 * thật sự tải được trả về ở cuối để bên gọi biết mà nói lại.
 */
export async function downloadBook(
  book: Book,
  {
    onProgress,
    signal,
  }: { onProgress?: (p: DownloadProgress) => void; signal?: AbortSignal } = {}
): Promise<{ saved: number; total: number }> {
  const cache = await caches.open(bookCacheName(book.id));
  const total = book.totalPages;
  let done = 0;
  let saved = 0;

  const pages = Array.from({ length: total }, (_, i) => i + 1);

  async function worker() {
    for (;;) {
      if (signal?.aborted) return;
      const page = pages.shift();
      if (page === undefined) return;
      try {
        // `cache.addAll` sẽ vứt cả nhóm nếu một đường hỏng — tự thêm từng
        // cái để giữ được phần đã tải.
        await Promise.all([
          ...[pageImageUrl(book.id, page), pageDocUrl(book.id, page)].map(
            async (url) => {
              const response = await fetch(url, { signal });
              if (response.ok) await cache.put(url, response);
            }
          ),
          cacheRsc(cache, book.id, page, signal),
        ]);
        saved++;
      } catch {
        /* trang này hỏng — đi tiếp, xem chú thích ở trên */
      }
      done++;
      onProgress?.({ done, total });
    }
  }

  await Promise.all(Array.from({ length: CONCURRENCY }, worker));
  return { saved, total };
}
