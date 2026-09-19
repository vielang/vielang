#!/usr/bin/env tsx
/**
 * Chép font của Excalidraw từ node_modules sang public/excalidraw/fonts để
 * app tự phục vụ.
 *
 * Mặc định Excalidraw tải font từ CDN esm.sh (xem `ASSETS_FALLBACK_URL`
 * trong gói). Với app này thì không ổn: bảng vẽ là chỗ người học ghi chú
 * tiếng Hàn, mà chữ Hàn nằm trong bộ Xiaolai — font dự phòng của Excalifont.
 * Mất CDN là chữ Hàn trong bản vẽ đổi mặt chữ, hoặc tệ hơn là không hiện.
 *
 * Chép cả thư mục `fonts` (~14MB, riêng Xiaolai 13MB chia thành 209 file con
 * theo unicode-range) — trình duyệt chỉ tải đúng phần chứa ký tự đang dùng,
 * nên nặng ở đĩa/deploy chứ không nặng ở đường truyền.
 *
 * `src/lib/excalidraw-assets.ts` trỏ `window.EXCALIDRAW_ASSET_PATH` vào thư
 * mục này. CDN vẫn được giữ làm `src` dự phòng trong @font-face, nên bản
 * chép thiếu/cũ thì chậm chứ không vỡ.
 *
 * Thư mục đích KHÔNG commit vào git (xem .gitignore) — luôn chép lại từ
 * node_modules qua npm lifecycle hook (`predev`, `prebuild`), giống cách
 * `build-content.ts` sinh JSON nội dung.
 *
 * Dùng:
 *   npm run copy-excalidraw-fonts
 *   npm run copy-excalidraw-fonts -- --force   # chép lại kể cả khi đã đúng
 */
import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";

const DEST_ROOT = path.resolve(process.cwd(), "public", "excalidraw");
const DEST_FONTS = path.join(DEST_ROOT, "fonts");
/** Đánh dấu bản đã chép, để lần chạy sau khỏi copy lại 200+ file vô ích. */
const STAMP = path.join(DEST_ROOT, ".version");

/**
 * Lần theo entry thật của gói thay vì đoán đường `node_modules/...` — chỗ
 * cài đặt phụ thuộc vào cách hoist của npm.
 */
function excalidrawDist(): string {
  const require = createRequire(import.meta.url);
  // `exports` của gói không mở `./package.json`, nên resolve entry rồi đi
  // ngược lên: <pkg>/dist/prod/index.js -> <pkg>/dist/prod.
  return path.dirname(require.resolve("@excalidraw/excalidraw"));
}

async function packageVersion(distDir: string): Promise<string> {
  const pkgPath = path.resolve(distDir, "..", "..", "package.json");
  const pkg = JSON.parse(await readFile(pkgPath, "utf8")) as { version?: string };
  return pkg.version ?? "unknown";
}

async function currentStamp(): Promise<string | null> {
  try {
    return (await readFile(STAMP, "utf8")).trim();
  } catch {
    return null;
  }
}

async function main() {
  const force = process.argv.includes("--force");
  const distDir = excalidrawDist();
  const version = await packageVersion(distDir);

  if (!force && (await currentStamp()) === version) {
    console.log(`  font Excalidraw ${version}: đã có, bỏ qua`);
    return;
  }

  // Xoá sạch trước khi chép: nâng cấp gói thường đổi tên file (hash nội dung
  // nằm trong tên), chép đè sẽ để lại một đống file mồ côi.
  await rm(DEST_FONTS, { recursive: true, force: true });
  await mkdir(DEST_ROOT, { recursive: true });
  await cp(path.join(distDir, "fonts"), DEST_FONTS, { recursive: true });
  await writeFile(STAMP, `${version}\n`, "utf8");

  console.log(`  font Excalidraw ${version}: đã chép sang public/excalidraw/fonts`);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
