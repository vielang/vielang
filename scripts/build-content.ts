#!/usr/bin/env tsx
/**
 * Gộp nội dung biên soạn thủ công thành 1 file JSON/sách để `src/lib` import
 * tĩnh được:
 *
 *   content/notes/<bookId>/<page>.md   -> content/notes/<bookId>.json  (HTML)
 *   content/quiz/<bookId>/<page>.json  -> content/quiz/<bookId>.json
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

const CONTENT_ROOT = path.resolve(process.cwd(), "content");
const NOTES_ROOT = path.join(CONTENT_ROOT, "notes");
const QUIZ_ROOT = path.join(CONTENT_ROOT, "quiz");

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

async function main() {
  for (const book of BOOKS) {
    const notes = await buildNotes(book.id);
    const questions = await buildQuiz(book.id);
    console.log(`  ${book.id}: ${notes} note, ${questions} câu hỏi`);
  }
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
