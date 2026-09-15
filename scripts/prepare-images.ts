#!/usr/bin/env tsx
/**
 * Pipeline chuyển ảnh trang sách (JPG gốc từ download_ebook.py) sang WebP
 * tối ưu rồi upload lên Cloudflare R2.
 *
 * Đầu vào : ../<sourceDir>_images/pages/page-0001.jpg ... (nằm ở thư mục gốc
 *           `kiip/`, một cấp trên thư mục `web/` — chạy script bằng
 *           `npm run prepare-images` để cwd luôn là `web/`). `sourceDir`
 *           lấy từ `src/lib/books.ts` (khác `id` với sách bài tập, vd
 *           `WB_step1` cho id `wb-step1`).
 * Đầu ra  : R2 bucket, key `books/<id>/pages/0001.webp` và
 *           `books/<id>/thumbs/0001.webp`.
 *
 * Dùng:
 *   npm run prepare-images                 # tất cả sách, bỏ qua ảnh đã có
 *   npm run prepare-images -- --book step1 # chỉ 1 sách
 *   npm run prepare-images -- --force      # upload lại kể cả đã có
 *   npm run prepare-images -- --skip-thumbs
 *
 * Cần các biến môi trường trong web/.env.local (xem .env.local.example):
 *   R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME
 */
import { config as loadEnv } from "dotenv";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";

// Next.js tự đọc .env.local, nhưng script standalone này thì không — nạp
// tường minh (ưu tiên .env.local, không ghi đè biến đã có sẵn trong shell).
loadEnv({ path: path.resolve(process.cwd(), ".env.local"), quiet: true });
import sharp from "sharp";
import {
  S3Client,
  PutObjectCommand,
  HeadObjectCommand,
} from "@aws-sdk/client-s3";
import { BOOKS, padPage, type Book } from "../src/lib/books";

const PAGE_QUALITY = 85;
const THUMB_WIDTH = 300;
const THUMB_QUALITY = 80;
const CONCURRENCY = 6;

function requireEnv(name: string): string {
  const v = process.env[name];
  if (!v) {
    console.error(`✗ Thiếu biến môi trường ${name}. Xem web/.env.local.example.`);
    process.exit(1);
  }
  return v;
}

const args = process.argv.slice(2);
const force = args.includes("--force");
const skipThumbs = args.includes("--skip-thumbs");
const bookFilterIdx = args.indexOf("--book");
const bookFilter = bookFilterIdx >= 0 ? args[bookFilterIdx + 1] : undefined;

const ACCOUNT_ID = requireEnv("R2_ACCOUNT_ID");
const ACCESS_KEY_ID = requireEnv("R2_ACCESS_KEY_ID");
const SECRET_ACCESS_KEY = requireEnv("R2_SECRET_ACCESS_KEY");
const BUCKET = requireEnv("R2_BUCKET_NAME");

