#!/usr/bin/env tsx
/**
 * Gộp nội dung biên soạn thủ công thành 1 file JSON/sách để `src/lib` import
 * tĩnh được:
 *
 *   content/notes/<bookId>/<page>.md   -> content/notes/<bookId>.json  (HTML)
 *   content/quiz/<bookId>/<page>.json  -> content/quiz/<bookId>.json
 *   content/grammar/<bookId>/<page>.json -> content/grammar/<bookId>.json
 *   content/answers/<bookId>/<page>.json -> content/answers/<bookId>.json
 *   content/it/<khoá>/<chương>/<bài>.md  -> content/it/courses.json
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
import { readdir, readFile, stat, writeFile } from "node:fs/promises";
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
const IT_ROOT = path.join(CONTENT_ROOT, "it");

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
    if (region.dot !== undefined) {
      const d = region.dot as unknown;
      if (
        !Array.isArray(d) ||
        d.length !== 2 ||
        d.some((n) => typeof n !== "number" || !Number.isFinite(n) || n < 0 || n > 1)
      ) {
        throw new Error(`${at}: "dot" phải là [x, y] trong khoảng 0–1`);
      }
    }
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

// ---------------------------------------------------------------------------
// Khoá học IT (mảng "IT" của thư viện — xem src/lib/courses.ts)

/**
 * Phần khai báo ở đầu file Markdown, giữa hai dòng "---". Tự đọc thay vì
 * thêm thư viện: chỉ cần `khoá: giá trị` một dòng, giá trị là chữ hoặc số.
 */
function frontMatter(raw: string): { meta: Record<string, string>; body: string } {
  const m = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/.exec(raw);
  if (!m) return { meta: {}, body: raw };
  const meta: Record<string, string> = {};
  for (const line of m[1].split(/\r?\n/)) {
    const at = line.indexOf(":");
    if (at > 0) meta[line.slice(0, at).trim()] = line.slice(at + 1).trim();
  }
  return { meta, body: raw.slice(m[0].length) };
}

/** "01-oop" -> "oop" (số đầu chỉ để sắp thứ tự trong thư mục). */
const unprefix = (name: string) => name.replace(/^\d+[-_]/, "");

/** Bỏ đuôi .md và số thứ tự. */
const lessonSlug = (file: string) => unprefix(file.replace(/\.md$/i, ""));

/**
 * Mục lục trong bài: lấy h2/h3 của HTML vừa dựng và gắn `id` cho chúng để
 * mục lục bấm được. Làm ngay lúc build, phía app không phải đụng vào HTML.
 */
