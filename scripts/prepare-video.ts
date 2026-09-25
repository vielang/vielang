#!/usr/bin/env tsx
/**
 * Chuẩn bị video "Học tiếng Hàn qua video" (`/video`): đọc video + phụ đề
 * đã tải bằng yt-dlp ở `../yt-dlp/<playlistId>/`, upload video (+ ảnh bìa
 * cắt bằng ffmpeg) lên R2, gộp phụ đề `.ko-orig.vtt`/`.vi.vtt` (dạng roll-up
 * của YouTube auto-sub, xem `vtt.ts`) thành cue sạch, ghi ra
 * `content/videos/<playlistId>.json` (COMMIT vào git — nhẹ, chỉ là text).
 *
 * Video gốc (~100MB/tập) không tải hết cùng lúc được (yt-dlp có thể bị
 * YouTube chặn tạm giữa chừng) — script CHỈ xử lý những tập đã có file .mp4
 * hoàn chỉnh (không phải file .part hay file trung gian `.fNNN.mp4` chưa
 * merge), bỏ qua và báo rõ những tập chưa sẵn sàng. Idempotent: chạy lại
 * nhiều lần an toàn, tập nào đã upload thì bỏ qua trừ khi `--force`; manifest
 * cũ được GIỮ LẠI cho tập chưa xử lý lại được ở lần chạy này.
 *
 * Dùng:
 *   npm run prepare-video                    # 10 tập đầu, bỏ qua đã có
 *   npm run prepare-video -- --max 20         # nhiều/ít hơn 10 tập đầu
 *   npm run prepare-video -- --force          # upload lại kể cả đã có
 *
 * Cần các biến môi trường trong .env.local (giống prepare-audio.ts) và
 * `ffmpeg`/`ffprobe` có sẵn trong PATH (dùng để cắt ảnh bìa + đo thời lượng).
 */
import { config as loadEnv } from "dotenv";
import { readdir, readFile, stat, writeFile, mkdtemp, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { createReadStream } from "node:fs";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  S3Client,
  PutObjectCommand,
  HeadObjectCommand,
} from "@aws-sdk/client-s3";
import { parseVtt, type Cue } from "./vtt";

loadEnv({ path: path.resolve(process.cwd(), ".env.local"), quiet: true });
const run = promisify(execFile);

const PLAYLIST_ID = "PLOBbhydezQbfUHGKsM8q9DUyiOFWCRzOw";

function requireEnv(name: string): string {
  const v = process.env[name];
  if (!v) {
    console.error(`✗ Thiếu biến môi trường ${name}. Xem .env.local.example.`);
    process.exit(1);
  }
  return v;
}

const args = process.argv.slice(2);
const force = args.includes("--force");
const maxIdx = args.indexOf("--max");
const max = maxIdx >= 0 ? Number(args[maxIdx + 1]) : 10;

const ACCOUNT_ID = requireEnv("R2_ACCOUNT_ID");
const ACCESS_KEY_ID = requireEnv("R2_ACCESS_KEY_ID");
const SECRET_ACCESS_KEY = requireEnv("R2_SECRET_ACCESS_KEY");
const BUCKET = requireEnv("R2_BUCKET_NAME");

/**
 * Timeout mặc định của SDK quá ngắn cho việc upload file trăm MB khi băng
 * thông đang bị yt-dlp tải playlist chiếm dụng song song — nới ra và cho
 * thử lại vài lần thay vì chết ngang giữa chừng.
 */
