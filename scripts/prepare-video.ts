#!/usr/bin/env tsx
/**
 * Chuẩn bị video "Học tiếng Hàn qua video" (`/video`): đọc video + phụ đề
 * đã tải bằng yt-dlp ở `../yt-dlp/<playlistId>/`, CHỌN `--count` tập đầu
 * tiên (theo thứ tự playlist) dài hơn `--min-duration` giây — các đoạn ngắn
 * hơn là clip "xem trước tập sau" (~30s), không phải nội dung thật — rồi
 * transcode về H.264 + AAC nếu cần (yt-dlp hay lấy bản AV1 + Opus của
 * YouTube — Safari/iOS không giải mã được hai codec đó, phát ra câm re, xem
 * `transcodeForCompat`) rồi upload (+ ảnh bìa cắt bằng ffmpeg) lên R2, gộp
 * phụ đề `.ko-orig.vtt`/`.vi.vtt` (dạng roll-up của YouTube auto-sub, xem
 * `vtt.ts`) thành cue sạch, ghi đè `content/videos/<playlistId>.json`
 * (COMMIT vào git — nhẹ, chỉ là text).
 *
 * Manifest là TOÀN BỘ tập đang chọn, không cộng dồn qua từng lần chạy: tập
 * nào bị chọn ở lần trước mà lần này không còn đạt tiêu chí (hoặc bị đẩy ra
 * ngoài top N) sẽ bị XOÁ khỏi manifest VÀ khỏi R2. Tập vẫn còn trong danh
 * sách chọn thì bỏ qua việc upload lại (trừ `--force`) — chỉ tính lại cue.
 *
 * Video gốc (~100MB/tập) không tải hết cùng lúc được (yt-dlp có thể bị
 * YouTube chặn tạm giữa chừng) — những tập chưa có file .mp4 hoàn chỉnh
 * (không phải file .part hay file trung gian `.fNNN.mp4` chưa merge) bị bỏ
 * qua khỏi vòng chọn, không tính là "không đạt thời lượng".
 *
 * Dùng:
 *   npm run prepare-video                        # 10 tập đầu dài hơn 60s
 *   npm run prepare-video -- --count 20           # nhiều/ít hơn 10 tập
 *   npm run prepare-video -- --min-duration 120   # ngưỡng dài khác 60s
 *   npm run prepare-video -- --force              # upload lại kể cả đã có
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
  DeleteObjectCommand,
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
const countIdx = args.indexOf("--count");
const count = countIdx >= 0 ? Number(args[countIdx + 1]) : 10;
const minDurIdx = args.indexOf("--min-duration");
const minDuration = minDurIdx >= 0 ? Number(args[minDurIdx + 1]) : 60;

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

async function deleteObject(key: string) {
  try {
    await s3.send(new DeleteObjectCommand({ Bucket: BUCKET, Key: key }));
  } catch {
    /* không có thì thôi */
  }
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

/**
 * yt-dlp lấy "best video/audio" của YouTube, mà với video mới thường là
 * AV1 (hình) + Opus (tiếng) đóng gói trong .mp4 — Chrome/Firefox/Android
 * phát được (nên "máy khác vẫn xem được"), nhưng Safari/iOS KHÔNG giải mã
 * được AV1 lẫn Opus nên `<video>` câm luôn, không lỗi rõ ràng nào bắn ra.
 *
 * Chuyển hẳn sang H.264 + AAC (10.5.2020~ mọi iPhone đều phát được) trước
 * khi upload. File đã sẵn H.264 thì chỉ remux (đưa moov atom lên đầu cho
 * tua nhanh hơn) — không encode lại cho đỡ tốn thời gian.
 */
async function needsTranscode(mp4Path: string): Promise<boolean> {
  try {
    const { stdout } = await run("ffprobe", [
      "-v", "error", "-select_streams", "v:0",
      "-show_entries", "stream=codec_name",
      "-of", "default=noprint_wrappers=1:nokey=1",
      mp4Path,
    ]);
    return stdout.trim() !== "h264";
  } catch {
    return true;
  }
}

async function transcodeForCompat(mp4Path: string, tmpDir: string, id: string): Promise<string> {
  const outPath = path.join(tmpDir, `${id}.compat.mp4`);
  const transcode = await needsTranscode(mp4Path);
  const args = transcode
    ? [
        "-y", "-loglevel", "error", "-nostats", "-i", mp4Path,
        "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p",
        "-c:a", "aac", "-b:a", "160k",
        "-movflags", "+faststart",
        outPath,
      ]
    : ["-y", "-loglevel", "error", "-nostats", "-i", mp4Path, "-c", "copy", "-movflags", "+faststart", outPath];
  await run("ffmpeg", args, { maxBuffer: 1024 * 1024 * 32 });
  return outPath;
}

interface SourceItem {
  id: string;
  base: string; // "01 - 지붕뚫고 하이킥 ..., #01" (không đuôi file)
  koVttPath: string;
  mp4Path: string;
}

async function findAllSourceItems(srcDir: string): Promise<SourceItem[]> {
  const files = await readdir(srcDir);
  const items: SourceItem[] = [];
  for (const file of files) {
    const m = /^(\d+) - (.+)\.ko-orig\.vtt$/.exec(file);
    if (!m) continue;
    const base = `${m[1]} - ${m[2]}`;
    items.push({ id: m[1], base, koVttPath: path.join(srcDir, file), mp4Path: path.join(srcDir, `${base}.mp4`) });
  }
  items.sort((a, b) => Number(a.id) - Number(b.id));
  return items;
}

