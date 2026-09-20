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
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
