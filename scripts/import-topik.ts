#!/usr/bin/env tsx
/**
 * Nhập đề TOPIK từ bộ dữ liệu chính thức (bản v2: JSON + ảnh + mp3, lấy từ
 * dịch vụ "토픽 기출문제 풀어보기" của topik.go.kr) vào app.
 *
 *   npm run import-topik                              # mọi kỳ, TOPIK I và II
 *   npm run import-topik -- --src ../topik/Chinh-thuc-v2 --round 102
 *
 * Làm gì:
 * 1. Đọc <src>/<kỳ>/topik{1,2}-listening.json + -reading.json, làm sạch
 *    HTML (xem `lib/exam-html`), suy ra khoảng câu của từng khối "※ [a~b]",
 *    ghép mốc thời gian từng câu nghe nếu có (content/exams/marks/…), rồi
 *    ghi content/exams/<kỳ>-topik{1,2}.json — file này COMMIT vào git.
 *    TOPIK II thêm phần viết (topik2-writing.json): đề chỉ có ảnh trang in,
 *    kèm trang đáp án mẫu — cắt thành ảnh từng câu (scripts/writing-crops.ts); thứ tự các phần như buổi thi thật: nghe → viết
 *    (tiết 1) → đọc (tiết 2).
 * 2. Chép tài nguyên sang public/img/exams/<kỳ>/: ảnh đổi sang WebP, file
 *    nghe giữ nguyên — thư mục này KHÔNG commit; đẩy lên R2 bằng
 *    `npm run prepare-exams`.
 */
import { spawnSync } from "node:child_process";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";
import { sanitizeExamHtml, webpName } from "../src/lib/exam-html";
import { cropAnswerKey, cropPage1, cropPage2, writeCrop } from "./writing-crops";
import {
  examAssetBase,
  type Content,
  type Exam,
  type ExamGroup,
  type ExamQuestion,
  type ExamSection,
  type WritingTask,
} from "../src/lib/exam-types";

const args = process.argv.slice(2);
const argOf = (name: string) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const SRC = path.resolve(argOf("--src") ?? "../topik/Chinh-thuc-v2");
const ONLY = argOf("--round");
const OUT_JSON = path.resolve("content/exams");
const OUT_ASSETS = path.resolve("public/img/exams");
const MARKS = path.resolve("content/exams/marks");
/** ffmpeg để chuẩn hoá file nghe (xem `copyAudio`). */
const FFMPEG = process.env.FFMPEG ?? "ffmpeg";

/**
 * Tên file nghe trong app: "<kỳ>_<cấp>-cbr.mp3". Tên MỚI so với file gốc vì
 * tài nguyên trên R2 để cache "immutable" một năm — giữ tên cũ thì trình
 * duyệt đã tải bản gốc sẽ dùng mãi bản đó.
 */
function audioName(srcFile: string): string {
  return path.basename(srcFile).replace(/\.mp3$/i, "-cbr.mp3");
}

/**
 * Chuẩn hoá file nghe thành MP3 CBR 128 kbps "sạch". Trình duyệt tua
 * (`currentTime`) theo vị trí byte: file gốc có bản VBR (tua theo mục lục thô
 * của file) và bản CBR lẫn byte rác giữa các khung — đo trong Chrome thấy
 * lệch từ nửa giây tới vài giây, nghe lại một câu là lẫn tiếng câu bên cạnh.
 * Mã hoá lại thì tua khớp đúng từng khung, và giữ nguyên dòng thời gian (các
 * mốc trong content/exams/marks đo trên file gốc vẫn đúng, lệch < 0.05s).
 */
async function copyAudio(srcFile: string, outFile: string) {
  if (await exists(outFile)) return; // đã chuẩn hoá từ lần nhập trước
  const r = spawnSync(FFMPEG, ["-y", "-loglevel", "error", "-i", srcFile, "-map", "0:a", "-c:a", "libmp3lame", "-b:a", "128k", outFile]);
  if (r.status !== 0) {
    throw new Error(
      `không chuẩn hoá được ${path.basename(srcFile)} — cần ffmpeg (đặt biến FFMPEG=<đường dẫn ffmpeg>): ${r.error?.message ?? r.stderr}`
    );
  }
}

/** Năm tổ chức từng kỳ (theo danh sách đề đã công bố trên topik.go.kr). */
const YEAR: Record<number, number> = {
  35: 2014, 36: 2014, 37: 2014, 41: 2015, 47: 2016, 52: 2017,
  60: 2018, 64: 2019, 83: 2022, 91: 2023, 96: 2024, 102: 2025,
};

