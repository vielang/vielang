/**
 * Đề thi TOPIK để luyện trong tab "Luyện thi" (`/exam`).
 *
 * Nguồn: dịch vụ "토픽 기출문제 풀어보기" của topik.go.kr — câu hỏi, lựa chọn,
 * đoạn văn là CHỮ (đã làm sạch HTML, xem `lib/exam-html`), tranh/biểu đồ là
 * ảnh riêng, kèm đáp án và điểm từng câu. `scripts/import-topik.ts` chuyển bộ
 * dữ liệu gốc thành `content/exams/<id>.json` theo đúng kiểu dưới đây.
 *
 * Chữ thật thay vì ảnh cắt từ PDF (bản đầu): nét ở mọi cỡ chữ, hợp chế độ
 * tối, bấm thẳng vào lựa chọn được, và sau này tra cứu/dịch được.
 *
 * Import TĨNH từng đề (xem `EXAMS`), cùng lý do với `lib/page-answers.ts`:
 * Output File Tracing của Next không lần được file đọc qua path dựng động.
 */
import { mediaOriginBase } from "@/lib/books";
import {
  examAssetBase,
  isImage,
  qKey,
  type Content,
  type Exam,
  type ExamGroup,
  type ExamLevel,
  type ExamQuestion,
  type ExamSection,
  type SectionId,
  type WritingTask,
} from "@/lib/exam-types";

export * from "@/lib/exam-types";

// Đề TOPIK đã gỡ khỏi VieLang; đề TOEIC sẽ thêm ở đây.
const EXAMS: Exam[] = [];

export function listExams(): Exam[] {
  return [...EXAMS].sort((a, b) => b.round - a.round || a.level.localeCompare(b.level));
}

export function getExam(id: string): Exam | undefined {
  return EXAMS.find((e) => e.id === id);
}

export function examTitle(exam: Exam): string {
  return `${exam.level} · Kỳ ${exam.round}`;
}

export function sectionVi(id: SectionId): string {
  return id === "listening" ? "Nghe" : id === "reading" ? "Đọc" : "Viết";
}

export function totalMinutes(exam: Exam): number {
  return exam.sections.reduce((n, s) => n + s.minutes, 0);
}

/** Điểm tối đa của một phần (câu trắc nghiệm + câu viết). */
export function sectionMax(s: ExamSection): number {
  return (
    s.questions.reduce((m, q) => m + q.points, 0) + (s.writing?.tasks.reduce((m, t) => m + t.points, 0) ?? 0)
  );
}

/** Số câu của một phần, kể cả câu viết. */
export function sectionCount(s: ExamSection): number {
  return s.questions.length + (s.writing?.tasks.length ?? 0);
}

export function maxScore(exam: Exam): number {
  return exam.sections.reduce((n, s) => n + sectionMax(s), 0);
}

/**
 * Số ký tự của bài viết như cách đếm ô 원고지: tính cả dấu cách, không tính
 * xuống dòng.
 */
export function countChars(text: string): number {
  return text.replace(/\r?\n/g, "").length;
}

/**
 * Các khối của phần viết: câu điền chỗ trống liền nhau gộp thành một khối
 * (51–52, cùng lời chỉ dẫn "[51~52]"), mỗi bài viết (53, 54) một khối.
 */
export function writingBlocks(tasks: WritingTask[]): WritingTask[][] {
  const out: WritingTask[][] = [];
  for (const t of tasks) {
    const last = out.at(-1);
    if (last && t.kind === "blanks" && last[0].kind === "blanks") last.push(t);
    else out.push([t]);
  }
  return out;
}

/** Khối "※ [a~b]" chứa câu `no`. */
export function groupOf(section: ExamSection, no: number): ExamGroup | undefined {
  return section.groups.find((g) => no >= g.from && no <= g.to);
}

/**
 * Các đoạn cần phát để NGHE LẠI RIÊNG một câu: hội thoại chung của khối (nếu
 * có và đoạn của câu không chứa sẵn nó) rồi đến đoạn của câu. Rỗng khi đề
 * chưa có mốc thời gian từng câu — lúc đó giao diện cho nghe cả bài.
 */
export function questionAudio(section: ExamSection, q: ExamQuestion): [number, number][] {
  const own = q.replay ?? q.audio;
  if (!own) return [];
  const d = groupOf(section, q.no)?.dialogue;
  if (d && own[0] >= d[1]) return [d, own];
  return [own];
}

