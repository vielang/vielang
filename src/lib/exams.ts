/**
 * Đề luyện TOEIC Listening & Reading trong tab "Luyện thi" (`/exam`).
 *
 * Đề do VieLang tự biên soạn theo đúng format TOEIC (không chép đề của ETS),
 * mỗi đề một file `content/exams/<id>.json` đúng kiểu `Exam`. Câu hỏi, lựa
 * chọn, đoạn văn là CHỮ (HTML đã làm sạch, xem `lib/exam-types`), kèm đáp án
 * và giải thích tiếng Việt từng câu.
 *
 * Import TĨNH từng đề (xem `EXAMS`), cùng lý do với `lib/page-answers.ts`:
 * Output File Tracing của Next không lần được file đọc qua path dựng động.
 */
import t1 from "../../content/exams/toeic-reading-01.json";
import { mediaOriginBase } from "@/lib/books";
import {
  examAssetBase,
  isImage,
  qKey,
  type Content,
  type Exam,
  type ExamGroup,
  type ExamQuestion,
  type ExamSection,
  type SectionId,
} from "@/lib/exam-types";

export * from "@/lib/exam-types";

// JSON được suy kiểu rộng (string thay vì union) — ép về `Exam`; phần soát dữ
// liệu trong exams.test lo việc file đúng kiểu thật.
const EXAMS = [t1] as unknown as Exam[];

export function listExams(): Exam[] {
  return [...EXAMS].sort((a, b) => a.round - b.round);
}

export function getExam(id: string): Exam | undefined {
  return EXAMS.find((e) => e.id === id);
}

export function examTitle(exam: Exam): string {
  return exam.title ?? `${exam.level} · Đề ${exam.round}`;
}

export function sectionVi(id: SectionId): string {
  return id === "listening" ? "Nghe" : "Đọc";
}

export function totalMinutes(exam: Exam): number {
  return exam.sections.reduce((n, s) => n + s.minutes, 0);
}

export function sectionCount(s: ExamSection): number {
  return s.questions.length;
}

/** Chữ của lựa chọn: 1 → "A" (ô tròn trên phiếu trả lời). */
export function optionLetter(choice: number): string {
  return "ABCD"[choice - 1];
}

/** Nhãn lựa chọn như đề in: 1 → "(A)". */
export function optionLabel(choice: number): string {
  return `(${optionLetter(choice)})`;
}

/** Khối chứa câu `no`. */
export function groupOf(section: ExamSection, no: number): ExamGroup | undefined {
  return section.groups.find((g) => no >= g.from && no <= g.to);
}

/**
 * Các khối để LUYỆN TẬP (mỗi màn một khối, một nút kiểm tra). Khối có văn
 * bản chung (Part 6–7) giữ nguyên — các câu cùng đọc một bài. Khối đọc
 * KHÔNG có văn bản (Part 5: 30 câu độc lập) thì tách mỗi câu một màn: kiểm
 * tra 30 câu một lượt thì lời giải thích dồn thành một cột dài, và "luyện
 * từng câu" hoá ra luyện từng lô. Phần nghe giữ khối như đề — audio của khối
 * đã gồm lời chỉ dẫn và các câu nối liền (xem `groupAudio`).
 */
