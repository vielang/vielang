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
    expect(pageImageUrl("step1", 20)).toBe(getPageUrl("step1", 20));
  });

  it("là đường cùng origin, không phải URL R2 trần", () => {
    // `sw.js` bỏ qua mọi request khác origin, nên trỏ thẳng r2.dev là mất
    // sạch cache ảnh lẫn tính năng offline.
    expect(pageImageUrl("step1", 20).startsWith("/img/books/")).toBe(true);
  });
});

describe("sách tải theo đời URL cũ", () => {
  it("bị dọn đi thay vì để người dùng tưởng vẫn đọc offline được", () => {
    // Đời cũ lưu ảnh dưới dạng `/_next/image?url=...`. Sau khi đổi đường ảnh,
    // mấy khoá đó không bao giờ khớp nữa: giao diện vẫn khoe "Đã tải
    // offline" mà mở ra lúc mất mạng thì trắng trang.
    const deleted = mockCaches({
      [bookCacheName("step1")]: [
        "/_next/image?url=%2Fbooks%2Fstep1%2Fpages%2F0001.webp&w=1080&q=90",
        "/read/step1/1",
      ],
    });

    return listOfflineBooks().then((ids) => {
      expect(ids).toEqual([]);
      expect(deleted).toEqual([bookCacheName("step1")]);
    });
  });
});

describe("sách tải theo đời URL mới", () => {
  it("được giữ nguyên", async () => {
    const deleted = mockCaches({
      [bookCacheName("step2")]: [
        "/img/books/step2/pages/0001.webp",
        "/read/step2/1",
      ],
    });

    expect(await listOfflineBooks()).toEqual(["step2"]);
    expect(deleted).toEqual([]);
  });

  it("chỉ dọn cuốn đời cũ, không đụng cuốn đời mới", async () => {
    const deleted = mockCaches({
      [bookCacheName("step1")]: ["/_next/image?url=x&w=1080&q=90"],
      [bookCacheName("step2")]: ["/img/books/step2/pages/0001.webp"],
    });

    expect(await listOfflineBooks()).toEqual(["step2"]);
    expect(deleted).toEqual([bookCacheName("step1")]);
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
    // Sơ cấp 1: 18 bài × 3 track (말하기, 듣기, 발음) + 1 track mở đầu.
    expect(countAudioTracks("step1")).toBe(55);
  });

  it("ước lượng khớp với thứ THẬT SỰ tải về", () => {
    // Đây mới là bất biến, chứ không phải “có cộng audio hay không”. Báo
    // một đằng tải một nẻo thì hoặc người dùng hết chỗ máy, hoặc tưởng đã
    // mang sách theo mà hoá ra không — cả hai đều chỉ lộ ra lúc mất mạng.
    const step1 = BOOKS.find((b) => b.id === "step1")!;
    const images = step1.totalPages * 180 * 1024;
    const audio = countAudioTracks("step1") * 1300 * 1024;

    expect(estimateBytes(step1)).toBe(
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
      [bookCacheName("step1")]: [
        "/img/books/step1/pages/0001.webp",
        "/img/books/step1/audio/1-S.mp3",
        "/read/step1/1",
      ],
    };
    const deleted = mockCaches(store);

    await deleteOfflineBook("step1");

    expect(deleted).toEqual([bookCacheName("step1")]);
    expect(store[bookCacheName("step1")]).toBeUndefined();
  });
});