// Kiểu của bộ dữ liệu gốc (rút gọn từ Chinh-thuc-v2/types.ts).
interface SrcRich { html: string; text: string }
interface SrcImage { image: string | null; alt: string | null }
type SrcContent = SrcRich | SrcImage;
interface SrcGroup {
  instruction: string;
  example: (SrcRich & { options: string[]; answer: number; optionLayout: number }) | null;
  passageHtml: string | null;
}
interface SrcQuestion {
  no: number; group: number | null; points: number; answer: 1 | 2 | 3 | 4;
  optionLayout: number; prompt: SrcContent; options: SrcContent[];
}
interface SrcSection {
  section: "listening" | "reading"; title: string; durationMinutes: number;
  audio: string | null; groups: SrcGroup[]; questions: SrcQuestion[];
}
interface SrcWriting {
  title: string; durationMinutes: number; questions: number[];
  pages: { image: string }[]; answerKey: { image: string }[];
}
interface Marks {
  intro?: [number, number];
  groups: { from: number; audio?: [number, number]; dialogue?: [number, number] }[];
  /** `play`: đoạn nghe lại riêng câu — chỉ phần lời đọc, bỏ khoảng dừng trả lời. */
  questions: { no: number; audio: [number, number]; play?: [number, number] }[];
}

async function exists(p: string) {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
}

function convertContent(c: SrcContent, base: string): Content {
  if ("image" in c) {
    if (!c.image) return { html: "" };
    return { image: `images/${webpName(path.basename(c.image))}`, ...(c.alt ? { alt: c.alt } : {}) };
  }
  return { html: sanitizeExamHtml(c.html, base) };
}

async function convertSection(src: SrcSection, round: number, marksFile: string): Promise<ExamSection> {
  const base = examAssetBase(String(round));
  const marks: Marks | null = (await exists(marksFile)) ? JSON.parse(await readFile(marksFile, "utf8")) : null;

  const groups: ExamGroup[] = src.groups.map((g, i) => {
    const nos = src.questions.filter((q) => q.group === i).map((q) => q.no);
    if (nos.length === 0) throw new Error(`khối ${i} không có câu nào`);
    const from = Math.min(...nos);
    const m = marks?.groups.find((x) => x.from === from);
    return {
      from,
      to: Math.max(...nos),
      instruction: g.instruction.replace(/\s+/g, " ").trim(),
      ...(g.example
        ? {
            example: {
              html: sanitizeExamHtml(g.example.html, base),
              options: g.example.options,
              answer: g.example.answer,
              layout: g.example.optionLayout,
            },
          }
        : {}),
      ...(g.passageHtml ? { passage: sanitizeExamHtml(g.passageHtml, base) } : {}),
      ...(m?.audio ? { audio: m.audio } : {}),
      ...(m?.dialogue ? { dialogue: m.dialogue } : {}),
    };
  });

  const questions: ExamQuestion[] = src.questions.map((q) => {
    if (q.options.length !== 4) throw new Error(`câu ${q.no} có ${q.options.length} lựa chọn`);
    const m = marks?.questions.find((x) => x.no === q.no);
    return {
      no: q.no,
      points: q.points,
      answer: q.answer,
      layout: q.optionLayout,
      prompt: convertContent(q.prompt, base),
      options: q.options.map((o) => convertContent(o, base)) as ExamQuestion["options"],
      ...(m ? { audio: m.audio } : {}),
      ...(m?.play ? { replay: m.play } : {}),
    };
  });

  return {
    id: src.section,
    title: src.title,
    minutes: src.durationMinutes,
    ...(src.audio ? { audio: `audio/${audioName(src.audio)}` } : {}),
    ...(marks?.intro ? { intro: marks.intro } : {}),
    groups,
    questions,
  };
}

/**
 * Câu viết theo khung đề TOPIK II (từ kỳ 35, 2014): 51–52 điền ㉠ ㉡ (10
 * điểm), 53 đoạn 200–300 chữ (30 điểm), 54 bài 600–700 chữ (50 điểm).
 */
const WRITING: Omit<WritingTask, "image" | "answer">[] = [
  { no: 51, points: 10, kind: "blanks" },
  { no: 52, points: 10, kind: "blanks" },
  { no: 53, points: 30, kind: "essay", chars: [200, 300] },
  { no: 54, points: 50, kind: "essay", chars: [600, 700] },
];

function convertWriting(src: SrcWriting): ExamSection {
  if (src.pages.length !== 2) throw new Error(`phần viết có ${src.pages.length} trang đề`);
  // Trang đáp án TOPIK II: trang đầu là đáp án nghe, trang cuối là đáp án
  // đọc, (các) trang giữa là đáp án mẫu + tiêu chí chấm phần viết.
  if (src.answerKey.length < 3) throw new Error(`đáp án TOPIK II chỉ có ${src.answerKey.length} trang`);
  return {
    id: "writing",
    title: src.title,
    minutes: src.durationMinutes,
    groups: [],
    questions: [],
    writing: {
      // Ảnh đề / đáp án mẫu cắt theo từng câu — xem `cutWriting`.
      tasks: WRITING.map((t) => ({ ...t, image: `writing/q${t.no}.webp`, answer: `answer-key/w${t.no}.webp` })),
    },
  };
}