export function practiceBlocks(section: ExamSection): ExamGroup[] {
  if (section.audio) return section.groups;
  return section.groups.flatMap((g) =>
    g.passage
      ? [g]
      : section.questions
          .filter((q) => q.no >= g.from && q.no <= g.to)
          .map((q) => ({ from: q.no, to: q.no, instruction: g.instruction }))
  );
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
 * Audio của CẢ KHỐI, giữ nguyên như đề thật: lời chỉ dẫn rồi từng câu kèm
 * KHOẢNG DỪNG trả lời sau câu — nghe xong có thời gian chọn đáp án, thời
 * lượng khớp với đề. Các đoạn nối liền nhau nên thường gộp thành một đoạn
 * duy nhất; hội thoại dùng chung chỉ phát một lần. Rỗng khi đề chưa có mốc
 * thời gian.
 *
 * Dừng sớm `BLOCK_END_MARGIN` giây trước khi hết khoảng dừng của câu cuối:
 * mốc "hết khoảng dừng" trùng đúng lúc khối sau bắt đầu, dừng sát mốc là
 * nghe lọt nửa giây tiếng của khối sau.
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
 * Đề câu hỏi để HIỂN THỊ; `null` khi không có gì để hiện — câu Part 6 (chỗ
 * trống nằm trong văn bản của khối) và câu nghe chỉ có lựa chọn.
 */
export function questionPrompt(c: Content): Content | null {
  if (isImage(c)) return c;
  return c.html.trim() ? c : null;
}

/**
 * Tách lời chỉ dẫn "Part 6 — Questions 131-134 refer to the following
 * e-mail." thành nhãn phần ("Part 6"), khoảng câu ("131–134") và phần chữ
 * còn lại, để phần và khoảng câu hiện thành nhãn nhỏ. Câu "Questions a-b
 * refer to…" vẫn giữ trong chữ — đó là lời chỉ dẫn in trên đề thật.
 */
export function splitInstruction(text: string): { part: string | null; range: string | null; text: string } {
  const head = /^\s*(Part\s+\d+)\s*[—–:-]?\s*/i.exec(text);
  const rest = head ? text.slice(head[0].length) : text.trim();
  const r = /Questions\s+(\d+)\s*[-–—~]\s*(\d+)/i.exec(rest);
  return { part: head ? head[1] : null, range: r ? `${r[1]}–${r[2]}` : null, text: rest };
}

// ---------------------------------------------------------------------------
// Chấm điểm

/** Đáp án đã chọn, theo `qKey` ("reading:112" → 3). */
export type Answers = Record<string, number>;

/** Điểm quy đổi tối đa của một phần TOEIC (nghe hoặc đọc: 5–495). */
export const SECTION_MAX = 495;

/**
 * Bảng quy đổi số câu đúng (trên 100) → điểm của phần, ƯỚC TÍNH.
 *
 * ETS không công bố bảng quy đổi: mỗi đề thật được "equate" riêng theo độ
 * khó, cùng số câu đúng ở hai đề có thể lệch vài chục điểm. Bảng dưới lấy
 * điểm giữa của bảng tham khảo hay được trích dẫn (theo từng nhóm 5 câu,
 * vd đọc 71–75 câu đúng ≈ 310–390) đặt ở giữa nhóm, rồi nội suy tuyến tính
 * — chỉ để người học biết mình đang ở khoảng nào, không phải điểm chính thức.
 * Phần nghe cho điểm nhỉnh hơn phần đọc ở cùng số câu đúng, như bảng gốc.
 */
const CONVERSION: Record<SectionId, readonly (readonly [raw: number, scaled: number])[]> = {
  listening: [
    [0, 5], [3, 25], [8, 35], [13, 40], [18, 60], [23, 90], [28, 115], [33, 140], [38, 170], [43, 195],
    [48, 225], [53, 250], [58, 275], [63, 300], [68, 325], [73, 355], [78, 385], [83, 410], [88, 440],
    [93, 465], [98, 485], [100, 495],
  ],
  reading: [
    [0, 5], [3, 15], [8, 25], [13, 45], [18, 60], [23, 80], [28, 100], [33, 120], [38, 150], [43, 180],
    [48, 205], [53, 235], [58, 265], [63, 295], [68, 325], [73, 350], [78, 375], [83, 405], [88, 430],
    [93, 460], [98, 480], [100, 495],
  ],
};

/**
 * Điểm quy đổi ước tính của một phần. Phần ít hơn 100 câu (đề rút gọn) thì
 * quy số câu đúng về thang 100 trước khi tra bảng. Điểm TOEIC luôn là bội
 * của 5.
 */
export function scaledScore(section: SectionId, correct: number, total: number): number {
  const raw = total > 0 ? (Math.max(0, Math.min(correct, total)) / total) * 100 : 0;
  const table = CONVERSION[section];
  const i = table.findIndex(([r]) => r >= raw);
  const [r1, s1] = table[i];
  const [r0, s0] = table[Math.max(0, i - 1)];
  const scaled = r1 === r0 ? s1 : s0 + ((raw - r0) / (r1 - r0)) * (s1 - s0);
  return Math.max(5, Math.min(SECTION_MAX, Math.round(scaled / 5) * 5));
}

/**
 * Mốc mục tiêu hay gặp của người học TOEIC, theo TỔNG điểm 10–990. TOEIC
 * không có "đậu / trượt" hay cấp như các kỳ thi khác — người học đặt mục tiêu
 * theo yêu cầu đầu ra, tuyển dụng: 450, 600, 750, 900.
 */
const TOTAL_MILESTONES = [450, 600, 750, 900] as const;

export interface Milestone {
  /** Mốc tổng điểm 990 mà mốc này tương đương. */
  total: number;
  /** Mốc quy về thang của đề này (đề chỉ có phần đọc: một nửa). */
  score: number;
}

/**
 * Các mốc mục tiêu theo thang điểm của đề: đủ hai phần thì 450/600/750/900;
 * đề chỉ có một phần (vd chỉ Reading) thì lấy một nửa — 225/300/375/450,
 * "tương đương tổng ~450/600/750/900".
 */
export function milestones(exam: Exam): Milestone[] {
  // Thang của đề / thang 990 = số phần / 2 (mỗi phần tối đa 495).
  const ratio = exam.sections.length / 2;
  return TOTAL_MILESTONES.map((total) => ({ total, score: Math.round((total * ratio) / 5) * 5 }));
}

/** Mốc cao nhất đã chạm; `null` khi chưa tới mốc thấp nhất. */
export function reachedMilestone(exam: Exam, score: number): Milestone | null {
  return milestones(exam).findLast((m) => score >= m.score) ?? null;
}

/** Mốc kế tiếp và số điểm còn thiếu; `null` khi đã vượt mốc cao nhất. */
export function nextMilestone(exam: Exam, score: number): { milestone: Milestone; need: number } | null {
  const m = milestones(exam).find((x) => score < x.score);
  return m ? { milestone: m, need: m.score - score } : null;
}

/** Đề có đủ hai phần thì điểm là tổng thật; thiếu phần thì mốc chỉ "tương đương". */
export function isFullTest(exam: Exam): boolean {
  return exam.sections.length === 2;
}

/** Tên mốc để hiện: "600" với đề đủ hai phần, "~600" (tương đương tổng) với đề thiếu phần. */
export function milestoneLabel(exam: Exam, m: Milestone): string {
  return isFullTest(exam) ? `${m.total}` : `~${m.total}`;
}

export interface SectionScore {
  id: SectionId;
  /** Điểm quy đổi ước tính 5–495. */
  score: number;
  max: number;
  /** Số câu đúng (điểm thô). */
  correct: number;
  total: number;
}

export interface ExamScore {
  sections: SectionScore[];
  /** Tổng điểm quy đổi các phần có trong đề. */
  score: number;
  max: number;
  milestone: Milestone | null;
}

/** Tiến độ luyện từng câu của một đề: số câu đã bấm "Kiểm tra" trên tổng số câu. */
export function practiceProgress(
  exam: Exam,
  practice: { checked: string[] } | undefined
): { done: number; total: number } {
  const checked = new Set(practice?.checked ?? []);
  let done = 0;
  let total = 0;
  for (const s of exam.sections) {
    for (const q of s.questions) if (checked.has(qKey(s.id, q.no))) done++;
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
  practice: { checked: string[] } | undefined
): { section: SectionId; no: number } {
  const checked = new Set(practice?.checked ?? []);
  for (const s of exam.sections) {
    const q = s.questions.find((x) => !checked.has(qKey(s.id, x.no)));
    if (q) return { section: s.id, no: q.no };
  }
  const first = exam.sections[0];
  return { section: first.id, no: first.questions[0]?.no ?? 1 };
}

export function scoreExam(exam: Exam, answers: Answers): ExamScore {
  const sections = exam.sections.map((s) => {
    const correct = s.questions.filter((q) => answers[qKey(s.id, q.no)] === q.answer).length;
    const total = sectionCount(s);
    return { id: s.id, score: scaledScore(s.id, correct, total), max: SECTION_MAX, correct, total };
  });
  const score = sections.reduce((n, s) => n + s.score, 0);
  return {
    sections,
    score,
    max: sections.reduce((n, s) => n + s.max, 0),
    milestone: reachedMilestone(exam, score),
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