const s3 = new S3Client({
  region: "auto",
  endpoint: `https://${ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: { accessKeyId: ACCESS_KEY_ID, secretAccessKey: SECRET_ACCESS_KEY },
});

/** Chạy tối đa `limit` tác vụ song song. */
async function pMap<T, R>(
  items: T[],
  limit: number,
  fn: (item: T, index: number) => Promise<R>
): Promise<R[]> {
  const results: R[] = new Array(items.length);
  let next = 0;
  async function worker() {
    while (true) {
      const i = next++;
      if (i >= items.length) return;
      results[i] = await fn(items[i], i);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, worker));
  return results;
}

async function objectExists(key: string): Promise<boolean> {
  try {
    await s3.send(new HeadObjectCommand({ Bucket: BUCKET, Key: key }));
    return true;
  } catch {
    return false;
  }
}

async function uploadWebp(key: string, buffer: Buffer) {
  await s3.send(
    new PutObjectCommand({
      Bucket: BUCKET,
      Key: key,
      Body: buffer,
      ContentType: "image/webp",
      CacheControl: "public, max-age=31536000, immutable",
    })
  );
}

interface PageResult {
  page: number;
  status: "uploaded" | "skipped" | "error";
  error?: string;
}

async function processPage(book: Book, page: number): Promise<PageResult> {
  const srcPath = path.resolve(
    process.cwd(),
    "..",
    `${book.sourceDir}_images`,
    "pages",
    `page-${padPage(page)}.jpg`
  );
  const pageKey = `books/${book.id}/pages/${padPage(page)}.webp`;
  const thumbKey = `books/${book.id}/thumbs/${padPage(page)}.webp`;

  try {
    const [pageDone, thumbDone] = force
      ? [false, false]
      : await Promise.all([
          objectExists(pageKey),
          skipThumbs ? Promise.resolve(true) : objectExists(thumbKey),
        ]);

    if (pageDone && thumbDone) {
      return { page, status: "skipped" };
    }

    await stat(srcPath); // báo lỗi rõ ràng nếu thiếu file nguồn
    const srcBuffer = await readFile(srcPath);

    if (!pageDone) {
      const webp = await sharp(srcBuffer).webp({ quality: PAGE_QUALITY }).toBuffer();
      await uploadWebp(pageKey, webp);
    }

    if (!skipThumbs && !thumbDone) {
      const thumbWebp = await sharp(srcBuffer)
        .resize({ width: THUMB_WIDTH })
        .webp({ quality: THUMB_QUALITY })
        .toBuffer();
      await uploadWebp(thumbKey, thumbWebp);
    }

    return { page, status: "uploaded" };
  } catch (err) {
    return { page, status: "error", error: (err as Error).message };
  }
}

async function processBook(book: Book) {
  console.log(`\n[${book.id}] ${book.titleVi} — ${book.totalPages} trang`);
  const pages = Array.from({ length: book.totalPages }, (_, i) => i + 1);
  let uploaded = 0;
  let skipped = 0;
  const errors: PageResult[] = [];

  let done = 0;
  await pMap(pages, CONCURRENCY, async (page) => {
    const result = await processPage(book, page);
    done++;
    if (result.status === "uploaded") uploaded++;
    else if (result.status === "skipped") skipped++;
    else errors.push(result);

    if (done % 20 === 0 || done === pages.length) {
      process.stdout.write(
        `\r  ${done}/${pages.length} (mới: ${uploaded}, bỏ qua: ${skipped}, lỗi: ${errors.length})   `
      );
    }
  });
  console.log();

  for (const e of errors) {
    console.error(`  ✗ trang ${e.page}: ${e.error}`);
  }

  return { bookId: book.id, uploaded, skipped, errors: errors.length };
}

async function main() {
  const books = bookFilter ? BOOKS.filter((b) => b.id === bookFilter) : BOOKS;
  if (books.length === 0) {
    console.error(`Không tìm thấy sách với id "${bookFilter}". Có: ${BOOKS.map((b) => b.id).join(", ")}`);
    process.exit(1);
  }

  console.log(`Bucket: ${BUCKET}  |  force=${force}  |  skipThumbs=${skipThumbs}`);
  const summaries = [];
  for (const book of books) {
    summaries.push(await processBook(book));
  }

  console.log("\n=== Tổng kết ===");
  let totalErrors = 0;
  for (const s of summaries) {
    console.log(
      `  ${s.bookId}: mới ${s.uploaded}, bỏ qua ${s.skipped}, lỗi ${s.errors}`
    );
    totalErrors += s.errors;
  }
  if (totalErrors > 0) {
    console.error(`\n✗ Có ${totalErrors} lỗi — chạy lại lệnh này (idempotent) để thử lại.`);
    process.exit(1);
  }
  console.log("\n✓ Xong. Nhớ kiểm tra vài URL ảnh public trước khi build UI dựa vào chúng.");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