const s3 = new S3Client({
  region: "auto",
  endpoint: `https://${ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: { accessKeyId: ACCESS_KEY_ID, secretAccessKey: SECRET_ACCESS_KEY },
  maxAttempts: 5,
  requestHandler: { connectionTimeout: 30_000, requestTimeout: 300_000 },
});

interface ManifestEntry {
  id: string;
  order: number;
  episode: number;
  part: number;
  showTitle: string;
  durationSec: number | null;
  hasVi: boolean;
  koCues: Cue[];
  viCues: Cue[];
}

async function objectExists(key: string): Promise<boolean> {
  try {
    await s3.send(new HeadObjectCommand({ Bucket: BUCKET, Key: key }));
    return true;
  } catch {
    return false;
  }
}

async function uploadFile(key: string, filePath: string, contentType: string) {
  if (!force && (await objectExists(key))) return "skipped" as const;
  const { size } = await stat(filePath);
  await s3.send(
    new PutObjectCommand({
      Bucket: BUCKET,
      Key: key,
      Body: createReadStream(filePath),
      ContentLength: size,
      ContentType: contentType,
      CacheControl: "public, max-age=31536000, immutable",
    })
  );
  return "uploaded" as const;
}

async function probeDuration(mp4Path: string): Promise<number | null> {
  try {
    const { stdout } = await run("ffprobe", [
      "-v", "error",
      "-show_entries", "format=duration",
      "-of", "default=noprint_wrappers=1:nokey=1",
      mp4Path,
    ]);
    const n = Number(stdout.trim());
    return Number.isFinite(n) ? n : null;
  } catch {
    return null;
  }
}

async function makePoster(mp4Path: string, duration: number | null, outPath: string): Promise<boolean> {
  const seekAt = duration && duration > 6 ? 3 : 0;
  try {
    await run("ffmpeg", [
      "-y", "-ss", String(seekAt), "-i", mp4Path,
      "-frames:v", "1", "-q:v", "4", "-vf", "scale=480:-2",
      outPath,
    ]);
    return existsSync(outPath);
  } catch {
    return false;
  }
}

interface SourceItem {
  id: string;
  base: string; // "01 - 지붕뚫고 하이킥 ..., #01" (không đuôi file)
  koVttPath: string;
}

async function findSourceItems(srcDir: string, max: number): Promise<SourceItem[]> {
  const files = await readdir(srcDir);
  const items: SourceItem[] = [];
  for (const file of files) {
    const m = /^(\d+) - (.+)\.ko-orig\.vtt$/.exec(file);
    if (!m) continue;
    items.push({ id: m[1], base: `${m[1]} - ${m[2]}`, koVttPath: path.join(srcDir, file) });
  }
  items.sort((a, b) => Number(a.id) - Number(b.id));
  return items.slice(0, max);
}

function parseTitle(base: string): { showTitle: string; episode: number; part: number } | null {
  const rest = base.replace(/^\d+ - /, "");
  const m = /^(.*?),\s*(\d+)회,\s*EP\d+,\s*#(\d+)$/.exec(rest);
  if (!m) return null;
  return { showTitle: m[1].replace(/\s+/g, " ").trim(), episode: Number(m[2]), part: Number(m[3]) };
}

async function processItem(srcDir: string, item: SourceItem, tmpDir: string): Promise<ManifestEntry | null> {
  const mp4Path = path.join(srcDir, `${item.base}.mp4`);
  if (!existsSync(mp4Path)) {
    console.log(`  ⏭ #${item.id}: chưa có file .mp4 hoàn chỉnh — bỏ qua (chạy lại sau khi tải xong).`);
    return null;
  }
  const titleInfo = parseTitle(item.base);
  if (!titleInfo) {
    console.log(`  ⏭ #${item.id}: không khớp mẫu tên file, bỏ qua — "${item.base}"`);
    return null;
  }

  const koCues = parseVtt(await readFile(item.koVttPath, "utf8"));
  const viVttPath = path.join(srcDir, `${item.base}.vi.vtt`);
  const hasVi = existsSync(viVttPath);
  const viCues = hasVi ? parseVtt(await readFile(viVttPath, "utf8")) : [];

  const duration = await probeDuration(mp4Path);
  const posterPath = path.join(tmpDir, `${item.id}.jpg`);
  const hasPoster = await makePoster(mp4Path, duration, posterPath);

  const videoStatus = await uploadFile(`videos/${PLAYLIST_ID}/${item.id}.mp4`, mp4Path, "video/mp4");
  const posterStatus = hasPoster
    ? await uploadFile(`videos/${PLAYLIST_ID}/${item.id}.jpg`, posterPath, "image/jpeg")
    : "skipped";

  console.log(
    `  ✓ #${item.id} Tập ${titleInfo.episode} · Phần ${titleInfo.part} — video ${videoStatus}, ảnh bìa ${posterStatus}${hasVi ? "" : " (chưa có phụ đề tiếng Việt)"}`
  );

  return {
    id: item.id,
    order: Number(item.id),
    episode: titleInfo.episode,
    part: titleInfo.part,
    showTitle: titleInfo.showTitle,
    durationSec: duration,
    hasVi,
    koCues,
    viCues,
  };
}

async function main() {
  const srcDir = path.resolve(process.cwd(), "..", "yt-dlp", PLAYLIST_ID);
  if (!existsSync(srcDir)) {
    console.error(`✗ Không tìm thấy thư mục ${srcDir}`);
    process.exit(1);
  }

  const manifestPath = path.resolve(process.cwd(), "content", "videos", `${PLAYLIST_ID}.json`);
  const existing: Record<string, ManifestEntry> = existsSync(manifestPath)
    ? Object.fromEntries(
        (JSON.parse(await readFile(manifestPath, "utf8")) as ManifestEntry[]).map((e) => [e.id, e])
      )
    : {};

  const items = await findSourceItems(srcDir, max);
  console.log(`Bucket: ${BUCKET}  |  playlist: ${PLAYLIST_ID}  |  ${items.length} tập trong phạm vi  |  force=${force}`);

  const tmpDir = await mkdtemp(path.join(tmpdir(), "prepare-video-"));
  try {
    for (const item of items) {
      const entry = await processItem(srcDir, item, tmpDir);
      if (entry) existing[item.id] = entry;
    }
  } finally {
    await rm(tmpDir, { recursive: true, force: true });
  }

  const manifest = Object.values(existing).sort((a, b) => a.order - b.order);
  await writeFile(manifestPath, JSON.stringify(manifest), "utf8");

  const ready = manifest.filter((e) => items.some((i) => i.id === e.id));
  const missing = items.filter((i) => !existing[i.id]);
  console.log(`\n=== Tổng kết ===`);
  console.log(`  Sẵn sàng: ${ready.length}/${items.length}`);
  if (missing.length > 0) {
    console.log(`  Còn thiếu: ${missing.map((i) => `#${i.id}`).join(", ")} — chạy lại lệnh này sau khi yt-dlp tải xong.`);
  }
  console.log(`  Đã ghi ${manifestPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