/**
 * Audio của CẢ KHỐI, giữ nguyên như đề thật: lời chỉ dẫn (+ câu mẫu) rồi
 * từng câu kèm KHOẢNG DỪNG trả lời sau câu — nghe xong có thời gian chọn đáp
 * án, thời lượng khớp với đề. Các đoạn nối liền nhau nên thường gộp thành
 * một đoạn duy nhất; hội thoại dùng chung chỉ phát một lần. Rỗng khi đề chưa
 * có mốc thời gian.
 *
 * Dừng sớm `BLOCK_END_MARGIN` giây trước khi hết khoảng dừng của câu cuối:
 * mốc "hết khoảng dừng" trùng đúng lúc khối sau bắt đầu (tiếng chuông), dừng
 * sát mốc là nghe lọt nửa giây tiếng của khối sau.
 */
const BLOCK_END_MARGIN = 0.7;

export function groupAudio(section: ExamSection, group: ExamGroup): [number, number][] {
  const items = section.questions.filter((q) => q.no >= group.from && q.no <= group.to);
  const parts = [...(group.audio ? [group.audio] : []), ...items.map((q) => q.audio)].filter(
    (p): p is [number, number] => !!p
  );
  if (parts.length < items.length) return [];
  const merged: [number, number][] = [];
  for (const [start, end] of parts) {
    const last = merged.at(-1);
    if (last && start - last[1] <= 0.3) last[1] = Math.max(last[1], end);
    else merged.push([start, end]);
  }
  const last = merged.at(-1);
  if (last && last[1] - last[0] > 2 * BLOCK_END_MARGIN) last[1] = Math.round((last[1] - BLOCK_END_MARGIN) * 10) / 10;
  return merged;
}

/**
 * Đề câu hỏi chỉ là "(4점)" thì không cần hiện: số điểm đã có ở tiêu đề câu,
 * còn câu nghe thì nội dung nằm trong file nghe.
 */
export function isPointsOnly(c: Content): boolean {
  return questionPrompt(c) === null;
}

/** "(3점)" ở đầu đề câu hỏi, kèm các dòng trống theo sau. */
const POINTS_HEAD = /^\s*\(\d+점\)\s*(?:<br\s*\/?>\s*)*/;
/** "(3점)" đứng riêng thành dòng cuối. */
const POINTS_LAST_LINE = /(?:<br\s*\/?>\s*)+\(\d+점\)\s*$/;

/**
 * Đề câu hỏi để HIỂN THỊ: bỏ "(N점)" ở đầu hoặc đứng riêng ở dòng cuối — số
 * điểm đã có ở tiêu đề câu, lặp lại chỉ thêm nhiễu. "…고르십시오. (3점)" thì
 * giữ nguyên vì nó nằm trong câu hỏi in. `null` khi không còn gì để hiện.
 */
export function questionPrompt(c: Content): Content | null {
  if (isImage(c)) return c;
  const html = c.html.replace(POINTS_HEAD, "").replace(POINTS_LAST_LINE, "");
  if (!html.trim()) return null;
  return html === c.html ? c : { html };
}

/**
 * Tách lời chỉ dẫn "※ [1～4] 다음을 듣고 …" thành khoảng câu ("1–4") và
 * phần chữ, để khoảng câu hiện thành nhãn nhỏ thay vì chữ đậm dài.
 */
export function splitInstruction(text: string): { range: string | null; text: string } {
  const m = /^※?\s*\[([^\]]+)\]\s*/.exec(text);
  if (!m) return { range: null, text: text.replace(/^※\s*/, "") };
  return { range: m[1].replace(/\s*[～~]\s*/, "–").trim(), text: text.slice(m[0].length) };
}

// ---------------------------------------------------------------------------
// Chấm điểm

/** Đáp án trắc nghiệm đã chọn, theo `qKey` ("reading:12" → 3). */
export type Answers = Record<string, number>;
/** Chữ đã viết ở câu viết, theo `textKey` ("51:0" → "…"). */
export type Texts = Record<string, string>;
/** Điểm TỰ CHẤM từng câu viết, theo số câu (53 → 24). */
export type Grades = Record<number, number>;

export interface SectionScore {
  id: SectionId;
  score: number;
  max: number;
  /** Trắc nghiệm: số câu đúng. Phần viết: số câu đã tự chấm. */
  correct: number;
  total: number;
}

export interface ExamScore {
  sections: SectionScore[];
  score: number;
  max: number;
  level: string | null;
  /** Còn câu viết chưa tự chấm — điểm và cấp hiện tại chưa phải cuối cùng. */
  ungraded: number;
}

/**
 * Ngưỡng cấp theo quy định TOPIK (tổng điểm các phần):
 * TOPIK I (200 điểm): 1급 ≥ 80, 2급 ≥ 140.
 * TOPIK II (300 điểm): 3급 ≥ 120, 4급 ≥ 150, 5급 ≥ 190, 6급 ≥ 230.
 */
