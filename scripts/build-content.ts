#!/usr/bin/env tsx
/**
 * Gộp nội dung biên soạn thủ công thành 1 file JSON/sách để `src/lib` import
 * tĩnh được:
 *
 *   content/notes/<bookId>/<page>.md   -> content/notes/<bookId>.json  (HTML)
 *   content/quiz/<bookId>/<page>.json  -> content/quiz/<bookId>.json
 *   content/grammar/<bookId>/<page>.json -> content/grammar/<bookId>.json
 *   content/answers/<bookId>/<page>.json -> content/answers/<bookId>.json
 *
 * Vì sao Markdown -> HTML ngay ở bước build: note hiển thị/sửa bằng Tiptap
 * (xem components/reader/note-editor.tsx), mà Tiptap đọc/ghi HTML. Convert
 * sẵn ở đây để toàn app chỉ làm việc với 1 định dạng duy nhất — bản gốc và
 * bản người dùng sửa (lưu localStorage) cùng là HTML.
 *
 * Vì sao cần bước gộp: `lib/notes.ts` và `lib/quiz.ts` dùng `import` tĩnh cho
 * các file JSON này (không dùng fs.readFileSync với path dựng động) để né lỗi
 * kinh điển của Next — Output File Tracing không detect được file đọc qua
 * path động ở runtime, dẫn tới ENOENT khi deploy serverless dù chạy đúng ở
 * `next dev`.
 *
 * File .json sinh ra KHÔNG commit vào git (xem .gitignore) — luôn sinh lại từ
 * nguồn qua npm lifecycle hook (`predev`, `prebuild`), nên không bao giờ lệch.
 */
import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { Marked } from "marked";
import { BOOKS } from "../src/lib/books";
import type { QuizItem, QuizSection } from "../src/lib/quiz";
import type { TranslationRegion } from "../src/lib/page-translation";
import type { GrammarPoint } from "../src/lib/page-grammar";
import type { AnswerKey } from "../src/lib/page-answers";

const CONTENT_ROOT = path.resolve(process.cwd(), "content");
const NOTES_ROOT = path.join(CONTENT_ROOT, "notes");
const QUIZ_ROOT = path.join(CONTENT_ROOT, "quiz");
const TRANSLATE_ROOT = path.join(CONTENT_ROOT, "translate");
const GRAMMAR_ROOT = path.join(CONTENT_ROOT, "grammar");
const ANSWERS_ROOT = path.join(CONTENT_ROOT, "answers");

/** Định nghĩa ngữ pháp dài tối đa bao nhiêu ký tự — xem `validateGrammar`. */
const MAX_GRAMMAR_VI = 200;

/** Câu ví dụ dài tối đa bao nhiêu ký tự. Một câu, không phải một đoạn. */
const MAX_GRAMMAR_EXAMPLE = 150;

/**
 * `gfm` (bảng, ~~gạch ngang~~) bật sẵn; `breaks: false` để xuống dòng đơn
 * trong .md không thành <br> — note được viết wrap ~76 cột cho dễ đọc ở
 * editor, không phải để ngắt dòng thật.
 */
const marked = new Marked({ gfm: true, breaks: false, async: false });

/** "0010.md" -> "10" */
function pageKey(file: string): string {
  return String(Number(file.replace(/\.[^.]+$/, "")));
}

async function listFiles(dir: string, ext: string): Promise<string[]> {
  try {
    return (await readdir(dir)).filter((f) => f.endsWith(ext)).sort();
  } catch {
    return []; // sách chưa có nội dung — vẫn sinh ra {} để import không lỗi
  }
}

async function buildNotes(bookId: string): Promise<number> {
  const dir = path.join(NOTES_ROOT, bookId);
  const notes: Record<string, string> = {};

  for (const file of await listFiles(dir, ".md")) {
    const markdown = await readFile(path.join(dir, file), "utf8");
    notes[pageKey(file)] = (marked.parse(markdown) as string).trim();
  }

  await writeFile(
    path.join(NOTES_ROOT, `${bookId}.json`),
    JSON.stringify(notes, null, 2) + "\n",
    "utf8"
  );
  return Object.keys(notes).length;
}

/**
 * Kiểm tra ngay lúc build thay vì để lỗi lòi ra ở trình duyệt: nội dung này
 * soạn tay hàng chục file nên sai sót (thiếu đáp án, `answer` trỏ ra ngoài
 * mảng, trùng id) là chuyện đương nhiên xảy ra — bắt ở đây thì thấy đúng
 * file, đúng câu.
 */
