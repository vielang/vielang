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
import e35 from "../../content/exams/35-topik1.json";
import e35b from "../../content/exams/35-topik2.json";
import e36 from "../../content/exams/36-topik1.json";
import e36b from "../../content/exams/36-topik2.json";
import e37 from "../../content/exams/37-topik1.json";
import e37b from "../../content/exams/37-topik2.json";
import e41 from "../../content/exams/41-topik1.json";
import e41b from "../../content/exams/41-topik2.json";
import e47 from "../../content/exams/47-topik1.json";
import e47b from "../../content/exams/47-topik2.json";
import e52 from "../../content/exams/52-topik1.json";
import e52b from "../../content/exams/52-topik2.json";
import e60 from "../../content/exams/60-topik1.json";
import e60b from "../../content/exams/60-topik2.json";
import e64 from "../../content/exams/64-topik1.json";
import e64b from "../../content/exams/64-topik2.json";
import e83 from "../../content/exams/83-topik1.json";
import e83b from "../../content/exams/83-topik2.json";
import e91 from "../../content/exams/91-topik1.json";
import e91b from "../../content/exams/91-topik2.json";
import e96 from "../../content/exams/96-topik1.json";
import e96b from "../../content/exams/96-topik2.json";
import e102 from "../../content/exams/102-topik1.json";
import e102b from "../../content/exams/102-topik2.json";
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
} from "@/lib/exam-types";

export * from "@/lib/exam-types";

const EXAMS = [
  e102, e102b, e96, e96b, e91, e91b, e83, e83b, e64, e64b, e60, e60b, e52, e52b, e47, e47b, e41, e41b, e37, e37b, e36, e36b, e35, e35b,
] as unknown as Exam[];

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
  if (!q.audio) return [];
  const d = groupOf(section, q.no)?.dialogue;
  if (d && q.audio[0] >= d[1]) return [d, q.audio];
  return [q.audio];
}

/**
 * Đề câu hỏi chỉ là "(4점)" thì không cần hiện: số điểm đã có ở tiêu đề câu,
 * còn câu nghe thì nội dung nằm trong file nghe.
 */
export function isPointsOnly(c: Content): boolean {
  return !isImage(c) && /^\s*(\(\d+점\))?\s*$/.test(c.html);
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