/**
 * Cắt ảnh phần viết theo từng câu (xem `scripts/writing-crops.ts`): đề
 * `writing/q51…q54.webp`, đáp án mẫu `answer-key/w51…w54.webp`.
 */
async function cutWriting(round: number, w: SrcWriting): Promise<number> {
  const srcDir = path.join(SRC, String(round));
  const outDir = path.join(OUT_ASSETS, String(round));
  await mkdir(path.join(outDir, "writing"), { recursive: true });
  await mkdir(path.join(outDir, "answer-key"), { recursive: true });
  const [page1, page2] = w.pages.map((p) => path.join(srcDir, p.image));
  const q12 = await cropPage1(page1);
  const q34 = await cropPage2(page2);
  const keyPages = w.answerKey.slice(1, -1).map((a) => path.join(srcDir, a.image));
  const key = await cropAnswerKey(keyPages);
  for (const no of [51, 52] as const) await writeCrop(page1, q12[no], path.join(outDir, "writing", `q${no}.webp`));
  for (const no of [53, 54] as const) await writeCrop(page2, q34[no], path.join(outDir, "writing", `q${no}.webp`));
  for (const no of [51, 52, 53, 54] as const) {
    await writeCrop(keyPages[key[no].page], key[no].crop, path.join(outDir, "answer-key", `w${no}.webp`), 8);
  }
  return 8;
}

async function copyAssets(round: number, sections: SrcSection[]) {
  const srcDir = path.join(SRC, String(round));
  const outDir = path.join(OUT_ASSETS, String(round));
  await mkdir(path.join(outDir, "images"), { recursive: true });
  await mkdir(path.join(outDir, "audio"), { recursive: true });

  // Chỉ chép ảnh mà đề thật sự dùng (thư mục gốc có ảnh của cả hai cấp).
  const used = new Set<string>();
  for (const s of sections) {
    const all = JSON.stringify(s);
    for (const m of all.matchAll(/images\/([^"'\\]+?\.(?:png|jpe?g|gif))/gi)) used.add(m[1]);
    if (s.audio) {
      const from = path.join(srcDir, s.audio);
      await copyAudio(from, path.join(outDir, "audio", audioName(from)));
    }
  }
  let converted = 0;
  for (const file of used) {
    const from = path.join(srcDir, "images", file);
    if (!(await exists(from))) throw new Error(`thiếu ảnh ${from}`);
    await sharp(from).webp({ quality: 85 }).toFile(path.join(outDir, "images", webpName(file)));
    converted++;
  }
  return converted;
}

async function main() {
  const rounds = (await readdir(SRC))
    .filter((d) => /^\d+$/.test(d) && (!ONLY || d === ONLY))
    .map(Number)
    .sort((a, b) => a - b);

  for (const round of rounds) {
    for (const level of [1, 2] as const) await importExam(round, level);
  }
}

async function importExam(round: number, level: 1 | 2) {
  const dir = path.join(SRC, String(round));
  const src = await Promise.all(
    (["listening", "reading"] as const).map(
      async (f) => JSON.parse(await readFile(path.join(dir, `topik${level}-${f}.json`), "utf8")) as SrcSection
    )
  );
  const id = `${round}-topik${level}`;
  const [listening, reading] = await Promise.all(
    src.map((s) => convertSection(s, round, path.join(MARKS, `${id}-${s.section}.json`)))
  );
  let sections = [listening, reading];
  let writingImages = 0;
  if (level === 2) {
    const w = JSON.parse(await readFile(path.join(dir, "topik2-writing.json"), "utf8")) as SrcWriting;
    sections = [listening, convertWriting(w), reading];
    writingImages = await cutWriting(round, w);
  }
  const exam: Exam = {
    id,
    round,
    year: YEAR[round] ?? 0,
    level: level === 1 ? "TOPIK I" : "TOPIK II",
    assetDir: String(round),
    source: `Đề thi và đáp án chính thức kỳ ${round}${YEAR[round] ? ` (${YEAR[round]})` : ""} — Viện Giáo dục Quốc tế Quốc gia Hàn Quốc (NIIED), topik.go.kr`,
    sections,
  };
  await writeFile(path.join(OUT_JSON, `${id}.json`), JSON.stringify(exam, null, 1) + "\n");
  const images = (await copyAssets(round, src)) + writingImages;
  const marked = listening.questions.filter((q) => q.audio).length;
  console.log(
    `  ${id}: ${sections.map((s) => `${s.id} ${s.questions.length || s.writing?.tasks.length} câu`).join(", ")}, ${images} ảnh` +
      (marked ? `, ${marked} câu nghe có mốc thời gian` : "")
  );
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