function validateItem(at: string, item: QuizItem): void {
  if (!item.prompt?.trim()) throw new Error(`${at}: thiếu "prompt"`);

  switch (item.kind) {
    case "choice":
      if (!Array.isArray(item.options) || item.options.length < 2) {
        throw new Error(`${at}: "options" cần ít nhất 2 phương án`);
      }
      if (
        !Number.isInteger(item.answer) ||
        item.answer < 0 ||
        item.answer >= item.options.length
      ) {
        throw new Error(
          `${at}: "answer" = ${item.answer} nằm ngoài "options" (0..${item.options.length - 1})`
        );
      }
      return;
    case "fill":
      if (!Array.isArray(item.answers) || item.answers.length === 0) {
        throw new Error(`${at}: "answers" phải có ít nhất 1 đáp án`);
      }
      if (item.answers.some((a) => !a?.trim())) {
        throw new Error(`${at}: "answers" chứa đáp án rỗng`);
      }
      return;
    case "free":
      return; // bài mở, không có đáp án để kiểm
    default:
      throw new Error(
        `${at}: "kind" phải là "choice" | "fill" | "free" (đang là ${
          JSON.stringify((item as { kind?: unknown }).kind)
        })`
      );
  }
}

function validateQuiz(source: string, data: unknown): QuizSection[] {
  if (!Array.isArray(data)) {
    throw new Error(`${source}: phải là 1 mảng mục bài tập (QuizSection[])`);
  }

  // id phải duy nhất trong cả trang, không chỉ trong 1 mục — `quiz-store` lưu
  // bài làm theo (trang, id) nên trùng id là 2 câu ghi đè đáp án của nhau.
  const seen = new Set<string>();

  for (const section of data as QuizSection[]) {
    if (!section?.title?.trim()) {
      throw new Error(`${source}: mục bài tập thiếu "title"`);
    }
    if (!Array.isArray(section.items) || section.items.length === 0) {
      throw new Error(`${source} — ${section.title}: thiếu "items"`);
    }
    for (const item of section.items) {
      const at = `${source} — ${section.title} (id=${item?.id ?? "?"})`;
      if (!item?.id) throw new Error(`${at}: thiếu "id"`);
      if (seen.has(item.id)) throw new Error(`${at}: trùng "id" trong cùng trang`);
      seen.add(item.id);
      validateItem(at, item);
    }
  }

  return data as QuizSection[];
}

async function buildQuiz(bookId: string): Promise<number> {
  const dir = path.join(QUIZ_ROOT, bookId);
  const quiz: Record<string, QuizSection[]> = {};

  for (const file of await listFiles(dir, ".json")) {
    const raw = await readFile(path.join(dir, file), "utf8");
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch (err) {
      throw new Error(`${bookId}/${file}: JSON hỏng — ${(err as Error).message}`);
    }
    quiz[pageKey(file)] = validateQuiz(`${bookId}/${file}`, parsed);
  }

  await writeFile(
    path.join(QUIZ_ROOT, `${bookId}.json`),
    JSON.stringify(quiz, null, 2) + "\n",
    "utf8"
  );
  return Object.values(quiz).reduce(
    (n, sections) => n + sections.reduce((m, s) => m + s.items.length, 0),
    0
  );
}

/**
 * Toạ độ vùng theo tỉ lệ 0–1 của ảnh trang.
 *
 * Kiểm ngay lúc build vì soạn tay bằng mắt rất dễ gõ nhầm số: vùng tràn ra
 * ngoài ảnh hay rộng/cao âm thì trên máy chỉ hiện ra một ô vô hình đặt sai
 * chỗ, không có lỗi nào cả — rất khó lần ra.
 */
function validateRect(at: string, rect: unknown): void {
  if (!Array.isArray(rect) || rect.length !== 4) {
    throw new Error(`${at}: "rect" phải là [x, y, rộng, cao]`);
  }
  if (rect.some((n) => typeof n !== "number" || !Number.isFinite(n))) {
    throw new Error(`${at}: "rect" chứa giá trị không phải số`);
  }
  const [x, y, w, h] = rect as number[];
  if (w <= 0 || h <= 0) {
    throw new Error(`${at}: "rect" có rộng/cao <= 0 (${w}, ${h})`);
  }
  if (x < 0 || y < 0 || x + w > 1.0001 || y + h > 1.0001) {
    throw new Error(
      `${at}: "rect" nằm ngoài ảnh — x+rộng=${(x + w).toFixed(3)}, ` +
        `y+cao=${(y + h).toFixed(3)} (phải <= 1)`
    );
  }
}

