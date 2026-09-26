"use client";

import type { StateStorage } from "zustand/middleware";

/**
 * Kho lưu IndexedDB cho persist của zustand, kèm đường nâng cấp một chiều từ
 * localStorage.
 *
 * Vì sao phải đổi: localStorage trần khoảng 5MB cho CẢ origin — dùng chung
 * giữa tiến độ, bài giảng, bảng vẽ và nét vẽ trên trang. Đo thử thì 30 nét
 * mỗi trang × 200 trang đã là ~3.3MB, tức một cuốn sách thôi là đã chạm mép.
 * IndexedDB cho vài trăm MB tới vài GB tuỳ dung lượng đĩa, và là bước đệm
 * đúng cho việc đồng bộ tài khoản về sau (dữ liệu đã nằm ngoài luồng ghi
 * đồng bộ, không còn chặn luồng chính).
 *
 * Cố ý KHÔNG thêm thư viện (idb, idb-keyval): chỗ này chỉ cần một bảng
 * khoá-giá trị, và persist của zustand vốn đã chấp nhận storage trả Promise.
 *
 * Mọi thao tác đều nuốt lỗi và trả về null/không làm gì: IndexedDB bị chặn
 * trong một số chế độ riêng tư, mà mất chú thích thì tiếc chứ không được
 * phép làm vỡ trang đọc.
 */
/**
 * ĐỪNG đổi theo tên thương hiệu. Đây là KHOÁ TÌM DỮ LIỆU, không phải nhãn
 * hiển thị: trình duyệt tra cơ sở dữ liệu đúng theo chuỗi này. App từng tên
 * là "KIIP Reader" và dữ liệu của mọi người đang dùng nằm trong cơ sở dữ
 * liệu mang tên đó. Đổi sang "vielang" là mở một cơ sở dữ liệu rỗng hoàn
 * toàn mới — nét vẽ trên trang, bản ghi âm, đánh dấu, tiến độ đọc, danh sách
 * sách đã tải offline của họ vẫn nằm nguyên trên máy nhưng app không còn
 * nhìn thấy, nhìn ra ngoài y hệt như bị xoá sạch.
 *
 * Muốn đổi thì phải kèm bước chuyển dữ liệu: mở cả tên cũ lẫn tên mới, chép
 * sang, và chỉ xoá bên cũ sau khi chép xong (xem phần di trú từ localStorage
 * ở cuối file — cùng hai quy tắc an toàn đó).
 */
const DB_NAME = "kiip-reader";
const DB_VERSION = 2;
const STORE = "keyval";
/**
 * Bảng riêng cho dữ liệu nhị phân (hiện là bản ghi âm giọng người dùng).
 *
 * Tách khỏi `keyval` vì `keyval` đi qua JSON của zustand persist, mà nhét một
 * đoạn ghi âm vài trăm KB vào JSON thì phải base64 hoá — phình thêm 33% và
 * phải nạp toàn bộ mỗi lần đọc bất cứ thứ gì. IndexedDB chứa thẳng Blob
 * được, nên bản ghi nằm riêng từng cái, đọc cái nào lấy cái đó.
 */
const BLOB_STORE = "blobs";

let dbPromise: Promise<IDBDatabase | null> | null = null;

