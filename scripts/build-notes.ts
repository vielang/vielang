#!/usr/bin/env tsx
/**
 * Gộp các file note Markdown (nguồn: content/notes/<bookId>/<page>.md) thành
 * 1 file JSON/sách (content/notes/<bookId>.json) — map "<page>" -> markdown.
 *
 * Vì sao cần bước gộp này thay vì đọc thẳng .md lúc runtime: `lib/notes.ts`
 * dùng `import` tĩnh cho 4 file JSON này (không dùng fs.readFileSync với
 * path dựng động) để né lỗi kinh điển của Next — Output File Tracing không
 * detect được file đọc qua path động ở runtime, dẫn tới ENOENT khi deploy
 * serverless dù chạy đúng ở `next dev`.
 *
 * File .json sinh ra KHÔNG commit vào git (xem .gitignore) — luôn sinh lại
 * từ nguồn .md qua npm lifecycle hook (`predev`, `prebuild`), nên không bao
 * giờ bị lệch dữ liệu.
 */
import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { BOOKS } from "../src/lib/books";

const NOTES_ROOT = path.resolve(process.cwd(), "content", "notes");

async function buildBook(bookId: string): Promise<number> {
  const dir = path.join(NOTES_ROOT, bookId);
  let files: string[];
  try {
    files = await readdir(dir);
  } catch {
    files = []; // sách chưa có note nào — vẫn sinh ra {} để import không lỗi
  }

  const notes: Record<string, string> = {};
  for (const file of files) {
    if (!file.endsWith(".md")) continue;
    const page = String(Number(file.replace(/\.md$/, ""))); // "0010" -> "10"
    notes[page] = (await readFile(path.join(dir, file), "utf8")).trim();
  }

  const outPath = path.join(NOTES_ROOT, `${bookId}.json`);
  await writeFile(outPath, JSON.stringify(notes, null, 2) + "\n", "utf8");
  return Object.keys(notes).length;
}

async function main() {
  for (const book of BOOKS) {
    const count = await buildBook(book.id);
    console.log(`  ${book.id}: ${count} note`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
