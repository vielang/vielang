#!/usr/bin/env tsx
/**
 * Đẩy tài nguyên đề thi (ảnh trang WebP + file nghe MP3) lên R2.
 *
 *   npm run prepare-exams                    # mọi đề, bỏ qua file đã có
 *   npm run prepare-exams -- --exam 102       # chỉ một kỳ
 *   npm run prepare-exams -- --force         # đẩy lại kể cả đã có
 *
 * Nguồn: public/img/exams/<kỳ>/** (ảnh WebP + file nghe, do `import-topik`
 * chép sang). Đích: exams/<kỳ>/** trên bucket — đúng chỗ mà `examAssetUrl`
 * (qua rewrite /img/exams) và `examAudioUrl` (thẳng R2) đọc.
 *
 * Thư mục nguồn KHÔNG commit (xem .gitignore): lúc dev Next phục vụ thẳng từ
 * public/, còn bản deploy không có nó nên phải có bản trên R2.
 *
 * Cần: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME
 * trong .env.local (xem .env.local.example).
 */
import { config as loadEnv } from "dotenv";
import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import { S3Client, PutObjectCommand, HeadObjectCommand } from "@aws-sdk/client-s3";

loadEnv({ path: ".env.local" });

const CONCURRENCY = 6;

function requireEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    console.error(`✗ Thiếu biến môi trường ${name} trong .env.local`);
    process.exit(1);
  }
  return value;
}

const args = process.argv.slice(2);
const force = args.includes("--force");
const examIdx = args.indexOf("--exam");
const examFilter = examIdx >= 0 ? args[examIdx + 1] : undefined;

const ACCOUNT_ID = requireEnv("R2_ACCOUNT_ID");
const BUCKET = requireEnv("R2_BUCKET_NAME");
const s3 = new S3Client({
  region: "auto",
  endpoint: `https://${ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: {
    accessKeyId: requireEnv("R2_ACCESS_KEY_ID"),
    secretAccessKey: requireEnv("R2_SECRET_ACCESS_KEY"),
  },
});

const ROOT = path.resolve(process.cwd(), "public", "img", "exams");
const TYPES: Record<string, string> = { ".webp": "image/webp", ".mp3": "audio/mpeg" };

async function walk(dir: string): Promise<string[]> {
  const out: string[] = [];
  for (const name of await readdir(dir)) {
    const full = path.join(dir, name);
    if ((await stat(full)).isDirectory()) out.push(...(await walk(full)));
    else if (TYPES[path.extname(name)]) out.push(full);
  }
  return out;
}

async function exists(key: string): Promise<boolean> {
  try {
    await s3.send(new HeadObjectCommand({ Bucket: BUCKET, Key: key }));
    return true;
  } catch {
    return false;
  }
}

async function main() {
  let exams: string[];
  try {
    exams = (await readdir(ROOT)).filter((e) => !examFilter || e === examFilter);
  } catch {
    console.error(`✗ Chưa có ${ROOT} — chạy "npm run import-topik" (đã gỡ cùng đề TOPIK; TOEIC sẽ có script nhập riêng) trước.`);
    process.exit(1);
  }

  for (const examId of exams) {
    const files = await walk(path.join(ROOT, examId));
    console.log(`\n[${examId}] ${files.length} file`);
    let uploaded = 0;
    let skipped = 0;
    let failed = 0;
    let next = 0;
    await Promise.all(
      Array.from({ length: CONCURRENCY }, async () => {
        while (next < files.length) {
          const file = files[next++];
          const rel = path.relative(path.join(ROOT, examId), file).split(path.sep).join("/");
          const key = `exams/${examId}/${rel}`;
          try {
            if (!force && (await exists(key))) {
              skipped++;
              continue;
            }
            await s3.send(
              new PutObjectCommand({
                Bucket: BUCKET,
                Key: key,
                Body: await readFile(file),
                ContentType: TYPES[path.extname(file)],
                CacheControl: "public, max-age=31536000, immutable",
              })
            );
            uploaded++;
          } catch (err) {
            failed++;
            console.error(`  ✗ ${key}: ${(err as Error).message}`);
          }
        }
      })
    );
    console.log(`  ✓ đẩy lên ${uploaded}, bỏ qua ${skipped}, lỗi ${failed}`);
  }
}

main();