/** Năm entity mà marked sinh ra khi escape; đủ cho tiêu đề bài học. */
function decodeEntities(text: string): string {
  const map: Record<string, string> = {
    "&lt;": "<",
    "&gt;": ">",
    "&quot;": '"',
    "&#39;": "'",
    "&amp;": "&",
  };
  // &amp; xử lý sau cùng, không thì "&amp;lt;" bị giải mã hai lần.
  return text.replace(/&(lt|gt|quot|#39);/g, (m) => map[m]).replace(/&amp;/g, "&");
}

function withHeadingIds(html: string): { html: string; headings: { id: string; text: string; level: 2 | 3 }[] } {
  const headings: { id: string; text: string; level: 2 | 3 }[] = [];
  const used = new Set<string>();
  const out = html.replace(/<h([23])>([\s\S]*?)<\/h\1>/g, (_all, lv: string, inner: string) => {
    // Bỏ thẻ, rồi GIẢI MÃ entity: tiêu đề kiểu "List<T> — …" ra khỏi marked
    // dưới dạng `List&lt;T&gt;`, mà mục lục hiển thị chuỗi này như chữ thường
    // nên người đọc sẽ thấy nguyên cả `&lt;`.
    const text = decodeEntities(inner.replace(/<[^>]+>/g, "")).trim();
    const base =
      text
        .toLowerCase()
        .normalize("NFD")
        .replace(/[\u0300-\u036f]/g, "")
        .replace(/đ/g, "d")
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, "") || "muc";
    let id = base;
    for (let i = 2; used.has(id); i++) id = `${base}-${i}`;
    used.add(id);
    headings.push({ id, text, level: Number(lv) as 2 | 3 });
    return `<h${lv} id="${id}">${inner}</h${lv}>`;
  });
  return { html: out, headings };
}

/**
 * Câu tự kiểm tra cuối bài học IT, viết ngay trong file .md bằng một khối
 * ```quiz chứa JSON:
 *
 *     ```quiz
 *     [
 *       { "prompt": "…", "options": ["…", "…"], "answer": 1, "explain": "…" }
 *     ]
 *     ```
 *
 * Để CHUNG file với bài thay vì một file JSON riêng: người viết bài sửa một
 * chỗ, và câu hỏi nằm ngay cạnh đoạn nội dung mà nó hỏi.
 *
 * Dạng rút gọn (`options` -> choice, `answers` -> fill) rồi mới dựng thành
 * `QuizItem` đầy đủ — `kind` và `id` suy ra được, bắt người viết gõ tay chỉ
 * tổ sinh lỗi. Id đánh theo thứ tự (`q1`, `q2`…) và là thứ được lưu vào bài
 * làm, nên CHÈN câu vào giữa sẽ làm lệch bài làm cũ của bài học đó — thêm
 * câu mới thì thêm vào cuối.
 */
const QUIZ_FENCE = /^```quiz[ \t]*\r?\n([\s\S]*?)^```[ \t]*$/m;

interface RawQuizItem {
  prompt?: string;
  options?: string[];
  answers?: string[];
  explain?: string;
}

function buildLessonQuiz(at: string, body: string): { body: string; quiz?: QuizSection[] } {
  const m = QUIZ_FENCE.exec(body);
  if (!m) return { body };

  let raw: unknown;
  try {
    raw = JSON.parse(m[1]);
  } catch (err) {
    throw new Error(`${at}: khối "quiz" không phải JSON hợp lệ — ${(err as Error).message}`);
  }
  if (!Array.isArray(raw) || raw.length === 0) {
    throw new Error(`${at}: khối "quiz" phải là một mảng câu hỏi, không rỗng`);
  }

  const items: QuizItem[] = raw.map((entry, i) => {
    const q = entry as RawQuizItem;
    const id = `q${i + 1}`;
    const where = `${at} câu ${i + 1}`;
    const base = { id, prompt: q.prompt ?? "", ...(q.explain ? { explain: q.explain } : {}) };

    if (q.options) {
      // `answer` viết theo cách người ta đánh số câu trắc nghiệm: 1 là phương
      // án đầu. Bên trong vẫn là chỉ số từ 0 như `ChoiceItem` quy định.
      const answer = Number((entry as { answer?: number }).answer);
      if (!Number.isInteger(answer) || answer < 1 || answer > q.options.length) {
        throw new Error(
          `${where}: "answer" = ${answer} nằm ngoài "options" (1..${q.options.length})`
        );
      }
      const item: QuizItem = { ...base, kind: "choice", options: q.options, answer: answer - 1 };
      validateItem(where, item);
      return item;
    }
    if (q.answers) {
      const item: QuizItem = { ...base, kind: "fill", answers: q.answers };
      validateItem(where, item);
      return item;
    }
    throw new Error(`${where}: cần "options" (trắc nghiệm) hoặc "answers" (điền)`);
  });

  return {
    // Cắt khối quiz khỏi phần nội dung: nó được vẽ bằng component, không phải
    // bằng HTML của bài.
    body: body.replace(QUIZ_FENCE, "").trimEnd(),
    quiz: [{ title: "Tự kiểm tra", items }],
  };
}

/** Thời gian đọc ước lượng: ~180 từ/phút, code tính gấp đôi thời gian. */
function readingMinutes(body: string): number {
  const code = (body.match(/```[\s\S]*?```/g) ?? []).join(" ");
  const words = body.split(/\s+/).length + code.split(/\s+/).length;
  return Math.max(1, Math.round(words / 180));
}

/**
 * Tô màu code bằng Shiki — dùng chính grammar TextMate và theme của VS Code
 * nên C# ra đúng màu như lúc gõ trong IDE, hơn hẳn regex của highlight.js.
 *
 * Chạy LÚC BUILD: HTML sinh ra đã có màu dán thẳng vào từng token, trình
 * duyệt không phải tải thêm thư viện tô màu nào (Shiki chỉ nằm trong
 * devDependencies). Đổi lại phải build lại khi sửa bài — vốn đã phải thế.
 *
 * `defaultColor: false` cho Shiki ghi màu của CẢ HAI theme vào biến CSS
 * (`--shiki-light`, `--shiki-dark`), để CSS chọn theo class `.dark` — xem
 * globals.css. Nếu chỉ nhúng một theme thì chế độ tối phải tô màu lại bằng
 * JS, hoặc code sáng trưng giữa nền tối.
 */
const CODE_THEMES = { light: "github-light", dark: "github-dark" } as const;

/**
 * Ngôn ngữ nạp sẵn cho Shiki. Chỉ nạp những thứ khoá học thật sự dùng: mỗi
 * grammar là một file khá nặng, nạp cả bundle thì build chậm vô ích.
 */
const CODE_LANGS = ["csharp", "bash", "sql", "json", "xml", "csv", "diff"];

/** Tên ngôn ngữ hay viết tắt trong file .md -> tên Shiki hiểu. */
const LANG_ALIASES: Record<string, string> = {
  cs: "csharp",
  "c#": "csharp",
  dotnet: "bash",
  sh: "bash",
  shell: "bash",
  console: "bash",
  plsql: "sql",
  oracle: "sql",
  csproj: "xml",
  text: "text",
};

type CodeHighlighter = Awaited<ReturnType<typeof import("shiki").createHighlighter>>;
type MermaidRenderer = typeof import("beautiful-mermaid").renderMermaidSVG;

/**
 * Sơ đồ trong bài học, viết bằng cú pháp Mermaid:
 *
 *     ```mermaid Vòng đời của một truy vấn LINQ
 *     flowchart TD
 *         A[...] --> B[...]
 *     ```
 *
 * Phần chữ sau "mermaid" là CHÚ THÍCH, hiện dưới sơ đồ và cũng là phần mô tả
 * cho người dùng trình đọc màn hình.
 *
 * Vẽ LÚC BUILD thành SVG: trang không phải tải thư viện vẽ nào (mermaid thật
 * nặng 300–800 KB gzip và chỉ chạy được trong trình duyệt). Dùng
 * `beautiful-mermaid` — bản viết lại thuần TypeScript, chạy thẳng trong Node,
 * không cần Chromium, nên `next build` trên máy chủ dựng vẫn chạy bình thường.
 *
 * Màu truyền vào là BIẾN CSS chứ không phải mã màu, nên sơ đồ tự đổi theo chế
 * độ sáng/tối mà không cần vẽ lại — cùng cách đang làm cho code.
 *
 * Tên biến phải là `--diagram-*` riêng, KHÔNG dùng thẳng token của app: thư
 * viện ghi các biến này lên chính thẻ svg, nên `--border: var(--border)` sẽ
 * tự tham chiếu vòng và hỏng, còn `--surface: var(--muted)` sẽ ăn nhầm giá
 * trị `--muted` vừa bị ghi đè ngay trên thẻ đó. Ánh xạ sang token thật nằm ở
 * globals.css.
 */
const DIAGRAM_COLORS = {
  bg: "var(--diagram-bg)",
  fg: "var(--diagram-fg)",
  line: "var(--diagram-line)",
  border: "var(--diagram-line)",
  muted: "var(--diagram-muted)",
  surface: "var(--diagram-surface)",
  accent: "var(--diagram-accent)",
  transparent: true,
} as const;

const escapeHtml = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!
  );