function validateTranslations(source: string, data: unknown): TranslationRegion[] {
  if (!Array.isArray(data)) {
    throw new Error(`${source}: phải là 1 mảng vùng dịch`);
  }

  const seen = new Set<string>();
  for (const region of data as TranslationRegion[]) {
    const at = `${source} (id=${region?.id ?? "?"})`;
    if (!region?.id) throw new Error(`${at}: thiếu "id"`);
    if (seen.has(region.id)) throw new Error(`${at}: trùng "id" trong cùng trang`);
    seen.add(region.id);
    if (!region.vi?.trim()) throw new Error(`${at}: thiếu bản dịch "vi"`);
    validateRect(at, region.rect);
  }

  return data as TranslationRegion[];
}

async function buildTranslations(bookId: string): Promise<number> {
  const dir = path.join(TRANSLATE_ROOT, bookId);
  const pages: Record<string, TranslationRegion[]> = {};

  for (const file of await listFiles(dir, ".json")) {
    const raw = await readFile(path.join(dir, file), "utf8");
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch (err) {
      throw new Error(`${bookId}/${file}: JSON hỏng — ${(err as Error).message}`);
    }
    pages[pageKey(file)] = validateTranslations(`${bookId}/${file}`, parsed);
  }

  await writeFile(
    path.join(TRANSLATE_ROOT, `${bookId}.json`),
    JSON.stringify(pages, null, 2) + "\n",
    "utf8"
  );
  return Object.values(pages).reduce((n, regions) => n + regions.length, 0);
}

/**
 * Điểm ngữ pháp gắn vào trang.
 *
 * `slug` phải hợp lệ vì nó là mã của CHÍNH điểm ngữ pháp, không gắn với
 * trang — sau này trang tra cứu sẽ gom theo mã này, gõ nhầm là gom sai.
 */
function validateGrammar(source: string, data: unknown): GrammarPoint[] {
  if (!Array.isArray(data)) {
    throw new Error(`${source}: phải là 1 mảng điểm ngữ pháp`);
  }

  const seen = new Set<string>();
  for (const point of data as GrammarPoint[]) {
    const at = `${source} (id=${point?.id ?? "?"})`;
    if (!point?.id) throw new Error(`${at}: thiếu "id"`);
    if (seen.has(point.id)) throw new Error(`${at}: trùng "id" trong cùng trang`);
    seen.add(point.id);

    if (!point.slug?.trim()) throw new Error(`${at}: thiếu "slug"`);
    if (!/^[a-z0-9-]+$/.test(point.slug)) {
      throw new Error(
        `${at}: "slug" chỉ được gồm chữ thường, số và dấu gạch ngang ` +
          `(đang là "${point.slug}")`
      );
    }
    if (!point.title?.trim()) throw new Error(`${at}: thiếu "title"`);
    if (!point.vi?.trim()) throw new Error(`${at}: thiếu định nghĩa "vi"`);

    // Định nghĩa là TOÀN BỘ thứ người dùng thấy, và nó nằm trong một bong
    // bóng nhỏ trên trang sách. Dài quá là che mất chính trang đang học —
    // đúng cái đã phải sửa ở bản đầu. Chặn ngay tại đây cho khỏi trôi dần.
    if (point.vi.length > MAX_GRAMMAR_VI) {
      throw new Error(
        `${at}: định nghĩa "vi" dài ${point.vi.length} ký tự, tối đa ` +
          `${MAX_GRAMMAR_VI}. Phần giải thích sâu thuộc về tab bài giảng.`
      );
    }

    if (!point.exKo?.trim()) throw new Error(`${at}: thiếu câu ví dụ "exKo"`);
    if (!point.exVi?.trim()) throw new Error(`${at}: thiếu bản dịch ví dụ "exVi"`);

    // MỘT câu, không phải một đoạn. Cả bong bóng phải đọc lướt được trong
    // vài giây, nếu không thì lại thành cái tấm phủ dài dòng đã phải bỏ.
    for (const [field, text] of [
      ["exKo", point.exKo],
      ["exVi", point.exVi],
    ] as const) {
      if (text.length > MAX_GRAMMAR_EXAMPLE) {
        throw new Error(
          `${at}: ví dụ "${field}" dài ${text.length} ký tự, tối đa ` +
            `${MAX_GRAMMAR_EXAMPLE}. Lấy một câu thôi, đừng lấy cả hộp 예문.`
        );
      }
    }

    validateRect(at, point.rect);
  }

  return data as GrammarPoint[];
}

