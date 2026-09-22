#!/usr/bin/env tsx
/**
 * Nhập đề TOPIK từ bộ dữ liệu chính thức (bản v2: JSON + ảnh + mp3, lấy từ
 * dịch vụ "토픽 기출문제 풀어보기" của topik.go.kr) vào app.
 *
 *   npm run import-topik                              # mọi kỳ, TOPIK I
 *   npm run import-topik -- --src ../topik/Chinh-thuc-v2 --round 102
 *
 * Làm gì:
 * 1. Đọc <src>/<kỳ>/topik1-listening.json + topik1-reading.json, làm sạch
 *    HTML (xem `lib/exam-html`), suy ra khoảng câu của từng khối "※ [a~b]",
 *    ghép mốc thời gian từng câu nghe nếu có (content/exams/marks/…), rồi
 *    ghi content/exams/<kỳ>-topik1.json — file này COMMIT vào git.
 * 2. Chép tài nguyên sang public/img/exams/<kỳ>/: ảnh đổi sang WebP, file
 *    nghe giữ nguyên — thư mục này KHÔNG commit; đẩy lên R2 bằng
 *    `npm run prepare-exams`.
 *
 * TOPIK II chưa nhập: cần làm phần viết (câu 51–54) mới chấm đủ 300 điểm.
 */
import { copyFile, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";
import { sanitizeExamHtml, webpName } from "../src/lib/exam-html";
import { examAssetBase, type Content, type Exam, type ExamGroup, type ExamQuestion, type ExamSection } from "../src/lib/exam-types";

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
interface Marks {
  intro?: [number, number];
  groups: { from: number; audio?: [number, number]; dialogue?: [number, number] }[];
  questions: { no: number; audio: [number, number] }[];
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
    };
  });

  return {
    id: src.section,
    title: src.title,
    minutes: src.durationMinutes,
    ...(src.audio ? { audio: `audio/${path.basename(src.audio)}` } : {}),
    ...(marks?.intro ? { intro: marks.intro } : {}),
    groups,
    questions,
  };
}

async function copyAssets(round: number, sections: SrcSection[]) {
  const srcDir = path.join(SRC, String(round));
  const outDir = path.join(OUT_ASSETS, String(round));
  await mkdir(path.join(outDir, "images"), { recursive: true });
  await mkdir(path.join(outDir, "audio"), { recursive: true });

  // Chỉ chép ảnh mà đề TOPIK I thật sự dùng (thư mục gốc có cả ảnh TOPIK II).
  const used = new Set<string>();
  for (const s of sections) {
    const all = JSON.stringify(s);
    for (const m of all.matchAll(/images\/([^"'\\]+?\.(?:png|jpe?g|gif))/gi)) used.add(m[1]);
    if (s.audio) await copyFile(path.join(srcDir, s.audio), path.join(outDir, "audio", path.basename(s.audio)));
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
    const dir = path.join(SRC, String(round));
    const src = await Promise.all(
      (["topik1-listening", "topik1-reading"] as const).map(
        async (f) => JSON.parse(await readFile(path.join(dir, `${f}.json`), "utf8")) as SrcSection
      )
    );
    const id = `${round}-topik1`;
    const sections = await Promise.all(
      src.map((s) => convertSection(s, round, path.join(MARKS, `${id}-${s.section}.json`)))
    );
    const exam: Exam = {
      id,
      round,
      year: YEAR[round] ?? 0,
      level: "TOPIK I",
      assetDir: String(round),
      source: `Đề thi và đáp án chính thức kỳ ${round}${YEAR[round] ? ` (${YEAR[round]})` : ""} — Viện Giáo dục Quốc tế Quốc gia Hàn Quốc (NIIED), topik.go.kr`,
      sections,
    };
    await writeFile(path.join(OUT_JSON, `${id}.json`), JSON.stringify(exam, null, 1) + "\n");
    const images = await copyAssets(round, src);
    const marked = sections[0].questions.filter((q) => q.audio).length;
    console.log(
      `  ${id}: ${sections.map((s) => `${s.id} ${s.questions.length} câu`).join(", ")}, ${images} ảnh` +
        (marked ? `, ${marked} câu nghe có mốc thời gian` : "")
    );
  }
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
