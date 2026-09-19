import { beforeEach, describe, expect, it, vi } from "vitest";
import { IDBFactory } from "fake-indexeddb";
import type { StateStorage } from "zustand/middleware";

/**
 * Mỗi test một IndexedDB trắng VÀ một bản module mới — `idb-storage` nhớ
 * lại kết nối ở module scope, dùng lại là test sau đọc trúng DB của test
 * trước.
 */
async function freshStorage(
  markQuota: (exceeded: boolean) => void = () => {}
): Promise<StateStorage> {
  vi.resetModules();
  globalThis.indexedDB = new IDBFactory();
  const { createIdbStorage } = await import("./idb-storage");
  return createIdbStorage(markQuota);
}

const KEY = "kiip-annotations-v1";

beforeEach(() => {
  localStorage.clear();
});

describe("đọc ghi", () => {
  it("chưa có gì thì trả null", async () => {
    const storage = await freshStorage();
    expect(await storage.getItem(KEY)).toBeNull();
  });

  it("ghi rồi đọc lại đúng nguyên văn", async () => {
    const storage = await freshStorage();
    await storage.setItem(KEY, '{"strokes":{"step1:10":[]}}');
    expect(await storage.getItem(KEY)).toBe('{"strokes":{"step1:10":[]}}');
  });

  it("xoá rồi thì đọc ra null", async () => {
    const storage = await freshStorage();
    await storage.setItem(KEY, "x");
    await storage.removeItem(KEY);
    expect(await storage.getItem(KEY)).toBeNull();
  });

  it("báo ghi thành công qua markQuota", async () => {
    const markQuota = vi.fn();
    const storage = await freshStorage(markQuota);
    await storage.setItem(KEY, "x");
    expect(markQuota).toHaveBeenCalledWith(false);
  });
});

describe("chuyển dữ liệu cũ từ localStorage", () => {
  it("đọc được bản cũ, chép sang IndexedDB rồi dọn localStorage", async () => {
    const storage = await freshStorage();
    localStorage.setItem(KEY, '{"strokes":{"step1:10":["cũ"]}}');

    expect(await storage.getItem(KEY)).toBe('{"strokes":{"step1:10":["cũ"]}}');
    // Đã nằm trong IndexedDB: lần đọc sau không cần tới localStorage nữa.
    expect(localStorage.getItem(KEY)).toBeNull();
    expect(await storage.getItem(KEY)).toBe('{"strokes":{"step1:10":["cũ"]}}');
  });

  it("KHÔNG đè lên dữ liệu đã có trong IndexedDB", async () => {
    // Người dùng đã vẽ tiếp trên bản mới; bản localStorage là quá khứ, đè
    // lên là mất bài.
    const storage = await freshStorage();
    await storage.setItem(KEY, "mới");
    localStorage.setItem(KEY, "cũ");

    expect(await storage.getItem(KEY)).toBe("mới");
  });

  it("chỉ chuyển đúng khoá được hỏi", async () => {
    const storage = await freshStorage();
    localStorage.setItem("kiip-notes-v1", "bài giảng");
    localStorage.setItem(KEY, "nét vẽ");

    await storage.getItem(KEY);

    expect(localStorage.getItem("kiip-notes-v1")).toBe("bài giảng");
  });
});

describe("khi IndexedDB không dùng được", () => {
  it("đọc ra null thay vì ném lỗi", async () => {
    vi.resetModules();
    // @ts-expect-error — dựng lại tình huống trình duyệt chặn IndexedDB.
    delete globalThis.indexedDB;
    const { createIdbStorage } = await import("./idb-storage");
    const storage = createIdbStorage(() => {});

    expect(await storage.getItem(KEY)).toBeNull();
  });

  it("vẫn đọc được bản localStorage cũ để không mất bài", async () => {
    vi.resetModules();
    // @ts-expect-error — xem trên.
    delete globalThis.indexedDB;
    localStorage.setItem(KEY, "cũ");
    const { createIdbStorage } = await import("./idb-storage");
    const storage = createIdbStorage(() => {});

    expect(await storage.getItem(KEY)).toBe("cũ");
    // Chưa chép được sang đâu cả thì tuyệt đối không được xoá bản cũ.
    expect(localStorage.getItem(KEY)).toBe("cũ");
  });

  it("ghi hỏng thì báo qua markQuota chứ không ném", async () => {
    vi.resetModules();
    // @ts-expect-error — xem trên.
    delete globalThis.indexedDB;
    const markQuota = vi.fn();
    const { createIdbStorage } = await import("./idb-storage");
    const storage = createIdbStorage(markQuota);

    await storage.setItem(KEY, "x");
    expect(markQuota).toHaveBeenCalledWith(true);
  });
});