async function buildGrammar(bookId: string): Promise<number> {
  const dir = path.join(GRAMMAR_ROOT, bookId);
  const pages: Record<string, GrammarPoint[]> = {};

  for (const file of await listFiles(dir, ".json")) {
    const raw = await readFile(path.join(dir, file), "utf8");
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch (err) {
      throw new Error(`${bookId}/${file}: JSON hỏng — ${(err as Error).message}`);
    }
    pages[pageKey(file)] = validateGrammar(`${bookId}/${file}`, parsed);
  }

  await writeFile(
    path.join(GRAMMAR_ROOT, `${bookId}.json`),
    JSON.stringify(pages, null, 2) + "\n",
    "utf8"
  );
  return Object.values(pages).reduce((n, points) => n + points.length, 0);
}

/**
 * Đáp án sách gắn vào mục bài tập.
 *
 * `source` phải là một trang có thật trong sách: nó là thứ người dùng dựa vào
 * để tự lật ra đối chiếu, gõ nhầm là chỉ sai chỗ.
 */
function validateAnswers(
  source: string,
  data: unknown,
  totalPages: number
): AnswerKey[] {
  if (!Array.isArray(data)) {
    throw new Error(`${source}: phải là 1 mảng mục đáp án`);
  }

  const seen = new Set<string>();
  for (const key of data as AnswerKey[]) {
    const at = `${source} (id=${key?.id ?? "?"})`;
    if (!key?.id) throw new Error(`${at}: thiếu "id"`);
    if (seen.has(key.id)) throw new Error(`${at}: trùng "id" trong cùng trang`);
    seen.add(key.id);

    if (!key.section?.trim()) throw new Error(`${at}: thiếu tên mục "section"`);
    if (!Number.isInteger(key.source) || key.source < 1 || key.source > totalPages) {
      throw new Error(
        `${at}: "source" phải là số trang bảng đáp án, 1–${totalPages} ` +
          `(đang là ${JSON.stringify(key.source)})`
      );
    }
    if (!Array.isArray(key.answers) || key.answers.length === 0) {
      throw new Error(`${at}: "answers" phải là mảng có ít nhất 1 dòng`);
    }
    for (const [i, line] of key.answers.entries()) {
      if (!line?.label?.trim() || !line?.text?.trim()) {
        throw new Error(`${at}: dòng đáp án thứ ${i + 1} thiếu "label" hoặc "text"`);
      }
    }

    validateRect(at, key.rect);
  }

  return data as AnswerKey[];
}

async function buildAnswers(bookId: string, totalPages: number): Promise<number> {
  const dir = path.join(ANSWERS_ROOT, bookId);
  const pages: Record<string, AnswerKey[]> = {};

  for (const file of await listFiles(dir, ".json")) {
    const raw = await readFile(path.join(dir, file), "utf8");
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch (err) {
      throw new Error(`${bookId}/${file}: JSON hỏng — ${(err as Error).message}`);
    }
    pages[pageKey(file)] = validateAnswers(`${bookId}/${file}`, parsed, totalPages);
  }

  await writeFile(
    path.join(ANSWERS_ROOT, `${bookId}.json`),
    JSON.stringify(pages, null, 2) + "\n",
    "utf8"
  );
  return Object.values(pages).reduce((n, keys) => n + keys.length, 0);
}

async function main() {
  for (const book of BOOKS) {
    const notes = await buildNotes(book.id);
    const questions = await buildQuiz(book.id);
    const regions = await buildTranslations(book.id);
    const grammar = await buildGrammar(book.id);
    const answers = await buildAnswers(book.id, book.totalPages);
    console.log(
      `  ${book.id}: ${notes} note, ${questions} câu hỏi, ` +
        `${regions} vùng dịch, ${grammar} điểm ngữ pháp, ${answers} mục đáp án`
    );
  }
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