function renderDiagram(
  render: MermaidRenderer,
  at: string,
  source: string,
  caption: string
): string {
  let svg: string;
  try {
    svg = render(source, DIAGRAM_COLORS);
  } catch (err) {
    throw new Error(`${at}: sơ đồ mermaid không vẽ được — ${(err as Error).message}`);
  }

  const title = caption || "Sơ đồ minh hoạ";
  svg = svg
    // Thư viện nhúng sẵn một @import tới Google Fonts. App đã có font riêng,
    // còn đây sẽ là một request ra ngoài trên mỗi trang có sơ đồ — bỏ.
    .replace(/@import url\([^)]*\);/g, "")
    .replace(/font-family:[^;]+;/, "font-family: inherit;")
    // Trình đọc màn hình chỉ thấy một khối đồ hoạ; cho nó cái tên.
    .replace(/^<svg /, `<svg role="img" aria-label="${escapeHtml(title)}" `);

  return (
    `<figure class="lesson-diagram">${svg}` +
    (caption ? `<figcaption>${escapeHtml(caption)}</figcaption>` : "") +
    `</figure>`
  );
}

/**
 * Renderer code của marked cho bài học IT. Ngôn ngữ lạ (hoặc khối không ghi
 * ngôn ngữ) rơi về `text`: vẫn ra đúng khung code, chỉ là không có màu —
 * tốt hơn là làm hỏng cả lần build.
 */