function openDb(): Promise<IDBDatabase | null> {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve) => {
    if (typeof indexedDB === "undefined") {
      resolve(null);
      return;
    }
    try {
      const request = indexedDB.open(DB_NAME, DB_VERSION);
      request.onupgradeneeded = () => {
        // Kiểm tra từng bảng thay vì làm theo số hiệu phiên bản: người dùng
        // có thể đang ở bất kỳ phiên bản cũ nào, cách này đúng cho mọi lối.
        for (const name of [STORE, BLOB_STORE]) {
          if (!request.result.objectStoreNames.contains(name)) {
            request.result.createObjectStore(name);
          }
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => resolve(null);
      // Tab khác đang giữ bản DB cũ và chặn nâng cấp — đừng treo mãi ở đây.
      request.onblocked = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
  return dbPromise;
}

function run<T>(
  mode: IDBTransactionMode,
  body: (store: IDBObjectStore) => IDBRequest<T>,
  storeName: string = STORE
): Promise<T | null> {
  return openDb().then(
    (db) =>
      new Promise<T | null>((resolve) => {
        if (!db) {
          resolve(null);
          return;
        }
        try {
          const tx = db.transaction(storeName, mode);
          const request = body(tx.objectStore(storeName));
          request.onsuccess = () => resolve(request.result);
          request.onerror = () => resolve(null);
          tx.onabort = () => resolve(null);
        } catch {
          resolve(null);
        }
      })
  );
}

/**
 * Chuyển dữ liệu cũ từ localStorage sang, đúng MỘT lần cho mỗi khoá.
 *
 * Chỉ chép khi IndexedDB chưa có gì dưới khoá đó — người dùng đã vẽ tiếp
 * trên bản mới rồi thì bản localStorage cũ là quá khứ, đè lên là mất bài.
 * Chép xong mới xoá bản cũ, nên nửa chừng có hỏng thì lần sau vẫn chép lại
 * được.
 */
async function migrateFromLocalStorage(name: string): Promise<string | null> {
  let legacy: string | null = null;
  try {
    legacy = localStorage.getItem(name);
  } catch {
    return null;
  }
  if (legacy === null) return null;

  const written = await run("readwrite", (store) => store.put(legacy, name));
  if (written === null) return legacy; // IndexedDB không dùng được — vẫn trả dữ liệu cũ để đọc
  try {
    localStorage.removeItem(name);
  } catch {
    /* xoá không được thì thôi, lần sau `getItem` đã thấy bản IndexedDB nên không chép lại */
  }
  return legacy;
}

/**
 * Đọc/ghi/xoá dữ liệu nhị phân theo khoá — xem `BLOB_STORE`.
 *
 * Lưu dạng `ArrayBuffer` chứ không lưu thẳng `Blob`: Safari đời cũ có lỗi
 * với Blob trong IndexedDB, và ArrayBuffer thì cấu trúc sao chép nào cũng
 * hiểu. Kiểu tệp không mất đi đâu — nó nằm sẵn trong mô tả bản ghi (xem
 * `Recording.mimeType`) và được trả lại lúc đọc.
 *
 * `putBlob` trả về `false` khi không ghi được (IndexedDB bị chặn hoặc hết
 * dung lượng): bản ghi âm nào không lưu nổi thì phải báo cho người dùng biết
 * ngay lúc đó, chứ để họ tưởng đã ghi xong rồi mất thì tệ hơn nhiều.
 */
export async function putBlob(key: string, blob: Blob): Promise<boolean> {
  const buffer = await blob.arrayBuffer();
  const ok = await run("readwrite", (store) => store.put(buffer, key), BLOB_STORE);
  return ok !== null;
}

export async function getBlob(key: string, mimeType: string): Promise<Blob | null> {
  const buffer = await run<ArrayBuffer | undefined>(
    "readonly",
    (store) => store.get(key),
    BLOB_STORE
  );
  return buffer ? new Blob([buffer], { type: mimeType }) : null;
}

export async function deleteBlob(key: string): Promise<void> {
  await run("readwrite", (store) => store.delete(key), BLOB_STORE);
}

/**
 * Báo cho các cửa sổ khác cùng origin biết một khoá vừa đổi.
 *
 * localStorage có sẵn sự kiện `storage` cho việc này, IndexedDB thì không —
 * mà cửa sổ bài giảng riêng (xem `note-window`) đang dựa vào đó để thấy sửa
 * đổi từ cửa sổ chính. `BroadcastChannel` lấp đúng chỗ đó, và cũng KHÔNG gửi
 * lại cho chính nơi vừa phát, đúng như `storage` — không lo vòng lặp.
 */
const CHANNEL_NAME = "kiip-store-changed";

function channel(): BroadcastChannel | null {
  if (typeof BroadcastChannel === "undefined") return null;
  try {
    broadcast ??= new BroadcastChannel(CHANNEL_NAME);
    return broadcast;
  } catch {
    return null;
  }
}
let broadcast: BroadcastChannel | null = null;

/** Nghe thay đổi của MỘT khoá, từ các cửa sổ khác. Trả về hàm huỷ đăng ký. */
export function subscribeToStoreChanges(key: string, onChange: () => void): () => void {
  const ch = channel();
  if (!ch) return () => {};
  const listener = (e: MessageEvent) => {
    if (e.data === key) onChange();
  };
  ch.addEventListener("message", listener);
  return () => ch.removeEventListener("message", listener);
}

/**
 * `markQuota` được gọi sau mỗi lần ghi (true khi hỏng). Hàm đó BẮT BUỘC tự
 * so sánh trước khi `set`: mỗi lần đổi state là persist lại ghi xuống, lại
 * hỏng, lại đổi cờ — không chặn thì thành vòng lặp vô tận.
 */
export function createIdbStorage(markQuota: (exceeded: boolean) => void): StateStorage {
  return {
    getItem: async (name) => {
      const stored = await run<string | undefined>("readonly", (store) => store.get(name));
      if (typeof stored === "string") return stored;
      return migrateFromLocalStorage(name);
    },
    setItem: async (name, value) => {
      const ok = await run("readwrite", (store) => store.put(value, name));
      markQuota(ok === null);
      if (ok !== null) channel()?.postMessage(name);
    },
    removeItem: async (name) => {
      await run("readwrite", (store) => store.delete(name));
      channel()?.postMessage(name);
    },
  };
}