function parseTitle(base: string): { showTitle: string; episode: number; part: number } | null {
  const rest = base.replace(/^\d+ - /, "");
  const m = /^(.*?),\s*(\d+)회,\s*EP\d+,\s*#(\d+)$/.exec(rest);
  if (!m) return null;
  return { showTitle: m[1].replace(/\s+/g, " ").trim(), episode: Number(m[2]), part: Number(m[3]) };
}

interface Selected {
  item: SourceItem;
  duration: number | null;
}

/** Duyệt theo thứ tự playlist, đo thời lượng từng tập ĐÃ có mp4 hoàn chỉnh,
 * chọn `count` tập đầu tiên dài hơn `minDuration` giây. */
async function selectItems(all: SourceItem[]): Promise<{ selected: Selected[]; notReadyCount: number; tooShortCount: number }> {
  const selected: Selected[] = [];
  let notReadyCount = 0;
  let tooShortCount = 0;
  for (const item of all) {
    if (selected.length >= count) break;
    if (!existsSync(item.mp4Path)) {
      notReadyCount++;
      continue;
    }
    const duration = await probeDuration(item.mp4Path);
    if (duration === null || duration < minDuration) {
      tooShortCount++;
      continue;
    }
    selected.push({ item, duration });
  }
  return { selected, notReadyCount, tooShortCount };
}

async function processItem(sel: Selected, tmpDir: string): Promise<ManifestEntry | null> {
  const { item, duration } = sel;
  const titleInfo = parseTitle(item.base);
  if (!titleInfo) {
    console.log(`  ⏭ #${item.id}: không khớp mẫu tên file, bỏ qua — "${item.base}"`);
    return null;
  }

  const koCues = parseVtt(await readFile(item.koVttPath, "utf8"));
  const viVttPath = path.join(path.dirname(item.mp4Path), `${item.base}.vi.vtt`);
  const hasVi = existsSync(viVttPath);
  const viCues = hasVi ? parseVtt(await readFile(viVttPath, "utf8")) : [];

  const posterPath = path.join(tmpDir, `${item.id}.jpg`);
  const hasPoster = await makePoster(item.mp4Path, duration, posterPath);

  // Transcode TỐN THỜI GIAN (vài chục giây/tập) — chỉ làm khi thật sự sắp
  // upload, không phải mỗi lần chạy script.
  const videoKey = `videos/${PLAYLIST_ID}/${item.id}.mp4`;
  const willUploadVideo = force || !(await objectExists(videoKey));
  const videoStatus = willUploadVideo
    ? await uploadFile(videoKey, await transcodeForCompat(item.mp4Path, tmpDir, item.id), "video/mp4")
    : ("skipped" as const);
  const posterStatus = hasPoster
    ? await uploadFile(`videos/${PLAYLIST_ID}/${item.id}.jpg`, posterPath, "image/jpeg")
    : "skipped";

  console.log(
    `  ✓ #${item.id} Tập ${titleInfo.episode} · Phần ${titleInfo.part} (${Math.round(duration ?? 0)}s) — video ${videoStatus}, ảnh bìa ${posterStatus}${hasVi ? "" : " (chưa có phụ đề tiếng Việt)"}`
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
  const previousIds: string[] = existsSync(manifestPath)
    ? (JSON.parse(await readFile(manifestPath, "utf8")) as ManifestEntry[]).map((e) => e.id)
    : [];

  const all = await findAllSourceItems(srcDir);
  const { selected, notReadyCount, tooShortCount } = await selectItems(all);
  console.log(
    `Bucket: ${BUCKET}  |  playlist: ${PLAYLIST_ID}  |  chọn ${selected.length}/${count} tập dài hơn ${minDuration}s` +
      `  |  bỏ qua: ${tooShortCount} tập ngắn, ${notReadyCount} tập chưa tải xong  |  force=${force}`
  );

  const tmpDir = await mkdtemp(path.join(tmpdir(), "prepare-video-"));
  const manifest: ManifestEntry[] = [];
  try {
    for (const sel of selected) {
      const entry = await processItem(sel, tmpDir);
      if (entry) manifest.push(entry);
    }
  } finally {
    await rm(tmpDir, { recursive: true, force: true });
  }
  manifest.sort((a, b) => a.order - b.order);

  const selectedIds = new Set(manifest.map((e) => e.id));
  const removedIds = previousIds.filter((id) => !selectedIds.has(id));
  for (const id of removedIds) {
    await deleteObject(`videos/${PLAYLIST_ID}/${id}.mp4`);
    await deleteObject(`videos/${PLAYLIST_ID}/${id}.jpg`);
    console.log(`  ✗ Đã xoá tập #${id} khỏi R2 (không còn trong danh sách chọn).`);
  }

  await writeFile(manifestPath, JSON.stringify(manifest), "utf8");

  console.log(`\n=== Tổng kết ===`);
  console.log(`  Sẵn sàng: ${manifest.length}/${count}`);
  if (removedIds.length > 0) console.log(`  Đã gỡ: ${removedIds.map((id) => `#${id}`).join(", ")}`);
  if (manifest.length < count) {
    console.log(`  Chưa đủ ${count} — chạy lại lệnh này sau khi yt-dlp tải thêm.`);
  }
  console.log(`  Đã ghi ${manifestPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