function lessonMarkdown(highlighter: CodeHighlighter, render: MermaidRenderer, at: string) {
  const loaded = new Set(highlighter.getLoadedLanguages());
  return new Marked(
    { gfm: true, breaks: false, async: false },
    {
      renderer: {
        code({ text, lang }) {
          const info = (lang ?? "").trim();
          // "mermaid Chú thích của sơ đồ" — chữ sau tên ngôn ngữ là chú thích.
          const space = info.indexOf(" ");
          const name = (space > 0 ? info.slice(0, space) : info).toLowerCase();
          if (name === "mermaid") {
            return renderDiagram(render, at, text, space > 0 ? info.slice(space + 1).trim() : "");
          }
          const resolved = LANG_ALIASES[name] ?? name;
          return highlighter.codeToHtml(text, {
            lang: loaded.has(resolved) ? resolved : "text",
            themes: CODE_THEMES,
            defaultColor: false,
          });
        },
      },
    }
  );
}

async function buildCourses(): Promise<number> {
  if (!(await exists(IT_ROOT))) return 0;
  // import động: chỉ khoá học mới cần Shiki, các phần nội dung khác build
  // xong từ lâu trước khi nó kịp nạp grammar.
  const { createHighlighter } = await import("shiki");
  const { renderMermaidSVG } = await import("beautiful-mermaid");
  const highlighter = await createHighlighter({
    themes: Object.values(CODE_THEMES),
    langs: CODE_LANGS,
  });
  const courses = [];
  for (const courseDir of (await readdir(IT_ROOT)).sort()) {
    const courseRoot = path.join(IT_ROOT, courseDir);
    if (!(await stat(courseRoot)).isDirectory()) continue;
    const info = frontMatter(await readFile(path.join(courseRoot, "course.md"), "utf8"));
    const modules = [];
    for (const moduleDir of (await readdir(courseRoot)).sort()) {
      const moduleRoot = path.join(courseRoot, moduleDir);
      if (!(await stat(moduleRoot)).isDirectory()) continue;
      const moduleInfo = frontMatter(await readFile(path.join(moduleRoot, "_module.md"), "utf8"));
      const lessons = [];
      for (const file of (await readdir(moduleRoot)).sort()) {
        if (!file.endsWith(".md") || file === "_module.md") continue;
        const raw = frontMatter(await readFile(path.join(moduleRoot, file), "utf8"));
        const { meta } = raw;
        const at = `${courseDir}/${moduleDir}/${file}`;
        const { body, quiz } = buildLessonQuiz(at, raw.body);
        // Marked dựng riêng cho từng bài để báo lỗi sơ đồ kèm tên file.
        const lessonMarked = lessonMarkdown(highlighter, renderMermaidSVG, at);
        const { html, headings } = withHeadingIds((lessonMarked.parse(body) as string).trim());
        lessons.push({
          slug: `${unprefix(moduleDir)}/${lessonSlug(file)}`,
          title: meta.title ?? lessonSlug(file),
          minutes: Number(meta.minutes) || readingMinutes(body),
          html,
          headings,
          ...(quiz ? { quiz } : {}),
        });
      }
      if (lessons.length === 0) continue;
      modules.push({
        slug: unprefix(moduleDir),
        title: moduleInfo.meta.title ?? unprefix(moduleDir),
        ...(moduleInfo.meta.summary ? { summary: moduleInfo.meta.summary } : {}),
        lessons,
      });
    }
    courses.push({
      id: courseDir,
      title: info.meta.title ?? courseDir,
      summary: info.meta.summary ?? "",
      level: info.meta.level ?? "Nền tảng",
      order: Number(info.meta.order) || 99,
      modules,
    });
  }
  await writeFile(path.join(IT_ROOT, "courses.json"), JSON.stringify(courses, null, 2) + "\n", "utf8");
  return courses.reduce((n, c) => n + c.modules.reduce((m, mod) => m + mod.lessons.length, 0), 0);
}

async function exists(p: string): Promise<boolean> {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
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
  const lessons = await buildCourses();
  if (lessons) console.log(`  IT: ${lessons} bài học`);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
