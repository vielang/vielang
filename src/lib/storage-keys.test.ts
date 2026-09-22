import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Chốt lại những chuỗi `kiip-*` KHÔNG được đổi theo tên thương hiệu.
 *
 * App đã đổi tên từ "KIIP Reader" sang "VieTopik", nhưng dữ liệu người dùng
 * thì vẫn nằm dưới các khoá mang tên cũ. Đây là khoá TRA DỮ LIỆU, không phải
 * nhãn hiển thị: đổi chuỗi là trình duyệt đi tìm một chỗ trống hoàn toàn mới.
 *
 * Rủi ro thật nằm ở chỗ hỏng mà KHÔNG báo lỗi. Không có ngoại lệ, không có
 * màn hình đỏ, build vẫn xanh, máy người viết code (chưa có dữ liệu cũ) nhìn
 * vẫn y hệt. Chỉ người dùng đang có dữ liệu mới thấy nét vẽ, ghi âm, tiến độ
 * và sách đã tải offline biến sạch. Một lần tìm-thay "kiip" -> "vietopik" cho
 * gọn là đủ gây ra chuyện đó, nên chặn ngay tại đây.
 *
 * Đọc thẳng file nguồn dạng chữ vì các hằng này cố ý không export — chúng là
 * chi tiết bên trong, nhưng là chi tiết mà đổi thì mất dữ liệu.
 */
function source(...parts: string[]): string {
  return readFileSync(path.resolve(process.cwd(), ...parts), "utf8");
}

describe("khoá IndexedDB", () => {
  it("giữ tên cũ, vì dữ liệu người dùng nằm trong cơ sở dữ liệu tên đó", () => {
    expect(source("src", "lib", "idb-storage.ts")).toContain(
      'const DB_NAME = "kiip-reader"'
    );
  });
});

describe("khoá localStorage", () => {
  it("giữ tên cũ, nếu không thì mất chế độ xem và cờ đã-xem-hướng-dẫn", () => {
    expect(source("src", "lib", "reader-prefs-store.ts")).toContain(
      'name: "kiip-reader-prefs-v1"'
    );
  });
});

describe("tên cache của service worker", () => {
  const sw = source("public", "sw.js");

  it("giữ tiền tố cũ cho cache tài liệu, tài nguyên và ảnh", () => {
    expect(sw).toContain("`kiip-doc-${VERSION}`");
    expect(sw).toContain("`kiip-asset-${VERSION}`");
    expect(sw).toContain("`kiip-image-${VERSION}`");
  });

  it("giữ tiền tố cũ cho sách tải offline", () => {
    // Nguy hiểm nhất trong cả nhóm: cache này cố ý không bao giờ bị dọn tự
    // động. Đổi tiền tố thì sách cũ vừa không còn được tra, vừa lọt khỏi cả
    // diện giữ lại lẫn diện người dùng xoá được — rác nằm lại vĩnh viễn trên
    // máy họ mà sách thì coi như mất.
    expect(sw).toContain('const BOOK_CACHE_PREFIX = "kiip-book-"');
  });

  it("phía app dùng đúng tiền tố mà service worker dùng", () => {
    // Hai file khai báo riêng nên trôi khỏi nhau được; lệch tiền tố là app
    // tải sách vào một cache mà service worker không bao giờ tra tới.
    expect(source("src", "lib", "offline-books.ts")).toContain(
      'const BOOK_CACHE_PREFIX = "kiip-book-"'
    );
  });
});

describe("khoá lịch sử học", () => {
  it("giữ nguyên tên, nếu không thì mất chuỗi ngày và lịch học của người đang dùng", () => {
    expect(source("src", "lib", "activity-store.ts")).toContain('name: "kiip-activity-v1"');
  });
});
