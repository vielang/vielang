import { afterEach, describe, expect, it, vi } from "vitest";
import {
  AUDIO_CAN_BE_CACHED,
  bookCacheName,
  countAudioTracks,
  deleteOfflineBook,
  estimateBytes,
  listOfflineBooks,
  pageImageUrl,
} from "./offline-books";
import { BOOKS, getPageUrl } from "./books";

/**
 * Cache API giả, vừa đủ cho `listOfflineBooks`: chỉ cần kể tên cache và kể
 * các khoá bên trong.
 */
function mockCaches(store: Record<string, string[]>) {
  const deleted: string[] = [];
  vi.stubGlobal("caches", {
    keys: async () => Object.keys(store),
    open: async (name: string) => ({
      keys: async () =>
        (store[name] ?? []).map((url) => ({ url: new URL(url, "https://x").href })),
    }),
    delete: async (name: string) => {
      deleted.push(name);
      delete store[name];
      return true;
    },
  });
  return deleted;
}

afterEach(() => vi.unstubAllGlobals());

describe("đường ảnh lúc tải về", () => {
  it("khớp từng ký tự với đường ảnh lúc đọc", () => {
    // Service worker tra cache theo NGUYÊN URL. Lệch một ký tự là tải cả cuốn
    // về rồi lúc mất mạng vẫn trắng trang — mà không có lỗi nào bắn ra.
    expect(pageImageUrl("en-elementary", 20)).toBe(getPageUrl("en-elementary", 20));
  });

  it("là đường cùng origin, không phải URL R2 trần", () => {
    // `sw.js` bỏ qua mọi request khác origin, nên trỏ thẳng r2.dev là mất
    // sạch cache ảnh lẫn tính năng offline.
    expect(pageImageUrl("en-elementary", 20).startsWith("/img/books/")).toBe(true);
  });
});

describe("sách tải theo đời URL cũ", () => {
  it("bị dọn đi thay vì để người dùng tưởng vẫn đọc offline được", () => {
    // Đời cũ lưu ảnh dưới dạng `/_next/image?url=...`. Sau khi đổi đường ảnh,
    // mấy khoá đó không bao giờ khớp nữa: giao diện vẫn khoe "Đã tải
    // offline" mà mở ra lúc mất mạng thì trắng trang.
    const deleted = mockCaches({
      [bookCacheName("en-elementary")]: [
        "/_next/image?url=%2Fbooks%2Fen-elementary%2Fpages%2F0001.webp&w=1080&q=90",
        "/read/en-elementary/1",
      ],
    });

    return listOfflineBooks().then((ids) => {
      expect(ids).toEqual([]);
      expect(deleted).toEqual([bookCacheName("en-elementary")]);
    });
  });
});

describe("sách tải theo đời URL mới", () => {
  it("được giữ nguyên", async () => {
    const deleted = mockCaches({
      [bookCacheName("en-pre-intermediate")]: [
        "/img/books/en-pre-intermediate/pages/0001.webp",
        "/read/en-pre-intermediate/1",
      ],
    });

    expect(await listOfflineBooks()).toEqual(["en-pre-intermediate"]);
    expect(deleted).toEqual([]);
  });

  it("chỉ dọn cuốn đời cũ, không đụng cuốn đời mới", async () => {
    const deleted = mockCaches({
      [bookCacheName("en-elementary")]: ["/_next/image?url=x&w=1080&q=90"],
      [bookCacheName("en-pre-intermediate")]: ["/img/books/en-pre-intermediate/pages/0001.webp"],
    });

    expect(await listOfflineBooks()).toEqual(["en-pre-intermediate"]);
    expect(deleted).toEqual([bookCacheName("en-elementary")]);
  });
});

describe("trình duyệt chặn Cache API", () => {
  it("trả về danh sách rỗng chứ không làm vỡ trang", async () => {
    vi.stubGlobal("caches", {
      keys: async () => {
        throw new Error("bị chặn");
      },
    });

    expect(await listOfflineBooks()).toEqual([]);
  });
});

describe("gói tải về gồm cả bài nghe", () => {
  it("đếm đúng số bài nghe của cuốn", () => {
    // Elementary: mỗi track in trên trang là một file — 221 track trong bảng trang.
    expect(countAudioTracks("en-elementary")).toBe(221);
  });

  it("ước lượng khớp với thứ THẬT SỰ tải về", () => {
    // Đây mới là bất biến, chứ không phải “có cộng audio hay không”. Báo
    // một đằng tải một nẻo thì hoặc người dùng hết chỗ máy, hoặc tưởng đã
    // mang sách theo mà hoá ra không — cả hai đều chỉ lộ ra lúc mất mạng.
    const book = BOOKS.find((b) => b.id === "en-elementary")!;
    const images = book.totalPages * 180 * 1024;
    const audio = countAudioTracks("en-elementary") * 1300 * 1024;

    expect(estimateBytes(book)).toBe(
      AUDIO_CAN_BE_CACHED ? images + audio : images
    );
  });

  it("sách không có audio thì không cộng thêm gì", () => {
    // Vài cuốn tiếng Anh chưa khảo sát xong bảng track nên chưa có audio.
    // Khẳng định luôn là có cuốn như vậy, chứ không lặng lẽ bỏ qua: bỏ qua
    // thì hôm nào mọi cuốn đều có audio, test này xanh mà chẳng kiểm gì.
    const noAudio = BOOKS.find((b) => countAudioTracks(b.id) === 0);

    expect(noAudio).toBeDefined();
    expect(estimateBytes(noAudio!)).toBe(noAudio!.totalPages * 180 * 1024);
  });
});

describe("xoá sách offline", () => {
  it("xoá đúng MỘT cache, nên ảnh và audio đi cùng nhau", async () => {
    // Cả hai nằm chung `kiip-book-<id>` chính là để chuyện này đúng: không
    // có đường nào sót lại âm thầm chiếm chỗ trên máy người dùng.
    const store: Record<string, string[]> = {
      [bookCacheName("en-elementary")]: [
        "/img/books/en-elementary/pages/0001.webp",
        "/img/books/en-elementary/audio/1.2.mp3",
        "/read/en-elementary/1",
      ],
    };
    const deleted = mockCaches(store);

    await deleteOfflineBook("en-elementary");

    expect(deleted).toEqual([bookCacheName("en-elementary")]);
    expect(store[bookCacheName("en-elementary")]).toBeUndefined();
  });
});