const LEVEL_CUTS: Record<ExamLevel, [number, string][]> = {
  "TOPIK I": [
    [140, "Cấp 2"],
    [80, "Cấp 1"],
  ],
  "TOPIK II": [
    [230, "Cấp 6"],
    [190, "Cấp 5"],
    [150, "Cấp 4"],
    [120, "Cấp 3"],
  ],
};

export function levelFor(examLevel: ExamLevel, score: number): string | null {
  return LEVEL_CUTS[examLevel].find(([cut]) => score >= cut)?.[1] ?? null;
}

/** Các ngưỡng cấp, từ thấp lên cao: [[80, "Cấp 1"], [140, "Cấp 2"]]. */
export function levelCuts(examLevel: ExamLevel): [number, string][] {
  return [...LEVEL_CUTS[examLevel]].reverse();
}

/** Cấp kế tiếp và số điểm còn thiếu; `null` khi đã đạt cấp cao nhất. */
export function nextLevel(examLevel: ExamLevel, score: number): { level: string; need: number } | null {
  const next = levelCuts(examLevel).find(([cut]) => score < cut);
  return next ? { level: next[1], need: next[0] - score } : null;
}

/**
 * Tiến độ luyện từng câu của một đề: câu trắc nghiệm đã bấm "Kiểm tra" và
 * câu viết đã tự chấm, trên tổng số câu.
 */
export function practiceProgress(
  exam: Exam,
  practice: { checked: string[]; grades?: Grades } | undefined
): { done: number; total: number } {
  const checked = new Set(practice?.checked ?? []);
  let done = 0;
  let total = 0;
  for (const s of exam.sections) {
    for (const q of s.questions) if (checked.has(qKey(s.id, q.no))) done++;
    for (const t of s.writing?.tasks ?? []) if (practice?.grades?.[t.no] !== undefined) done++;
    total += sectionCount(s);
  }
  return { done, total };
}

/**
 * Chỗ để "Luyện tiếp": câu đầu tiên chưa làm, theo thứ tự các phần. Làm hết
 * rồi thì về câu đầu.
 */
export function nextPracticeTarget(
  exam: Exam,
  practice: { checked: string[]; grades?: Grades } | undefined
): { section: SectionId; no: number } {
  const checked = new Set(practice?.checked ?? []);
  for (const s of exam.sections) {
    const q = s.questions.find((x) => !checked.has(qKey(s.id, x.no)));
    if (q) return { section: s.id, no: q.no };
    const t = s.writing?.tasks.find((x) => practice?.grades?.[x.no] === undefined);
    if (t) return { section: s.id, no: t.no };
  }
  const first = exam.sections[0];
  return { section: first.id, no: first.questions[0]?.no ?? first.writing?.tasks[0].no ?? 1 };
}

export function scoreExam(exam: Exam, answers: Answers, grades: Grades = {}): ExamScore {
  let ungraded = 0;
  const sections = exam.sections.map((s) => {
    let score = 0;
    let correct = 0;
    for (const q of s.questions) {
      if (answers[qKey(s.id, q.no)] === q.answer) {
        score += q.points;
        correct++;
      }
    }
    for (const t of s.writing?.tasks ?? []) {
      const g = grades[t.no];
      if (g === undefined) {
        ungraded++;
        continue;
      }
      // Điểm tự chấm lưu trên máy người dùng — kẹp lại cho chắc.
      score += Math.max(0, Math.min(t.points, g));
      correct++;
    }
    return { id: s.id, score, max: sectionMax(s), correct, total: sectionCount(s) };
  });
  const score = sections.reduce((n, s) => n + s.score, 0);
  return {
    sections,
    score,
    max: sections.reduce((n, s) => n + s.max, 0),
    level: levelFor(exam.level, score),
    ungraded,
  };
}

// ---------------------------------------------------------------------------
// Tài nguyên (ảnh, file nghe)

export function examAssetUrl(exam: Exam, path: string): string {
  return `${examAssetBase(exam.assetDir)}/${path}`;
}

/**
 * File nghe trỏ THẲNG R2 ở production — cùng lý do với bài nghe của sách
 * (xem chú thích dài ở `lib/audio.ts`): cache biên của Vercel trả sai khoảng
 * byte (`Range`) qua rewrite, audio quay mãi không phát. Lúc dev thì lấy file
 * trong `public/`, dev server trả `Range` đúng.
 */
export function examAudioUrl(exam: Exam, file: string): string {
  if (process.env.NODE_ENV !== "production") return examAssetUrl(exam, file);
  return `${mediaOriginBase()}/exams/${exam.assetDir}/${file}`;
}
