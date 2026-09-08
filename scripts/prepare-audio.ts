#!/usr/bin/env tsx
/**
 * Upload audio (mp3) từng sách lên Cloudflare R2.
 *
 * Đầu vào : ../SB_<BOOK_ID>_audio/*.mp3 (đã giải nén sẵn từ zip tải về —
 *           xem README mục "Audio"), nằm ở thư mục gốc `kiip/`.
 * Đầu ra  : R2 bucket, key `books/<id>/audio/<file>.mp3` — không convert
 *           (đã là mp3 sẵn), chỉ copy nguyên vẹn.
 *
 * Dùng:
 *   npm run prepare-audio                 # tất cả sách, bỏ qua file đã có
 *   npm run prepare-audio -- --book step1 # chỉ 1 sách
 *   npm run prepare-audio -- --force      # upload lại kể cả đã có
 *
 * Cần các biến môi trường trong web/.env.local (giống prepare-images.ts):
 *   R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME
 */
import { config as loadEnv } from "dotenv";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import {
  S3Client,
  PutObjectCommand,
  HeadObjectCommand,
} from "@aws-sdk/client-s3";
import { BOOKS, type Book } from "../src/lib/books";

loadEnv({ path: path.resolve(process.cwd(), ".env.local"), quiet: true });

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

interface FileResult {
  file: string;
  status: "uploaded" | "skipped" | "error";
  error?: string;
}

async function processFile(book: Book, srcDir: string, file: string): Promise<FileResult> {
  const key = `books/${book.id}/audio/${file}`;
  try {
    if (!force && (await objectExists(key))) {
      return { file, status: "skipped" };
    }
    const buffer = await readFile(path.join(srcDir, file));
    await s3.send(
      new PutObjectCommand({
        Bucket: BUCKET,
        Key: key,
        Body: buffer,
        ContentType: "audio/mpeg",
        CacheControl: "public, max-age=31536000, immutable",
      })
    );
    return { file, status: "uploaded" };
  } catch (err) {
    return { file, status: "error", error: (err as Error).message };
  }
}

async function processBook(book: Book) {
  const srcDir = path.resolve(process.cwd(), "..", `SB_${book.id}_audio`);
  let files: string[];
  try {
    files = (await readdir(srcDir)).filter((f) => f.endsWith(".mp3"));
  } catch {
    console.error(`  ✗ Không tìm thấy thư mục ${srcDir} — xem README mục "Audio".`);
    return { bookId: book.id, uploaded: 0, skipped: 0, errors: 1 };
  }

  console.log(`\n[${book.id}] ${book.titleVi} — ${files.length} file audio`);
  let uploaded = 0;
  let skipped = 0;
  const errors: FileResult[] = [];

  let done = 0;
  await pMap(files, CONCURRENCY, async (file) => {
    const result = await processFile(book, srcDir, file);
    done++;
    if (result.status === "uploaded") uploaded++;
    else if (result.status === "skipped") skipped++;
    else errors.push(result);

    if (done % 20 === 0 || done === files.length) {
      process.stdout.write(
        `\r  ${done}/${files.length} (mới: ${uploaded}, bỏ qua: ${skipped}, lỗi: ${errors.length})   `
      );
    }
  });
  console.log();

  for (const e of errors) {
    console.error(`  ✗ ${e.file}: ${e.error}`);
  }

  return { bookId: book.id, uploaded, skipped, errors: errors.length };
}

async function main() {
  const books = bookFilter ? BOOKS.filter((b) => b.id === bookFilter) : BOOKS;
  if (books.length === 0) {
    console.error(`Không tìm thấy sách với id "${bookFilter}". Có: ${BOOKS.map((b) => b.id).join(", ")}`);
    process.exit(1);
  }

  console.log(`Bucket: ${BUCKET}  |  force=${force}`);
  const summaries = [];
  for (const book of books) {
    summaries.push(await processBook(book));
  }

  console.log("\n=== Tổng kết ===");
  let totalErrors = 0;
  for (const s of summaries) {
    console.log(`  ${s.bookId}: mới ${s.uploaded}, bỏ qua ${s.skipped}, lỗi ${s.errors}`);
    totalErrors += s.errors;
  }
  if (totalErrors > 0) {
    console.error(`\n✗ Có ${totalErrors} lỗi — chạy lại lệnh này (idempotent) để thử lại.`);
    process.exit(1);
  }
  console.log("\n✓ Xong. Nhớ kiểm tra vài URL audio public trước khi build UI dựa vào chúng.");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
