#!/usr/bin/env tsx
/**
 * Sinh biểu tượng PWA từ một hình SVG viết tay ở ngay dưới.
 *
 * Icon phải là PNG: manifest cho phép SVG nhưng Android vẫn dựng lối tắt màn
 * hình chính bằng PNG, và `maskable` thì bắt buộc raster để hệ điều hành cắt
 * theo hình nó muốn.
 *
 * Khác các script khác trong thư mục này, kết quả ở đây ĐƯỢC commit vào git:
 * icon gần như không bao giờ đổi, mà bắt mọi lần build phải chạy lại sharp
 * chỉ để ra đúng hai file y hệt thì không đáng.
 *
 * Chạy lại khi đổi hình:
 *   npm run make-icons
 */
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";

const OUT_DIR = path.resolve(process.cwd(), "public", "icons");

/**
 * Trang sách đang mở, nét dày, trên nền đen — cùng tông với giao diện.
 *
 * Lề rộng 20% quanh hình: `maskable` bị hệ điều hành cắt tròn hoặc cắt
 * squircle tuỳ máy, phần nằm ngoài vòng an toàn đó coi như mất.
 */
const SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" fill="#0a0a0a"/>
  <g fill="none" stroke="#ffffff" stroke-width="26"
     stroke-linecap="round" stroke-linejoin="round">
    <path d="M256 176c-28-24-64-32-102-32h-42v192h42c38 0 74 8 102 32"/>
    <path d="M256 176c28-24 64-32 102-32h42v192h-42c-38 0-74 8-102 32"/>
    <path d="M256 176v192"/>
  </g>
</svg>`;

const SIZES = [192, 512];

/**
 * Ảnh xem trước khi chia sẻ link (Facebook, Zalo, Messenger…) — 1200×630 là
 * tỉ lệ cả Facebook lẫn Zalo hiện trọn không cắt. Cùng hình quyển sách với
 * icon, thêm tên và một dòng nói site dùng để làm gì: người lướt nhóm chỉ
 * liếc qua ảnh chứ ít đọc chữ bên dưới.
 *
 * Chữ dựng bằng font hệ thống lúc chạy script (không nhúng font) — kết quả
 * là PNG commit vào git nên máy build không cần có font đó.
 */
const OG_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 630">
  <rect width="1200" height="630" fill="#0a0a0a"/>
  <g transform="translate(90 187) scale(0.5)" fill="none" stroke="#ffffff" stroke-width="26"
     stroke-linecap="round" stroke-linejoin="round">
    <path d="M256 176c-28-24-64-32-102-32h-42v192h42c38 0 74 8 102 32"/>
    <path d="M256 176c28-24 64-32 102-32h42v192h-42c-38 0-74 8-102 32"/>
    <path d="M256 176v192"/>
  </g>
  <g font-family="Segoe UI, Arial, Helvetica, sans-serif" fill="#ffffff">
    <text x="380" y="290" font-size="96" font-weight="700">VieTopik</text>
    <text x="380" y="360" font-size="38" fill="#d4d4d4">Học tiếng Hàn KIIP &amp; TOPIK cho người Việt</text>
    <text x="380" y="420" font-size="30" fill="#a3a3a3">Dịch sách · Ngữ pháp · Luyện đề · Cẩm nang sống ở Hàn</text>
  </g>
</svg>`;

async function main() {
  await mkdir(OUT_DIR, { recursive: true });
  const svg = Buffer.from(SVG);

  for (const size of SIZES) {
    const png = await sharp(svg).resize(size, size).png().toBuffer();
    await writeFile(path.join(OUT_DIR, `icon-${size}.png`), png);
    console.log(`  icons/icon-${size}.png`);
  }

  // Apple bỏ qua manifest, chỉ đọc <link rel="apple-touch-icon">, và không
  // tự bo góc cho ảnh trong suốt — nên bản này để nền đặc, cỡ 180.
  const apple = await sharp(Buffer.from(SVG)).resize(180, 180).png().toBuffer();
  await writeFile(path.join(OUT_DIR, "apple-touch-icon.png"), apple);
  console.log("  icons/apple-touch-icon.png");

  const og = await sharp(Buffer.from(OG_SVG)).png().toBuffer();
  await writeFile(path.resolve(OUT_DIR, "..", "og.png"), og);
  console.log("  og.png");
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
