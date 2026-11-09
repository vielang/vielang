import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Chốt lại những chuỗi `kiip-*` KHÔNG được đổi theo tên thương hiệu.
 *
 * App đã đổi tên từ "KIIP Reader" sang "VieLang", nhưng dữ liệu người dùng
 * thì vẫn nằm dưới các khoá mang tên cũ. Đây là khoá TRA DỮ LIỆU, không phải
 * nhãn hiển thị: đổi chuỗi là trình duyệt đi tìm một chỗ trống hoàn toàn mới.
 *
 * Rủi ro thật nằm ở chỗ hỏng mà KHÔNG báo lỗi. Không có ngoại lệ, không có
 * màn hình đỏ, build vẫn xanh, máy người viết code (chưa có dữ liệu cũ) nhìn
 * vẫn y hệt. Chỉ người dùng đang có dữ liệu mới thấy tiến độ, bài làm và
 * lịch học biến sạch. Một lần tìm-thay "kiip" -> "vielang" cho
 * gọn là đủ gây ra chuyện đó, nên chặn ngay tại đây.
 *
 * Đọc thẳng file nguồn dạng chữ vì các hằng này cố ý không export — chúng là
 * chi tiết bên trong, nhưng là chi tiết mà đổi thì mất dữ liệu.
 */
function source(...parts: string[]): string {
  return readFileSync(path.resolve(process.cwd(), ...parts), "utf8");
}

describe("khoá localStorage", () => {
  const keys: [string, string][] = [
    ["progress-store.ts", "kiip-progress-v1"],
    ["quiz-store.ts", "kiip-quiz-v1"],
    ["activity-store.ts", "kiip-activity-v1"],
    ["guide-checklist-store.ts", "kiip-guide-v1"],
  ];
  for (const [file, key] of keys) {
    it(`${file} giữ nguyên tên ${key}`, () => {
      expect(source("src", "lib", file)).toContain(`name: "${key}"`);
    });
  }
});

describe("tên cache của service worker", () => {
  it("giữ tiền tố cũ cho cache tài liệu, tài nguyên và ảnh", () => {
    const sw = source("public", "sw.js");
    expect(sw).toContain("`kiip-doc-${VERSION}`");
    expect(sw).toContain("`kiip-asset-${VERSION}`");
    expect(sw).toContain("`kiip-image-${VERSION}`");
  });
});
