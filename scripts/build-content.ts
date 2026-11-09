#!/usr/bin/env tsx
/**
 * Dựng nội dung Markdown thành JSON để `src/lib` import tĩnh được:
 *
 *   content/it/<khoá>/<chương>/<bài>.md  -> content/it/courses.json
 *   content/cam-nang/<mục>/<bài>.md      -> content/cam-nang/guide.json
 *
 * Vì sao import tĩnh: Output File Tracing của Next không lần được file đọc
 * qua path dựng động lúc chạy, dẫn tới ENOENT khi deploy serverless dù chạy
 * đúng ở `next dev`.
 *
 * File .json sinh ra KHÔNG commit vào git (xem .gitignore) — luôn sinh lại từ
 * nguồn qua npm lifecycle hook (`predev`, `prebuild`), nên không bao giờ lệch.
 */
import { mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { Marked } from "marked";
import { GUIDE_GROUPS } from "../src/lib/guide-groups";
import { GUIDE_SECTIONS as GUIDE_SECTION_LIST } from "../src/lib/guide-sections";
import type { QuizItem, QuizSection } from "../src/lib/quiz";

const CONTENT_ROOT = path.resolve(process.cwd(), "content");
const IT_ROOT = path.join(CONTENT_ROOT, "it");

/**
 * `gfm` (bảng, ~~gạch ngang~~) bật sẵn; `breaks: false` để xuống dòng đơn
 * trong .md không thành <br> — bài được viết wrap ~76 cột cho dễ đọc ở
 * editor, không phải để ngắt dòng thật.
 */
const marked = new Marked({ gfm: true, breaks: false, async: false });

async function listFiles(dir: string, ext: string): Promise<string[]> {
  try {
    return (await readdir(dir)).filter((f) => f.endsWith(ext)).sort();
  } catch {
    return []; // mục chưa có bài
  }
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
  code?: string;
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
    const base = {
      id,
      prompt: q.prompt ?? "",
      ...(q.code ? { code: q.code.trimEnd() } : {}),
      ...(q.explain ? { explain: q.explain } : {}),
    };

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
const CODE_LANGS = ["csharp", "bash", "sql", "json", "xml", "csv", "diff", "http"];

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
/**
 * Dòng định nghĩa theo FORMAT.md có dạng "🧬 **Thuật ngữ**: một câu". Gắn class
 * `definition` cho đoạn đó để CSS cho nó một nền nhạt, người đọc quét là thấy
 * các khái niệm chính của bài.
 */
function markDefinitions(html: string): string {
  return html.replace(
    /<p>(?=\p{Extended_Pictographic}[️‍\p{Extended_Pictographic}]*\s*<strong>)/gu,
    '<p class="definition">'
  );
}

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
          const html = highlighter.codeToHtml(text, {
            lang: loaded.has(resolved) ? resolved : "text",
            themes: CODE_THEMES,
            defaultColor: false,
          });
          // Cặp phản ví dụ "SAI / ĐÚNG" nhận ra qua comment ở dòng đầu (//, #,
          // --). Gắn data-verdict để CSS tô viền đỏ / xanh: quét qua là biết
          // khối nào là cách sai.
          const firstLine = text.trimStart().split("\n", 1)[0];
          const verdict = /^(\/\/|#|--)\s*SAI\b/.test(firstLine)
            ? "wrong"
            : /^(\/\/|#|--)\s*ĐÚNG/.test(firstLine)
              ? "right"
              : "";
          const pre = verdict ? html.replace("<pre ", `<pre data-verdict="${verdict}" `) : html;
          // Nút chép code: bài nào cũng dặn "chép vào Program.cs". Click được
          // xử lý bằng event delegation ở LessonView, ở đây chỉ cần markup.
          return (
            `<div class="code-block">${pre}` +
            `<button type="button" class="copy-code" data-copy-code aria-label="Chép code">Chép</button></div>`
          );
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
        const { html, headings } = withHeadingIds(
          markDefinitions((lessonMarked.parse(body) as string).trim())
        );
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

// ---------------------------------------------------------------------------
// Cẩm nang (xem src/lib/guide.ts)

const GUIDE_ROOT = path.join(CONTENT_ROOT, "cam-nang");

/** Thư mục con = mục của cẩm nang. Thứ tự và tên hiển thị nằm ở src/lib/guide.ts. */
// Danh sách mục lấy từ src/lib/guide-sections.ts — một nguồn duy nhất.
const GUIDE_SECTIONS = GUIDE_SECTION_LIST.map((s) => s.id);

/** Khoá ổn định cho một dòng checklist: băm nội dung, không theo thứ tự. */
function checkKey(text: string): string {
  let h = 5381;
  for (const ch of text) h = ((h << 5) + h + ch.charCodeAt(0)) >>> 0;
  return h.toString(36);
}

/**
 * Checklist hồ sơ (`- [ ] …`) thành ô tick được. Marked vẽ ô `disabled`;
 * ở đây bỏ `disabled` và gắn khoá để app nhớ ô nào đã tick (xem
 * GuideArticleView).
 *
 * Khoá băm theo NỘI DUNG dòng chứ không theo thứ tự: bài được sửa, chèn
 * thêm giấy tờ vào giữa thì các ô đã tick vẫn đúng chỗ. Đổi chữ của một dòng
 * thì dòng đó mất dấu tick — chấp nhận được, vì nội dung đã khác.
 */
function interactiveChecklist(html: string): { html: string; checks: number } {
  let checks = 0;
  const out = html.replace(
    /<li><input[^>]*type="checkbox"[^>]*>\s*([\s\S]*?)<\/li>/g,
    (_all, inner: string) => {
      checks++;
      const key = checkKey(decodeEntities(inner.replace(/<[^>]+>/g, "")).trim());
      return (
        `<li class="check-item"><label><input type="checkbox" data-check="${key}">` +
        `<span>${inner}</span></label></li>`
      );
    }
  );
  return { html: out, checks };
}

/** Link ra ngoài (nguồn chính thức) mở tab mới — đọc xong vẫn còn bài để đối chiếu. */
function externalLinks(html: string): string {
  return html.replace(/<a href="(https?:\/\/[^"]+)"/g, '<a href="$1" target="_blank" rel="noopener noreferrer"');
}

/**
 * Bảng rộng hơn màn điện thoại: bọc trong một khung cuộn ngang riêng, để chỉ
 * bảng cuộn chứ không kéo cả trang tràn ngang.
 */
function scrollableTables(html: string): string {
  return html.replace(/<table>([\s\S]*?)<\/table>/g, '<div class="table-scroll"><table>$1</table></div>');
}

/**
 * Mục Nguồn: ghi tên miền cạnh mỗi link (`learn.microsoft.com`,
 * `git-scm.com`…) để người đọc thấy ngay nguồn có phải trang chính thức
 * không, khỏi phải bấm vào mới biết.
 */
function sourceDomains(html: string): string {
  const at = html.search(/<h2[^>]*>Nguồn<\/h2>/);
  if (at === -1) return html;
  const tail = html
    .slice(at)
    .replace(/(<a href="https?:\/\/(?:www\.)?([^/"]+)[^"]*"[^>]*>[\s\S]*?<\/a>)/g, '$1 <span class="source-domain">$2</span>');
  return html.slice(0, at) + tail;
}

async function buildGuide(): Promise<number> {
  // Chưa có mục nào vẫn phải sinh guide.json rỗng: lib/guide.ts import tĩnh file này.
  await mkdir(GUIDE_ROOT, { recursive: true });
  const sections: Record<string, unknown[]> = {};
  let count = 0;
  for (const section of GUIDE_SECTIONS) {
    const dir = path.join(GUIDE_ROOT, section);
    const articles = [];
    for (const file of await listFiles(dir, ".md")) {
      const at = `cam-nang/${section}/${file}`;
      const { meta, body } = frontMatter(await readFile(path.join(dir, file), "utf8"));
      if (!meta.title) throw new Error(`${at}: thiếu "title"`);
      if (!meta.updated) throw new Error(`${at}: thiếu "updated" — bài cẩm nang phải ghi ngày cập nhật`);
      if (!/^## Nguồn\s*$/m.test(body)) throw new Error(`${at}: thiếu mục "## Nguồn"`);
      const groups = GUIDE_GROUPS[section];
      if (groups && !groups.includes(meta.group ?? "")) {
        // Gõ sai tên nhóm thì bài rơi vào nhóm "Khác" mà không ai hay — chặn ở đây.
        throw new Error(`${at}: "group" phải là một trong: ${groups.join(", ")}`);
      }
      const { title, summary, updated, ...facts } = meta;
      const checklist = interactiveChecklist(marked.parse(body) as string);
      const { html, headings } = withHeadingIds(
        sourceDomains(scrollableTables(externalLinks(checklist.html.trim())))
      );
      articles.push({
        slug: lessonSlug(file),
        title,
        summary: summary ?? "",
        updated,
        facts,
        html,
        headings,
        checks: checklist.checks,
      });
    }
    sections[section] = articles;
    count += articles.length;
  }
  await writeFile(path.join(GUIDE_ROOT, "guide.json"), JSON.stringify(sections, null, 2) + "\n", "utf8");
  return count;
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
  const lessons = await buildCourses();
  if (lessons) console.log(`  IT: ${lessons} bài học`);
  const guide = await buildGuide();
  if (guide) console.log(`  Cẩm nang: ${guide} bài`);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
